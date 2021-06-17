use actix_files::Files;
use actix_web::{web, App, HttpServer};

mod settings;
use crate::settings::SETTINGS;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    {
        let _settings = SETTINGS.read().unwrap();
    }
    HttpServer::new(|| {
        App::new()
            .route("/settings", web::post().to(settings::change_settings))
            .route("/settings", web::get().to(settings::get_settings))
            .service(Files::new("/", "./frontend/dist").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
