use actix_web::{get, web, HttpResponse};
use oauth2::{AuthorizationCode, TokenResponse};
use oauth2::reqwest::async_http_client;
use std::fs;

use crate::models::SavedToken;
use crate::auth::oauth_client;

#[get("/")]
pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body("<html>Bienvenue 👋<br><a href=\"/login\">Se connecter avec Google</a></html>")
}

#[get("/login")]
pub async fn login() -> HttpResponse {
    let client = oauth_client();
    let (auth_url, _csrf_token) = client
        .authorize_url(|| oauth2::CsrfToken::new("random".to_string()))
        .add_scope(oauth2::Scope::new(
            "https://www.googleapis.com/auth/youtube.readonly".to_string(),
        ))
        .url();

    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

#[get("/auth/callback")]
pub async fn callback(query: web::Query<std::collections::HashMap<String, String>>) -> HttpResponse {
    let client = oauth_client();

    if let Some(code) = query.get("code") {
        let token_result = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await;

        match token_result {
            Ok(token) => {
                let saved = SavedToken {
                    access_token: token.access_token().secret().to_string(),
                    refresh_token: token.refresh_token().map(|r| r.secret().to_string()),
                    expires_in: token.expires_in().map(|d| d.as_secs()),
                };

                fs::write("token.json", serde_json::to_string_pretty(&saved).unwrap())
                    .expect("Impossible d'écrire le token");

                HttpResponse::Ok().body("✅ Token sauvegardé dans token.json")
            }
            Err(err) => HttpResponse::InternalServerError().body(format!("Erreur: {:?}", err)),
        }
    } else {
        HttpResponse::BadRequest().body("Pas de code dans la requête")
    }
}

#[get("/subscriptions")]
pub async fn subscriptions() -> HttpResponse {
    let data = std::fs::read_to_string("token.json").unwrap();
    let saved: crate::models::SavedToken = serde_json::from_str(&data).unwrap();

    let mut all_items = Vec::new();
    let client = reqwest::Client::new();

    let mut page_token: Option<String> = None;

    loop {
        let mut url = format!(
            "https://www.googleapis.com/youtube/v3/subscriptions?part=snippet&mine=true&maxResults=50"
        );

        if let Some(token) = &page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let res = client
            .get(&url)
            .bearer_auth(&saved.access_token)
            .send()
            .await;

        match res {
            Ok(r) => {
                let body = r.json::<serde_json::Value>().await.unwrap();
                if let Some(items) = body["items"].as_array() {
                    all_items.extend(items.clone());
                }

                // Vérifier s'il y a une page suivante
                if let Some(next) = body["nextPageToken"].as_str() {
                    page_token = Some(next.to_string());
                } else {
                    break; // pas de page suivante
                }
            }
            Err(e) => return HttpResponse::InternalServerError().body(format!("Erreur reqwest: {}", e)),
        }
    }

    // Retourner toutes les chaînes trouvées
    HttpResponse::Ok().json(all_items)
}

#[post("/refresh")]
pub async fn refresh() -> HttpResponse {
    let client = oauth_client();

    // Charger le token sauvegardé
    let data = match std::fs::read_to_string("token.json") {
        Ok(d) => d,
        Err(_) => return HttpResponse::InternalServerError().body("Impossible de lire token.json"),
    };

    let saved: SavedToken = match serde_json::from_str(&data) {
        Ok(s) => s,
        Err(_) => return HttpResponse::InternalServerError().body("Token corrompu"),
    };

    if let Some(refresh_token) = saved.refresh_token {
        match super::refresh_oauth_token(&client, &refresh_token).await {
            Ok(new_token) => {
                // Sauvegarder le nouveau token
                if let Err(e) = std::fs::write("token.json", serde_json::to_string_pretty(&new_token).unwrap()) {
                    return HttpResponse::InternalServerError().body(format!("Erreur sauvegarde: {}", e));
                }
                HttpResponse::Ok().json(new_token)
            }
            Err(err) => HttpResponse::InternalServerError().body(err),
        }
    } else {
        HttpResponse::BadRequest().body("Aucun refresh_token disponible")
    }
}