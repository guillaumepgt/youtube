use actix_web::{get, web, HttpResponse, Responder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::env;

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

#[get("/videos/{query}")]
pub async fn videos(query: web::Path<String>) -> impl Responder {
    let api_key = env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY non défini");
    let query = query.into_inner();

    match get_videos(&api_key, &query).await {
        Ok(videos) => HttpResponse::Ok().json(videos),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erreur: {}", e)),
    }
}

async fn get_videos(api_key: &str, query: &str) -> Result<Vec<Video>, Box<dyn Error>> {
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

    // Étape 3: Récupérer les vidéos (avec snippet et contentDetails)
    let url = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&maxResults=20&key={}",
        uploads_playlist, api_key
    );
    let res: Value = client.get(&url).send().await?.json().await?;

    let mut video_list = vec![];
    if let Some(items) = res["items"].as_array() {
        for item in items {
            if let Some(video_id) = item["snippet"]["resourceId"]["videoId"].as_str() {
                // Récupérer les stats de la vidéo
                let stats_url = format!(
                    "https://www.googleapis.com/youtube/v3/videos?part=contentDetails,statistics&id={}&key={}",
                    video_id, api_key
                );
                let stats_res: Value = client.get(&stats_url).send().await?.json().await?;

                let duration = stats_res["items"]
                    .as_array()
                    .and_then(|items| items.get(0))
                    .and_then(|item| item["contentDetails"]["duration"].as_str())
                    .unwrap_or("PT0S")
                    .to_string();

                let view_count = stats_res["items"]
                    .as_array()
                    .and_then(|items| items.get(0))
                    .and_then(|item| item["statistics"]["viewCount"].as_str())
                    .unwrap_or("0")
                    .to_string();

                let video = Video {
                    video_id: video_id.to_string(),
                    title: item["snippet"]["title"]
                        .as_str()
                        .unwrap_or("Titre inconnu")
                        .to_string(),
                    description: item["snippet"]["description"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    thumbnail: item["snippet"]["thumbnails"]["medium"]["url"]
                        .as_str()
                        .unwrap_or("https://via.placeholder.com/320x180")
                        .to_string(),
                    channel_title: item["snippet"]["channelTitle"]
                        .as_str()
                        .unwrap_or("Chaîne inconnue")
                        .to_string(),
                    published_at: item["snippet"]["publishedAt"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    duration,
                    view_count,
                };

                video_list.push(video);
            }
        }
    }

    if video_list.is_empty() {
        return Err("Aucune vidéo trouvée".into());
    }

    Ok(video_list)
}