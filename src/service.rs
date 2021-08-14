use actix_rt::Arbiter;
use async_trait::async_trait;
use erased_serde::Serialize;
use futures::channel::mpsc;
use futures::stream::StreamExt;
use futures::SinkExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
/// A trait to represent services that asynchronously query information from the outside world
/// periodically and transmit the latest results to whomever is listening (usually the
/// ServiceHandler).
pub trait Service {
    // Sets the sender within the service so that it can transmit its results out as it gets them.
    fn set_sender(&mut self, tx: mpsc::Sender<Box<dyn Serialize + Send + Sync>>);

    // Starts the service. Currently, services run indefinitely.
    async fn start_service(&mut self);

    // Gets the service name. This is used mostly for the ServiceHandler to keep track of the
    // latest results for a given service.
    fn get_service_name(&self) -> String;
}

/// A structure to spawn services and listen for their latest results. Note that this erases the
/// types of the responses down to `dyn Serialize`, so this is used to hold the data that will
/// eventually be serialized and sent to the front end.
pub struct ServiceHandler {
    latest_results: Arc<Mutex<HashMap<String, Box<dyn Serialize + Send + Sync>>>>,
}

impl ServiceHandler {
    pub fn new() -> Self {
        Self {
            latest_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Take ownership of the service and spawn two tasks on the arbiter: one that runs the service
    /// indefiitely and one to receive its results and store it into the latest_results HashMap.
    pub fn start_service(
        &self,
        arbiter: &mut Arbiter,
        update_tx: mpsc::UnboundedSender<String>,
        mut service: Box<dyn Service + Send>,
    ) {
        let (tx, rx) = mpsc::channel(1);
        service.set_sender(tx);
        let service_name = service.get_service_name();
        let latest_results = self.latest_results.clone();
        arbiter.spawn(async move { service.start_service().await });
        arbiter.spawn(async move {
            rx.for_each(|wr| async {
                // We do two things with the result we receive:
                // - we store it into the latest_results hashmap in case something later directly
                //   queries our latest result.
                // - we send it out to be transmitted to the websocket to go to the frontend.
                let mut update_tx = update_tx.clone();
                let mut map = latest_results.lock().await;
                let result = serde_json::to_string(&wr).unwrap();
                map.insert(service_name.clone(), wr);
                update_tx.send(result).await.unwrap();
            })
            .await
        });
    }

    /// Returns the latest result for a given service or None if no results have been received for
    /// a given service.
    pub async fn get_latest_result(&self, service_name: String) -> Option<String> {
        let map = self.latest_results.lock().await;
        map.get(&service_name).map(|res| serde_json::to_string(&res).unwrap())
    }
}
