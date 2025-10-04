use actix_web::{get, web, HttpResponse, Responder};
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use std::env;

#[get("/videos/{query}")]
pub async fn videos(query: web::Path<String>) -> impl Responder {
    let api_key = env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY non défini");
    let query = query.into_inner();

    match get_videos(&api_key, &query).await {
        Ok(videos) => HttpResponse::Ok().json(videos),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erreur: {}", e)),
    }
}

async fn get_videos(api_key: &str, query: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let client = Client::new();

    // Étape 1: Chercher la chaîne
    let search_url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=channel&q={}&key={}",
        query, api_key
    );
    let search_res: Value = client.get(&search_url).send().await?.json().await?;
    let channel_id = search_res["items"]
        .as_array()
        .and_then(|items| items.get(0))
        .and_then(|item| item["id"]["channelId"].as_str())
        .ok_or("Impossible de récupérer le channelId")?;

    // Étape 2: Playlist des uploads
    let url = format!(
        "https://www.googleapis.com/youtube/v3/channels?part=contentDetails&id={}&key={}",
        channel_id, api_key
    );
    let res: Value = client.get(&url).send().await?.json().await?;
    let uploads_playlist = res["items"]
        .as_array()
        .and_then(|items| items.get(0))
        .and_then(|item| item["contentDetails"]["relatedPlaylists"]["uploads"].as_str())
        .ok_or("Impossible de récupérer la playlist")?;

    // Étape 3: Récupérer les vidéos
    let url = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&maxResults=5&key={}",
        uploads_playlist, api_key
    );
    let res: Value = client.get(&url).send().await?.json().await?;

    let mut links = vec![];
    if let Some(items) = res["items"].as_array() {
        for item in items {
            if let Some(video_id) = item["snippet"]["resourceId"]["videoId"].as_str() {
                links.push(format!("https://www.yout-ube.com/watch?v={}", video_id));
            }
        }
    }

    if links.is_empty() {
        return Err("Aucune vidéo trouvée".into());
    }

    Ok(links)
}