use actix_web::{get, web, HttpResponse, HttpRequest};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::basic::BasicClient;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::env;
use chrono::{DateTime, Utc};
use log::{info, error, warn};
use futures::future::join_all;
use serde::{Deserialize, Serialize};

use crate::auth::oauth_client;
use crate::models::{SavedToken, Video};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCallbackQuery {
    code: String,
    state: String,
}

#[get("/")]
pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body("<html>Bienvenue 👋<br><a href=\"/login\">Se connecter avec Google</a></html>")
}

#[get("/login")]
pub async fn login() -> HttpResponse {
    let client = oauth_client();
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("https://www.googleapis.com/auth/youtube.readonly".to_string()))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent")
        .url();

    info!("Redirection vers Google OAuth: {}", auth_url);

    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

#[get("/auth/callback")]
pub async fn callback(query: web::Query<AuthCallbackQuery>) -> HttpResponse {
    info!("Callback reçu avec query: {:?}", query);

    let client = oauth_client();
    let code = AuthorizationCode::new(query.code.clone());

    info!("Tentative d'échange du code...");
    let token_result = client
        .exchange_code(code)
        .request_async(async_http_client)
        .await;

    match token_result {
        Ok(token) => {
            info!("✓ Échange du code réussi !");

            let access_token = token.access_token().secret().to_string();
            let refresh_token = token.refresh_token()
                .map(|r| r.secret().to_string())
                .unwrap_or_default();
            let expires_in = token.expires_in().map(|d| d.as_secs()).unwrap_or(3600);

            info!("Tokens obtenus:");
            info!("  - access_token: {} caractères", access_token.len());
            info!("  - refresh_token: {}", if refresh_token.is_empty() { "ABSENT" } else { "présent" });
            info!("  - expires_in: {} secondes", expires_in);

            let redirect_url = format!(
                "http://localhost:3000/?access_token={}&refresh_token={}&expires_in={}",
                access_token, refresh_token, expires_in
            );

            info!("Redirection vers: {}", redirect_url);

            let html = format!(
                r#"<!DOCTYPE html>
                <html>
                <head>
                    <meta charset="UTF-8">
                    <title>Redirection...</title>
                </head>
                <body>
                    <p>Authentification réussie, redirection en cours...</p>
                    <script>
                        window.location.href = "{}";
                    </script>
                </body>
                </html>"#,
                redirect_url
            );

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(err) => {
            error!("✗ Erreur d'échange de code: {:?}", err);

            let html = r#"<!DOCTYPE html>
                <html>
                <head>
                    <meta charset="UTF-8">
                    <title>Erreur</title>
                </head>
                <body>
                    <p>Erreur d'authentification, redirection en cours...</p>
                    <script>
                        window.location.href = "http://localhost:3000/?error=auth_failed";
                    </script>
                </body>
                </html>"#;

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
    }
}

async fn refresh_oauth_token(client: &BasicClient, refresh_token: &str) -> Result<SavedToken, String> {
    match client
        .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
        .request_async(async_http_client)
        .await
    {
        Ok(token) => {
            let saved = SavedToken {
                access_token: token.access_token().secret().to_string(),
                refresh_token: token.refresh_token().map(|r| r.secret().to_string()),
                expires_in: token.expires_in().map(|d| d.as_secs()),
                issued_at: Utc::now(),
            };
            Ok(saved)
        }
        Err(e) => {
            error!("Erreur lors du rafraîchissement du token: {:?}", e);
            Err(format!("Erreur lors du rafraîchissement du token: {:?}", e))
        }
    }
}

#[get("/subscriptions")]
pub async fn subscriptions(req: HttpRequest) -> HttpResponse {
    let access_token = match req.headers().get("authorization") {
        Some(auth) => {
            match auth.to_str() {
                Ok(auth_str) => auth_str.replace("Bearer ", ""),
                Err(_) => return HttpResponse::Unauthorized().body("Token invalide"),
            }
        }
        None => return HttpResponse::Unauthorized().body("Aucun token d'accès fourni"),
    };

    let client = Client::new();
    let mut all_items = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let mut url = format!(
            "https://www.googleapis.com/youtube/v3/subscriptions?part=snippet&mine=true&maxResults=50"
        );
        if let Some(token) = &page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        info!("Envoi de la requête à l'API YouTube: {}", url);
        let res = match client.get(&url).bearer_auth(&access_token).send().await {
            Ok(r) => r,
            Err(e) => {
                error!("Erreur reqwest pour /subscriptions: {}", e);
                return HttpResponse::InternalServerError().body(format!("Erreur reqwest: {}", e));
            }
        };

        if res.status().is_success() {
            let body: Value = match res.json().await {
                Ok(body) => body,
                Err(e) => {
                    error!("Erreur de parsing de la réponse /subscriptions: {}", e);
                    return HttpResponse::InternalServerError().body(format!("Erreur de parsing: {}", e));
                }
            };
            info!("Réponse reçue pour /subscriptions, items: {}", body["items"].as_array().map_or(0, |items| items.len()));
            if let Some(items) = body["items"].as_array() {
                all_items.extend(items.clone());
            }

            page_token = body["nextPageToken"].as_str().map(|s| s.to_string());
            if page_token.is_none() {
                break;
            }
        } else {
            let status = res.status();
            let error_body = res.text().await.unwrap_or_default();
            error!("Erreur HTTP {} pour /subscriptions: {}", status, error_body);
            return HttpResponse::InternalServerError().body(format!("Erreur HTTP {}: {}", status, error_body));
        }
    }

    if all_items.is_empty() {
        warn!("Aucun abonnement trouvé pour l'utilisateur");
    }

    HttpResponse::Ok().json(all_items)
}

#[get("/subscriptions/videos")]
pub async fn subscriptions_videos(req: HttpRequest) -> HttpResponse {
    let access_token = match req.headers().get("authorization") {
        Some(auth) => {
            match auth.to_str() {
                Ok(auth_str) => auth_str.replace("Bearer ", ""),
                Err(_) => return HttpResponse::Unauthorized().body("Token invalide"),
            }
        }
        None => return HttpResponse::Unauthorized().body("Aucun token d'accès fourni"),
    };

    let client_oauth = oauth_client();
    let mut saved: SavedToken = SavedToken {
        access_token: access_token.clone(),
        refresh_token: None,
        expires_in: None,
        issued_at: Utc::now(),
    };

    if let Some(refresh_token) = req.headers().get("refresh_token").and_then(|v| v.to_str().ok()) {
        saved.refresh_token = Some(refresh_token.to_string());

        if let Some(expires_in) = req.headers().get("expires_in")
            .and_then(|v| v.to_str().ok())
            .and_then(|e| e.parse::<u64>().ok())
        {
            if Utc::now() > saved.issued_at + chrono::Duration::seconds(expires_in as i64 - 300) {
                info!("Token expiré ou presque, tentative de rafraîchissement");
                saved = match refresh_oauth_token(&client_oauth, refresh_token).await {
                    Ok(token) => token,
                    Err(e) => return HttpResponse::InternalServerError().body(e),
                };
            }
        }
    }

    let client = Client::new();
    let api_key = match env::var("YOUTUBE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            error!("YOUTUBE_API_KEY non défini");
            return HttpResponse::InternalServerError().body("YOUTUBE_API_KEY non défini");
        }
    };
    let mut channel_ids: Vec<String> = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let mut url = format!(
            "https://www.googleapis.com/youtube/v3/subscriptions?part=snippet&mine=true&maxResults=50"
        );
        if let Some(token) = &page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        info!("Envoi de la requête à l'API YouTube pour abonnements: {}", url);
        let res = match client.get(&url).bearer_auth(&saved.access_token).send().await {
            Ok(r) => r,
            Err(e) => {
                error!("Erreur reqwest pour /subscriptions: {}", e);
                return HttpResponse::InternalServerError().body(format!("Erreur reqwest: {}", e));
            }
        };

        if res.status().is_success() {
            let body: Value = match res.json().await {
                Ok(body) => body,
                Err(e) => {
                    error!("Erreur de parsing de la réponse /subscriptions: {}", e);
                    return HttpResponse::InternalServerError().body(format!("Erreur de parsing: {}", e));
                }
            };
            let items_count = body["items"].as_array().map_or(0, |items| items.len());
            info!("Réponse reçue pour /subscriptions, items: {}", items_count);
            if items_count == 0 {
                warn!("Aucun abonnement trouvé pour l'utilisateur");
            }

            if let Some(items) = body["items"].as_array() {
                for item in items {
                    if let Some(channel_id) = item["snippet"]["resourceId"]["channelId"].as_str() {
                        channel_ids.push(channel_id.to_string());
                    }
                }
            }

            page_token = body["nextPageToken"].as_str().map(|s| s.to_string());
            if page_token.is_none() {
                break;
            }
        } else {
            let status = res.status();
            let error_body = res.text().await.unwrap_or_default();
            error!("Erreur HTTP {} pour /subscriptions: {}", status, error_body);
            return HttpResponse::InternalServerError().body(format!("Erreur HTTP {}: {}", status, error_body));
        }
    }

    info!("Nombre total d'abonnements récupérés: {}", channel_ids.len());
    if channel_ids.is_empty() {
        warn!("Aucun abonnement trouvé, retour d'un message");
        return HttpResponse::Ok().json(serde_json::json!({"message": "Aucun abonnement trouvé"}));
    }

    let mut uploads: Vec<String> = Vec::new();
    for chunk in channel_ids.chunks(50) {
        let ids = chunk.join(",");
        let url = format!(
            "https://www.googleapis.com/youtube/v3/channels?part=contentDetails&id={}&key={}",
            ids, api_key
        );

        info!("Envoi de la requête pour /channels: {}", url);
        let res = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                error!("Erreur reqwest pour /channels: {}", e);
                continue;
            }
        };

        if res.status().is_success() {
            let body: Value = match res.json().await {
                Ok(body) => body,
                Err(e) => {
                    error!("Erreur de parsing de la réponse /channels: {}", e);
                    continue;
                }
            };
            if let Some(items) = body["items"].as_array() {
                for item in items {
                    if let Some(pid) = item["contentDetails"]["relatedPlaylists"]["uploads"].as_str() {
                        uploads.push(pid.to_string());
                    } else {
                        warn!("Aucune playlist d'uploads pour la chaîne {:?}", item["id"]);
                    }
                }
            }
        } else if res.status() == StatusCode::TOO_MANY_REQUESTS {
            warn!("Erreur 429 pour /channels, attente de 60 secondes avant réessai");
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            let retry_res = match client.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    error!("Erreur reqwest pour /channels (réessai): {}", e);
                    continue;
                }
            };
            if retry_res.status().is_success() {
                let body: Value = match retry_res.json().await {
                    Ok(body) => body,
                    Err(e) => {
                        error!("Erreur de parsing de la réponse /channels (réessai): {}", e);
                        continue;
                    }
                };
                if let Some(items) = body["items"].as_array() {
                    for item in items {
                        if let Some(pid) = item["contentDetails"]["relatedPlaylists"]["uploads"].as_str() {
                            uploads.push(pid.to_string());
                        }
                    }
                }
            } else {
                let status = retry_res.status();
                let error_body = retry_res.text().await.unwrap_or_default();
                error!("Erreur HTTP {} pour /channels (réessai): {}", status, error_body);
            }
        } else {
            let status = res.status();
            let error_body = res.text().await.unwrap_or_default();
            error!("Erreur HTTP {} pour /channels: {}", status, error_body);
        }
    }

    info!("Nombre total de playlists d'uploads: {}", uploads.len());
    if uploads.is_empty() {
        warn!("Aucune playlist d'uploads trouvée");
        return HttpResponse::Ok().json(serde_json::json!({"message": "Aucune playlist d'uploads trouvée"}));
    }

    let futures: Vec<_> = uploads.into_iter().map(|pid| {
        let client = client.clone();
        let api_key = api_key.clone();
        async move {
            let mut videos: Vec<Video> = Vec::new();
            let mut video_page_token: Option<String> = None;

            loop {
                let mut url = format!(
                    "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&maxResults=5&key={}",
                    pid, api_key
                );
                if let Some(token) = &video_page_token {
                    url.push_str(&format!("&pageToken={}", token));
                }

                info!("Envoi de la requête pour /playlistItems: {}", url);
                let res = match client.get(&url).send().await {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Erreur reqwest pour /playlistItems (playlist {}): {}", pid, e);
                        return Vec::new();
                    }
                };

                if res.status().is_success() {
                    let body: Value = match res.json().await {
                        Ok(body) => body,
                        Err(e) => {
                            error!("Erreur de parsing de la réponse /playlistItems (playlist {}): {}", pid, e);
                            return Vec::new();
                        }
                    };
                    let video_count = body["items"].as_array().map_or(0, |items| items.len());
                    info!("Nombre de vidéos récupérées pour la playlist {}: {}", pid, video_count);

                    if let Some(video_items) = body["items"].as_array() {
                        for video_item in video_items {
                            if let Some(video_id) = video_item["snippet"]["resourceId"]["videoId"].as_str() {
                                if let Some(published_at) = video_item["snippet"]["publishedAt"].as_str() {
                                    match DateTime::parse_from_rfc3339(published_at) {
                                        Ok(date) => {
                                            let title = video_item["snippet"]["title"]
                                                .as_str()
                                                .unwrap_or("Sans titre")
                                                .to_string();

                                            let thumbnail = video_item["snippet"]["thumbnails"]["medium"]["url"]
                                                .as_str()
                                                .or_else(|| video_item["snippet"]["thumbnails"]["default"]["url"].as_str())
                                                .unwrap_or("")
                                                .to_string();

                                            let channel_title = video_item["snippet"]["channelTitle"]
                                                .as_str()
                                                .unwrap_or("Chaîne inconnue")
                                                .to_string();

                                            videos.push(Video {
                                                url: format!("https://www.youtube.com/watch?v={}", video_id),
                                                video_id: video_id.to_string(),
                                                published_at: date.with_timezone(&Utc),
                                                title,
                                                thumbnail,
                                                channel_title,
                                            });
                                            info!("Vidéo ajoutée: {} (publiée le {})", video_id, published_at);
                                        }
                                        Err(e) => {
                                            error!("Erreur lors du parsing de la date pour la vidéo {}: {}", video_id, e);
                                        }
                                    }
                                } else {
                                    warn!("Aucune date de publication pour la vidéo {}", video_id);
                                }
                            }
                        }
                    }

                    video_page_token = body["nextPageToken"].as_str().map(|s| s.to_string());
                    if video_page_token.is_none() || videos.len() >= 5 {
                        break;
                    }
                } else if res.status() == StatusCode::TOO_MANY_REQUESTS {
                    warn!("Erreur 429 pour /playlistItems (playlist {}), attente de 60 secondes avant réessai", pid);
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    let retry_res = match client.get(&url).send().await {
                        Ok(r) => r,
                        Err(e) => {
                            error!("Erreur reqwest pour /playlistItems (réessai, playlist {}): {}", pid, e);
                            return Vec::new();
                        }
                    };
                    if retry_res.status().is_success() {
                        let body: Value = match retry_res.json().await {
                            Ok(body) => body,
                            Err(e) => {
                                error!("Erreur de parsing de la réponse /playlistItems (réessai, playlist {}): {}", pid, e);
                                return Vec::new();
                            }
                        };
                        let video_count = body["items"].as_array().map_or(0, |items| items.len());
                        info!("Nombre de vidéos récupérées pour la playlist {} (réessai): {}", pid, video_count);
                        if let Some(video_items) = body["items"].as_array() {
                            for video_item in video_items {
                                if let Some(video_id) = video_item["snippet"]["resourceId"]["videoId"].as_str() {
                                    if let Some(published_at) = video_item["snippet"]["publishedAt"].as_str() {
                                        match DateTime::parse_from_rfc3339(published_at) {
                                            Ok(date) => {
                                                let title = video_item["snippet"]["title"]
                                                    .as_str()
                                                    .unwrap_or("Sans titre")
                                                    .to_string();

                                                let thumbnail = video_item["snippet"]["thumbnails"]["medium"]["url"]
                                                    .as_str()
                                                    .or_else(|| video_item["snippet"]["thumbnails"]["default"]["url"].as_str())
                                                    .unwrap_or("")
                                                    .to_string();

                                                let channel_title = video_item["snippet"]["channelTitle"]
                                                    .as_str()
                                                    .unwrap_or("Chaîne inconnue")
                                                    .to_string();

                                                videos.push(Video {
                                                    url: format!("https://www.youtube.com/watch?v={}", video_id),
                                                    published_at: date.with_timezone(&Utc),
                                                    video_id: video_id.to_string(),
                                                    title,
                                                    thumbnail,
                                                    channel_title,
                                                });
                                                info!("Vidéo ajoutée: {} (publiée le {})", video_id, published_at);
                                            }
                                            Err(e) => {
                                                error!("Erreur lors du parsing de la date pour la vidéo {}: {}", video_id, e);
                                            }
                                        }
                                    } else {
                                        warn!("Aucune date de publication pour la vidéo {}", video_id);
                                    }
                                }
                            }
                        }
                        video_page_token = body["nextPageToken"].as_str().map(|s| s.to_string());
                        if video_page_token.is_none() || videos.len() >= 5 {
                            break;
                        }
                    } else {
                        let status = retry_res.status();
                        let error_body = retry_res.text().await.unwrap_or_default();
                        error!("Erreur HTTP {} pour /playlistItems (réessai, playlist {}): {}", status, pid, error_body);
                        break;
                    }
                } else {
                    let status = res.status();
                    let error_body = res.text().await.unwrap_or_default();
                    error!("Erreur HTTP {} pour /playlistItems (playlist {}): {}", status, pid, error_body);
                    break;
                }
            }
            videos
        }
    }).collect();

    let all_videos: Vec<Video> = join_all(futures).await.into_iter().flatten().collect();

    info!("Nombre total de vidéos collectées: {}", all_videos.len());
    if all_videos.is_empty() {
        warn!("Aucune vidéo collectée après traitement des abonnements");
        return HttpResponse::Ok().json(serde_json::json!({"message": "Aucune vidéo trouvée pour les abonnements"}));
    }

    let mut sorted_videos = all_videos;
    sorted_videos.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    info!("Nombre de vidéos retournées: {}", sorted_videos.len());
    HttpResponse::Ok().json(sorted_videos)
}