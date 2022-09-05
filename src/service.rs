use crate::message::UpdateMessage;
use crate::voice::{command::Command, VoiceProcessing};
use actix_rt::{time::interval, Arbiter};
use async_trait::async_trait;
use erased_serde::Serialize;
use futures::channel::mpsc as futures_mpsc;
use futures::stream::StreamExt;
use futures::SinkExt;
use std::collections::HashMap;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};

#[async_trait]
/// A trait to represent services that asynchronously query information from the outside world
/// periodically and transmit the latest results to whomever is listening (usually the
/// ServiceHandler).
pub trait Service {
    // Gets the service name. This is used mostly for the ServiceHandler to keep track of the
    // latest results for a given service.
    fn get_service_name(&self) -> String;

    // Ask the service to perform an update and return its latest results.
    async fn update(&self) -> UpdateMessage;

    // Gets the polling rate in seconds for this service.
    fn get_polling_rate(&self) -> u64;
}

/// A structure to spawn services and listen for their latest results. Note that this erases the
/// types of the responses down to `dyn Serialize`, so this is used to hold the data that will
/// eventually be serialized and sent to the front end.
pub struct ServiceHandler {
    latest_results: Arc<Mutex<HashMap<String, Box<dyn Serialize + Send + Sync>>>>,
    request_rx: Option<
        futures_mpsc::UnboundedReceiver<(String, futures_mpsc::UnboundedSender<Option<String>>)>,
    >,
    voice_processing: VoiceProcessing,
    services: HashMap<String, Arc<dyn Service + Send + Sync>>,
    command_rx: Option<mpsc::UnboundedReceiver<Command>>,
}

impl ServiceHandler {
    pub fn new(
        request_rx: futures_mpsc::UnboundedReceiver<(
            String,
            futures_mpsc::UnboundedSender<Option<String>>,
        )>,
    ) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        Self {
            latest_results: Arc::new(Mutex::new(HashMap::new())),
            request_rx: Some(request_rx),
            voice_processing: VoiceProcessing::new(command_tx),
            command_rx: Some(command_rx),
            services: HashMap::new(),
        }
    }

    /// Take ownership of the service and spawn two tasks on the arbiter: one that runs the service
    /// indefiitely and one to receive its results and store it into the latest_results HashMap.
    pub fn start_service(
        &mut self,
        arbiter: &mut Arbiter,
        update_tx: futures_mpsc::UnboundedSender<String>,
        service: Arc<dyn Service + Send + Sync>,
    ) {
        let service_name = service.get_service_name();
        let latest_results = self.latest_results.clone();
        let service_to_poll = Arc::clone(&service);
        let mut interval = interval(Duration::from_secs(service.get_polling_rate()));
        self.services.insert(service_name.clone(), service);
        arbiter.spawn(async move {
            loop {
                // We do two things with the result we receive:
                // - we store it into the latest_results hashmap in case something later directly
                //   queries our latest result.
                // - we send it out to be transmitted to the websocket to go to the frontend.
                let service_result = service_to_poll.update().await;
                let mut update_tx = update_tx.clone();
                let mut map = latest_results.lock().await;
                let result = serde_json::to_string(&service_result).unwrap();
                println!("{}", result);
                map.insert(service_name.clone(), Box::new(service_result));
                update_tx.send(result).await.unwrap();
                interval.tick().await;
            }
        });
    }

    /// Start the service handler which will be composed of two parts: one which listens to the
    /// services that run periodically and one which handles commands said by the user and
    /// interacts with the running services directly.
    pub fn start_handler(&mut self, arbiter: &mut Arbiter) {
        let latest_results = self.latest_results.clone();
        let mut request_rx = self.request_rx.take().unwrap();
        let mut command_rx = self.command_rx.take().unwrap();
        arbiter.spawn(async move {
            loop {
                let msg = request_rx.next().await.unwrap();
                let (service_name, mut sender_tx) = msg;
                let map = latest_results.lock().await;
                let result = map
                    .get(&service_name)
                    .map(|res| serde_json::to_string(&res).unwrap());
                sender_tx.send(result.clone()).await.unwrap();
            }
        });

        self.voice_processing.start_listeners();

        arbiter.spawn(async move {
            loop {
                let command = command_rx.recv().await;
                println!("{:?}", command);
            }
        });
    }

    /// Returns the latest result for a given service or None if no results have been received for
    /// a given service.
    pub async fn get_latest_result(&self, service_name: String) -> Option<String> {
        let result_map = self.latest_results.lock().await;
        result_map
            .get(&service_name)
            .map(|res| serde_json::to_string(&res).unwrap())
    }
}
