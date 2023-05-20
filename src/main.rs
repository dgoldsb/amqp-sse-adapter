use actix_web::Responder;
use actix_web::{web, App, HttpServer};
mod broadcaster;
mod listener;
mod routing_key;
use self::broadcaster::SseBroadcastingConsumer;
use self::listener::Listener;
use self::routing_key::{MyString, RoutingKey};
use actix_web_lab::__reexports::tracing::log;
use actix_web_lab::extract::Path;
use std::{io, sync::Arc};

pub struct AppState {
    broadcaster: Arc<SseBroadcastingConsumer>,
    listener: Arc<Listener>,
}

/// Establish a connection between an SSE sender and and AMQP queue.
pub async fn establish_amqp_sse_bridge(
    state: web::Data<AppState>,
    Path((routing_key_string,)): Path<(String,)>,
) -> impl Responder {
    log::info!(
        "Accepted request for an SSE stream for routing key {}",
        routing_key_string
    );

    // Parse the routing key from the API path.
    let routing_key: RoutingKey = MyString(routing_key_string).try_into().unwrap();

    // Create a new listener, or look up a cached one.
    log::info!("Creating a queue...");
    let queue_name: String = state.listener.create_queue(&routing_key).await;
    log::info!("Generated the queue name {}", queue_name);
    let consumer_tag = state
        .listener
        .add_callback(&queue_name, SseBroadcastingConsumer::instance())
        .await;
    log::info!("Connected a consumer with consumer tag {}", consumer_tag);

    // Create and register a new SSE stream.
    let channel_stream = state
        .broadcaster
        .new_client(&routing_key, &consumer_tag)
        .await;
    log::debug!("Connected a channel stream ({:?})", channel_stream);

    channel_stream
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    log::info!("Starting the main application...");
    let broadcaster = SseBroadcastingConsumer::instance();
    log::debug!("Created the broadcaster");
    let listener: Arc<Listener> = Listener::create().await;
    log::debug!("Created the listener");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                broadcaster: Arc::clone(&broadcaster),
                listener: Arc::clone(&listener),
            }))
            // This route is used to listen to events over SSE.
            .route(
                "/events/{routing_key}",
                web::get().to(establish_amqp_sse_bridge),
            )
    })
    .bind(format!("{}:{}", "0.0.0.0", "8000"))?
    .run()
    .await
}
