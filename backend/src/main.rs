use actix_web::{App, HttpServer, http};
use actix_cors::Cors;
use dotenv::dotenv;

mod auth;
mod subscriptions;
mod videos;
mod models;
mod search_video;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("Serveur démarré sur http://localhost:8080");
    println!("Routes disponibles:");
    println!("  GET  /");
    println!("  GET  /login");
    println!("  GET  /auth/callback");
    println!("  GET  /subscriptions");
    println!("  GET  /subscriptions/videos");

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::CONTENT_TYPE,
                        http::header::HeaderName::from_static("refresh_token"),
                        http::header::HeaderName::from_static("expires_in"),
                    ])
                    .expose_headers(vec![
                        http::header::AUTHORIZATION,
                    ])
                    .max_age(3600)
            )
            .service(subscriptions::index)
            .service(subscriptions::login)
            .service(subscriptions::callback)
            .service(subscriptions::subscriptions)
            .service(subscriptions::subscriptions_videos)
            .service(videos::videos)
            .service(search_video::search_youtube_videos)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}