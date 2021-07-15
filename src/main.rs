use actix_files::Files;
use actix_rt::Arbiter;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use async_trait::async_trait;
use futures::channel::mpsc;
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

mod news;
mod settings;
mod weather;
use crate::settings::SETTINGS;
use crate::weather::WeatherReport;

lazy_static! {
    pub static ref CURRENT_WEATHER: Arc<Mutex<Option<WeatherReport>>> = Arc::new(Mutex::new(None));
}

#[async_trait]
pub trait Service {
    async fn start_service(&mut self);
}

#[get("/weather")]
/// Get the most recent weather that's been queried or return nothing if no weather information is
/// available.
async fn get_weather() -> HttpResponse {
    let report;
    {
        report = CURRENT_WEATHER.lock().unwrap();
    }
    match &*report {
        Some(report) => HttpResponse::Ok().body(serde_json::to_string(&report).unwrap()),
        None => HttpResponse::BadRequest().body("No weather available"),
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
    let arbiter = Arbiter::new();
    {
        let _settings = SETTINGS.read().unwrap();
    }

    // Start up the weather service and the listener for results. Results are currently stored
    // in a cached static variable. This will likely move somewhere else.
    let (tx, rx) = mpsc::channel(1);
    let mut weather = weather::WeatherService::new(tx);
    arbiter.spawn(async move { weather.start_service().await });
    arbiter.spawn(async move {
        rx.for_each(|wr| async move { *CURRENT_WEATHER.lock().unwrap() = Some(wr) })
            .await
    });

    HttpServer::new(move || {
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
