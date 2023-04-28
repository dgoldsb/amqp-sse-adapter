use actix_web::rt::time::interval;
use actix_web_lab::sse::{self, ChannelStream, Sse};
use futures_util::future;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::{sync::Arc, time::Duration};

pub type RoutingKey = [char; 255];

pub struct Broadcaster {
    inner: Mutex<BroadcasterInner>,
}

#[derive(Clone, Debug, Default)]
/// Maintains a mapping from routine key to a sequence of open SSE connections.
struct BroadcasterInner {
    clients: HashMap<RoutingKey, Vec<sse::Sender>>,
}

impl Broadcaster {
    /// Constructs new broadcaster and spawns ping loop.
    pub fn create() -> Arc<Self> {
        let this = Arc::new(Broadcaster {
            inner: Mutex::new(BroadcasterInner::default()),
        });
        Broadcaster::spawn_ping(Arc::clone(&this));

        this
    }

    /// Pings clients every 10 seconds to see if they are alive and remove them from the broadcast list if not.
    fn spawn_ping(this: Arc<Self>) {
        actix_web::rt::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;
                this.remove_stale_clients().await;
            }
        });
    }

    /// Removes all non-responsive clients from broadcast list.
    async fn remove_stale_clients(&self) {
        let clients = &mut self.inner.lock().clients;
        for (key, value) in clients.clone().iter() {
            let ok_senders = self.remove_stale_clients_from_vector(value).await;
            clients.insert(*key, ok_senders);
        }
    }

    /// Removes all non-responsive clients from broadcast list.
    async fn remove_stale_clients_from_vector(
        &self,
        senders: &Vec<sse::Sender>,
    ) -> Vec<sse::Sender> {
        println!("active client {:?}", senders);

        let mut ok_senders = Vec::new();

        println!("okay active client {:?}", ok_senders);

        for sender in senders {
            if sender
                .send(sse::Event::Comment("ping".into()))
                .await
                .is_ok()
            {
                ok_senders.push(sender.clone());
            }
        }

        ok_senders
    }

    /// Registers client with broadcaster, returning an SSE response body.
    pub async fn new_client(&self, routing_key: &RoutingKey) -> Sse<ChannelStream> {
        println!("starting creation");
        let (tx, rx) = sse::channel(10);

        tx.send(sse::Data::new("connected")).await.unwrap();
        println!("creating new clients success {:?}", tx);

        let mut default_vec = Vec::new();

        self.inner
            .lock()
            .clients
            .get_mut(routing_key)
            .unwrap_or(&mut default_vec)
            .push(tx);

        if default_vec.len() != 0 {
            // TODO: Create key if not exists.
            self.inner.lock().clients.insert(*routing_key, default_vec);
        }
        rx
    }

    /// Broadcasts `msg` to all clients.
    pub async fn broadcast(&self, key: &RoutingKey, msg: &str) {
        let clients = self.inner.lock().clients.clone();

        let default_vec = Vec::new();

        let send_futures = clients
            .get(key)
            .unwrap_or(&default_vec)
            .iter()
            .map(|client| client.send(sse::Data::new(msg)));

        // try to send to all clients, ignoring failures
        // disconnected clients will get swept up by `remove_stale_clients`
        let _ = future::join_all(send_futures).await;
    }
}
