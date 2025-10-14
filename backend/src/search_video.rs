use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct VideoSnippet {
    title: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VideoId {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VideoItem {
    id: VideoId,
    snippet: VideoSnippet,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    items: Vec<VideoItem>,
}

const YOUTUBE_API_URL: &str = "https://www.googleapis.com/youtube/v3/search";

#[get("/search/{query}")]
async fn search_youtube_videos(path: web::Path<String>) -> impl Responder {
    let api_key = match env::var("YOUTUBE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("ERREUR: La variable d'environnement YOUTUBE_API_KEY n'est pas définie.");
            return HttpResponse::InternalServerError().body("Erreur de configuration du serveur.");
        }
    };

    let search_query = path.into_inner();

    let url = format!(
        "{}?part=snippet&type=video&maxResults=50&q={}&key={}",
        YOUTUBE_API_URL,
        search_query,
        api_key
    );

    let client = reqwest::Client::new();
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(_) => return HttpResponse::InternalServerError().body("Erreur lors de la communication avec l'API YouTube."),
    };

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_else(|_| "Impossible de lire le corps de l'erreur.".to_string());
        eprintln!("Erreur de l'API YouTube ({}): {}", status, error_body);
        return HttpResponse::build(status).body(format!("L'API YouTube a retourné une erreur: {}", error_body));
    }

    match response.json::<SearchResponse>().await {
        Ok(search_results) => HttpResponse::Ok().json(search_results),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la désérialisation de la réponse de YouTube."),
    }
}