mod auth;
mod cache;
mod calendar;
mod commits;
mod date_time_serializer;
mod envvar;
mod graphql;
mod models;
mod repositories;
mod templates;
mod web;

use actix_web::{App, HttpServer};

const DEFAULT_IP: &str = "0.0.0.0";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut port = 8080;
    if let Ok(p) = envvar::get("PORT") {
        match p.parse::<u16>() {
            Ok(n) => port = n,
            Err(e) => eprintln!(
                "could not parse {} to integer, using the default port {}: {}",
                p, port, e
            ),
        }
    }
    println!("Starting the server at {}:{}", DEFAULT_IP, port);
    HttpServer::new(|| {
        App::new()
            .service(web::index)
            .service(web::callback)
            .service(web::calendar)
    })
    .bind((DEFAULT_IP, port))?
    .run()
    .await
}
