use actix_files::Files;
use actix_web::{get, web, App, HttpResponse, HttpServer};

mod news;
mod settings;
mod weather;
use crate::settings::SETTINGS;

#[get("/weather/{lat}/{lon}")]
async fn get_weather(info: web::Path<(f32, f32)>) -> HttpResponse {
    let info = info.into_inner();
    let weather = weather::Weather();
    let report = weather.get_weather_report(info.0, info.1).await;
    match report {
        Ok(report) => HttpResponse::Ok().body(serde_json::to_string(&report).unwrap()),
        Err(err) => HttpResponse::BadRequest().body(format!("{:?}", err)),
    }
}

#[get("/news")]
async fn get_news() -> HttpResponse {
    let news = news::get_news().await;
    match news {
        Ok(news) => HttpResponse::Ok().body(serde_json::to_string(&news).unwrap()),
        Err(err) => HttpResponse::BadRequest().body(format!("{:?}", err)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    {
        let _settings = SETTINGS.read().unwrap();
    }
    HttpServer::new(|| {
        App::new()
            .route("/settings", web::post().to(settings::change_settings))
            .route("/settings", web::get().to(settings::get_settings))
            .service(get_weather)
            .service(get_news)
            .service(Files::new("/", "./frontend/dist").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
