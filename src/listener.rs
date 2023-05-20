use crate::broadcaster::SseBroadcastingConsumer;
use crate::routing_key::{MyString, RoutingKey};
use actix_web_lab::__reexports::tracing::log;
use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicConsumeArguments, Channel, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use rand::{distributions::Alphanumeric, Rng};
use std::env;
use std::sync::Arc;
use tokio::sync::OnceCell;

pub struct Listener {}

lazy_static! {
    pub static ref AMQP_CONNECTION: Mutex<OnceCell<Connection>> = Mutex::new(OnceCell::new());
}

/// Create a connection connected to the AMQP exchange.
pub async fn create_connection() -> Connection {
    // Open a connection to RabbitMQ server.
    let connection = Connection::open(&OpenConnectionArguments::new(
        &env::var("HOST").unwrap(),
        env::var("PORT").unwrap().parse::<u16>().unwrap(),
        &env::var("USERNAME").unwrap(),
        &env::var("PASSWORD").unwrap(),
    ))
    .await
    .unwrap();
    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();
    log::debug!("Opened AMQP connection");
    connection
}

lazy_static! {
    pub static ref CHANNEL: Mutex<OnceCell<Channel>> = Mutex::new(OnceCell::new());
}

/// Create a channel connected to the AMQP exchange.
pub async fn create_channel() -> Channel {
    // Open a channel on the connection.
    let channel = AMQP_CONNECTION
        .lock()
        .get_or_init(|| async { create_connection().await })
        .await
        .open_channel(None)
        .await
        .unwrap();
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();
    log::debug!("Opened AMQP channel");
    channel
}

impl Listener {
    pub async fn create() -> Arc<Listener> {
        let _ = CHANNEL
            .lock()
            .get_or_init(|| async { create_channel().await })
            .await;
        Arc::new(Listener {})
    }

    /// Create a queue, if it does not yet exist.
    pub async fn create_queue(&self, routing_key: &RoutingKey) -> String {
        // Unpack the routing key by stripping from spaces.
        let routing_key_my_string: MyString = (*routing_key).try_into().unwrap();
        let routing_key_string: String = routing_key_my_string.0;
        log::debug!("Creating a queue for routing key {}", routing_key_string);

        let exchange_name = &env::var("EXCHANGE").unwrap();
        log::debug!("Connecting on exchange {}", exchange_name);

        // Generate a random string to add to the queue name.
        let random_string: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        log::debug!("Generated random string {}", random_string);

        // Declare a transient queue.
        let queue_name = format!("sse.{}.{}", routing_key_string, random_string);
        log::debug!("Generated queue name {}", queue_name);
        let (queue_name, _, _) = CHANNEL
            .lock()
            .get_or_init(|| async { create_channel().await })
            .await
            .queue_declare(QueueDeclareArguments::transient_autodelete(
                queue_name.as_str(),
            ))
            .await
            .unwrap()
            .unwrap();

        // Bind the queue to exchange.
        log::debug!(
            "Binding queue name {} to {}",
            queue_name,
            routing_key_string
        );
        CHANNEL
            .lock()
            .get_or_init(|| async { create_channel().await })
            .await
            .queue_bind(QueueBindArguments::new(
                &queue_name,
                exchange_name,
                &routing_key_string,
            ))
            .await
            .unwrap();
        log::info!("Connected {} to {}", queue_name, routing_key_string);
        queue_name
    }

    /// Add a callback, all callbacks are called when a message on the matching queue is received.
    pub async fn add_callback(
        &self,
        queue_name: &str,
        consumer: &'static SseBroadcastingConsumer,
    ) -> String {
        // Generate a consumer tag.
        let random_string: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        // Start a consumer.
        let args = BasicConsumeArguments::new(&queue_name, &random_string);
        let consumer_tag = CHANNEL
            .lock()
            .get_or_init(|| async { create_channel().await })
            .await
            .basic_consume(consumer, args)
            .await
            .unwrap();

        // Return the consumer tag, so that the caller can (eventually) cancel the consumer.
        consumer_tag
    }
}
