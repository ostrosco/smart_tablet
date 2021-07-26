use actix_files::Files;
use actix_rt::Arbiter;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use std::sync::Arc;

mod news;
mod service;
mod settings;
mod weather;
use crate::news::NewsService;
use crate::service::ServiceHandler;
use crate::settings::SETTINGS;
use crate::weather::WeatherService;

#[get("/weather")]
/// Get the most recent weather that's been queried or return nothing if no weather information is
/// available.
async fn get_weather(service_handler: web::Data<Arc<ServiceHandler>>) -> HttpResponse {
    let weather_report = service_handler.get_latest_result(WeatherService::get_service_name());
    match weather_report {
        Some(report) => HttpResponse::Ok().content_type("application/json").body(report),
        None => HttpResponse::NoContent().body("No weather available at this time"),
    }
}

#[get("/news")]
/// Get the most recent news that's been queried or return nothing if not news information is
/// available.
async fn get_news(service_handler: web::Data<Arc<ServiceHandler>>) -> HttpResponse {
    let news = service_handler.get_latest_result(NewsService::get_service_name());
    match news {
        Some(news) => HttpResponse::Ok().body(news),
        None => HttpResponse::NoContent().body("No news available at this time"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut arbiter = Arbiter::new();
    {
        let _settings = SETTINGS.read().unwrap();
    }

    // Start up all the relevant services in the service handler.
    let service_handler = ServiceHandler::new();
    service_handler.start_service(&mut arbiter, Box::new(weather::WeatherService::new()));
    service_handler.start_service(&mut arbiter, Box::new(news::NewsService::new()));
    let service_handler = Arc::new(service_handler);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service_handler.clone()))
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
