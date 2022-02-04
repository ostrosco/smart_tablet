use actix_files::Files;
use actix_rt::Arbiter;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use futures::{channel::mpsc, SinkExt, StreamExt};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{self, Message},
};

mod message;
mod news;
mod service;
mod settings;
mod voice;
mod weather;
use crate::news::NewsService;
use crate::service::ServiceHandler;
use crate::settings::SETTINGS;
use crate::weather::WeatherService;

#[get("/weather")]
/// Get the most recent weather that's been queried or return nothing if no weather information is
/// available.
async fn get_weather(service_handler: web::Data<Arc<ServiceHandler>>) -> HttpResponse {
    let weather_report = service_handler
        .get_latest_result(WeatherService::get_service_name())
        .await;
    match weather_report {
        Some(report) => HttpResponse::Ok()
            .content_type("application/json")
            .body(report),
        None => HttpResponse::NoContent().body("No weather available at this time"),
    }
}

#[get("/news")]
/// Get the most recent news that's been queried or return nothing if not news information is
/// available.
async fn get_news(service_handler: web::Data<Arc<ServiceHandler>>) -> HttpResponse {
    let news = service_handler
        .get_latest_result(NewsService::get_service_name())
        .await;
    match news {
        Some(news) => HttpResponse::Ok().body(news),
        None => HttpResponse::NoContent().body("No news available at this time"),
    }
}

/// Wrapper function to handle any errors that result from establishing the update
/// connection to the frontend.
async fn accept_update_connection(
    peer: SocketAddr,
    stream: TcpStream,
    update_rx: mpsc::UnboundedReceiver<String>,
) {
    if let Err(e) = handle_update_connection(peer, stream, update_rx).await {
        match e {
            tungstenite::Error::ConnectionClosed
            | tungstenite::Error::Protocol(_)
            | tungstenite::Error::Utf8 => (),
            err => eprintln!("Error processing connection: {}", err),
        }
    }
}

/// Accepts the websocket connection from the frontend and sends any updates from the running
/// services asynchronously to the frontend for handling.
async fn handle_update_connection(
    _peer: SocketAddr,
    stream: TcpStream,
    update_rx: mpsc::UnboundedReceiver<String>,
) -> tungstenite::Result<()> {
    let ws_stream = accept_async(stream)
        .await
        .expect("couldn't accept websocket");

    update_rx
        .fold(ws_stream, |mut ws_stream, update| async move {
            ws_stream
                .send(Message::Text(update))
                .await
                .expect("couldn't send update");
            ws_stream
        })
        .await;

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut arbiter = Arbiter::new();
    {
        let _settings = SETTINGS.read().unwrap();
    }

    // Start up the update websocket.
    let addr = "127.0.0.1:9000";
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Couldn't start listener on port 9000");
    let (update_tx, update_rx) = mpsc::unbounded();
    arbiter.spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let peer = stream
                .peer_addr()
                .expect("no peer address for new connection");
            accept_update_connection(peer, stream, update_rx).await
        }
    });

    // Start up all the relevant services in the service handler.
    let (request_tx, request_rx) = mpsc::unbounded();
    let mut service_handler = ServiceHandler::new(request_rx);
    service_handler.start_service(
        &mut arbiter,
        update_tx.clone(),
        Box::new(weather::WeatherService::new()),
    );
    service_handler.start_service(
        &mut arbiter,
        update_tx.clone(),
        Box::new(news::NewsService::new()),
    );
    service_handler.start_service(
        &mut arbiter,
        update_tx.clone(),
        Box::new(voice::CommandService::new(request_tx)),
    );
    service_handler.start_handler(&mut arbiter);

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
