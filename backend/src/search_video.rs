use actix_web::{get, web,  HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::env;

// Structure de sortie finale (inchangée)
#[derive(Serialize, Deserialize, Debug)]
pub struct Video {
    pub video_id: String,
    pub title: String,
    pub description: String,
    pub thumbnail: String,
    pub channel_title: String,
    pub published_at: String,
    pub duration: String,
    pub view_count: String,
}

// --- Structures pour l'endpoint /search (Étape 1) ---
#[derive(Deserialize, Debug)]
struct SearchResultItem {
    id: SearchResultId,
}

#[derive(Deserialize, Debug)]
struct SearchResultId {
    // MODIFICATION 1: Le video_id est maintenant optionnel
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Deserialize, Debug)]
struct SearchResponse {
    items: Vec<SearchResultItem>,
}

// --- Structures pour l'endpoint /videos (Étape 2) (inchangées) ---
#[derive(Deserialize, Debug)]
struct VideoListResponse {
    items: Vec<VideoDetails>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VideoDetails {
    id: String,
    snippet: VideoSnippet,
    content_details: ContentDetails,
    statistics: Statistics,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VideoSnippet {
    published_at: String,
    title: String,
    description: String,
    thumbnails: Thumbnails,
    channel_title: String,
}
#[derive(Deserialize, Debug)]
struct Thumbnails {
    high: ThumbnailInfo,
}
#[derive(Deserialize, Debug)]
struct ThumbnailInfo {
    url: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ContentDetails {
    duration: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Statistics {
    view_count: String,
}

// --- Gestionnaire de route ---
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
    let client = reqwest::Client::new();

    // --- ÉTAPE 1: Rechercher les vidéos pour obtenir leurs IDs ---
    let search_url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video&maxResults=50&q={}&key={}",
        search_query, api_key
    );

    let search_response = match client.get(&search_url).send().await {
        Ok(resp) => resp,
        Err(_) => return HttpResponse::InternalServerError().body("Étape 1: Erreur de communication avec l'API YouTube."),
    };

    if !search_response.status().is_success() {
        return HttpResponse::new(search_response.status());
    }

    // Amélioration du logging pour le débogage
    let search_results = match search_response.json::<SearchResponse>().await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Erreur de désérialisation (Étape 1): {:?}", e);
            return HttpResponse::InternalServerError().body("Étape 1: Erreur de désérialisation.");
        }
    };

    // MODIFICATION 2: Utiliser filter_map pour ignorer les résultats sans video_id
    let video_ids = search_results.items.into_iter()
        .filter_map(|item| item.id.video_id)
        .collect::<Vec<String>>()
        .join(",");

    // Si après filtrage il n'y a plus aucun ID, on retourne un tableau vide.
    if video_ids.is_empty() {
        return HttpResponse::Ok().json(Vec::<Video>::new());
    }

    // --- ÉTAPE 2: Obtenir les détails complets (inchangée) ---
    let details_url = format!(
        "https://www.googleapis.com/youtube/v3/videos?part=snippet,contentDetails,statistics&id={}&key={}",
        video_ids, api_key
    );

    let details_response = match client.get(&details_url).send().await {
        Ok(resp) => resp,
        Err(_) => return HttpResponse::InternalServerError().body("Étape 2: Erreur de communication avec l'API YouTube."),
    };

    if !details_response.status().is_success() {
        return HttpResponse::new(details_response.status());
    }

    let video_details_list = match details_response.json::<VideoListResponse>().await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Erreur de désérialisation (Étape 2): {:?}", e);
            return HttpResponse::InternalServerError().body("Étape 2: Erreur de désérialisation des détails vidéo.");
        }
    };

    // --- ÉTAPE 3: Transformer les données (inchangée) ---
    let final_videos: Vec<Video> = video_details_list.items.into_iter().map(|detail| {
        Video {
            video_id: detail.id,
            title: detail.snippet.title,
            description: detail.snippet.description,
            thumbnail: detail.snippet.thumbnails.high.url,
            channel_title: detail.snippet.channel_title,
            published_at: detail.snippet.published_at,
            duration: detail.content_details.duration,
            view_count: detail.statistics.view_count,
        }
    }).collect();

    HttpResponse::Ok().json(final_videos)
}
