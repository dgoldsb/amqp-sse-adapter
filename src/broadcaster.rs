use crate::listener::{create_channel, CHANNEL};
use crate::routing_key::{MyString, RoutingKey};
use actix_web::rt::time::interval;
use actix_web_lab::__reexports::tracing::log;
use actix_web_lab::sse::{self, ChannelStream, Sse};
use amqprs::channel::{BasicCancelArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use futures_util::future;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::{sync::Arc, time::Duration};

#[derive(Clone, Debug)]
struct SenderTagPair {
    sender: sse::Sender,
    consumer_tag: String,
}

pub struct SseBroadcastingConsumer {
    inner: Mutex<SseBroadcastingConsumerInner>,
}

#[derive(Clone, Debug, Default)]
/// Maintains a mapping from routing key to a sequence of open SSE connections.
struct SseBroadcastingConsumerInner {
    routing_key_consumers_map: HashMap<RoutingKey, Vec<SenderTagPair>>,
}

/// An AMQP consumer that publishes the message body on an SSE sender.
/// The correct sender(s) are chosen based on the routing key.
impl SseBroadcastingConsumer {
    pub fn instance() -> &'static Arc<Self> {
        lazy_static! {
            static ref INSTANCE: Arc<SseBroadcastingConsumer> = SseBroadcastingConsumer::create();
        }

        &INSTANCE
    }

    /// Constructs new broadcaster and spawns ping loop.
    pub fn create() -> Arc<Self> {
        let this = Arc::new(SseBroadcastingConsumer {
            inner: Mutex::new(SseBroadcastingConsumerInner::default()),
        });
        SseBroadcastingConsumer::spawn_ping(Arc::clone(&this));

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
        let routing_key_consumers_map = &mut self.inner.lock().routing_key_consumers_map;
        for (key, value) in routing_key_consumers_map.clone().iter() {
            let ok_senders = self.remove_stale_clients_from_vector(value).await;
            routing_key_consumers_map.insert(*key, ok_senders);
        }
    }

    /// Removes all non-responsive clients from broadcast list.
    async fn remove_stale_clients_from_vector(
        &self,
        senders: &Vec<SenderTagPair>,
    ) -> Vec<SenderTagPair> {
        log::debug!("Active clients are {:?}", senders);

        let mut ok_senders = Vec::new();

        log::debug!("Okay active clients are {:?}", ok_senders);

        for pair in senders {
            if pair
                .sender
                .send(sse::Event::Comment("ping".into()))
                .await
                .is_ok()
            {
                ok_senders.push(pair.clone());
            } else {
                log::info!("Client timed out, consumer tag is {}", pair.consumer_tag);
                CHANNEL
                    .lock()
                    .get_or_init(|| async { create_channel().await })
                    .await
                    .basic_cancel(BasicCancelArguments::new(&pair.consumer_tag))
                    .await
                    .unwrap();
            }
        }

        ok_senders
    }

    /// Registers client with broadcaster, returning an SSE response body.
    pub async fn new_client(
        &self,
        routing_key: &RoutingKey,
        consumer_tag: &String,
    ) -> Sse<ChannelStream> {
        log::debug!("Starting creation of a new SSE client");
        let (sender, stream) = sse::channel(10);

        sender.send(sse::Data::new("connected")).await.unwrap();
        log::debug!("Creating new clients success {:?}", sender);

        let mut default_vec = Vec::new();
        let consumer_tag = consumer_tag.clone();
        self.inner
            .lock()
            .routing_key_consumers_map
            .get_mut(routing_key)
            .unwrap_or(&mut default_vec)
            .push(SenderTagPair {
                sender,
                consumer_tag,
            });

        if default_vec.len() != 0 {
            self.inner
                .lock()
                .routing_key_consumers_map
                .insert(*routing_key, default_vec);
        }
        stream
    }

    /// Broadcasts `msg` to all clients that fall under this routing key.
    pub async fn broadcast(&self, key: &RoutingKey, msg: &str) {
        let clients = self.inner.lock().routing_key_consumers_map.clone();

        let default_vec = Vec::new();

        let send_futures = clients
            .get(key)
            .unwrap_or(&default_vec)
            .iter()
            .map(|pair| pair.sender.send(sse::Data::new(msg)));

        // try to send to all clients, ignoring failures
        // disconnected clients will get swept up by `remove_stale_clients`
        let _ = future::join_all(send_futures).await;
    }
}

#[async_trait::async_trait]
impl AsyncConsumer for &SseBroadcastingConsumer {
    async fn consume(
        &mut self,
        _channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let body = String::from_utf8(content).unwrap();
        let routing_key: RoutingKey = MyString(deliver.routing_key().to_string())
            .try_into()
            .unwrap();
        self.broadcast(&routing_key, &body).await;
    }
}
