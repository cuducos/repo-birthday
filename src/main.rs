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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut port = web::DEFAULT_PORT;
    if let Ok(p) = envvar::get("PORT") {
        match p.parse::<u16>() {
            Ok(n) => port = n,
            Err(e) => eprintln!(
                "could not parse {} to integer, using the default port {}: {}",
                p, port, e
            ),
        }
    }
    println!("Starting the server at {}:{}", web::DEFAULT_IP, port);
    HttpServer::new(|| {
        App::new()
            .service(web::index)
            .service(web::callback)
            .service(web::calendar)
            .service(web::calendar_alt)
            .service(web::view)
    })
    .bind((web::DEFAULT_IP, port))?
    .run()
    .await
}
