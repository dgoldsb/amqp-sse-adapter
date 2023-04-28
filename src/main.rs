use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::{web, App, HttpServer};
mod broadcaster;
use self::broadcaster::{Broadcaster, RoutingKey};
use actix_web_lab::extract::Path;
use std::{io, sync::Arc};

pub struct AppState {
    broadcaster: Arc<Broadcaster>,
}

// SSE
pub async fn sse_client(
    state: web::Data<AppState>,
    Path((routing_key_string,)): Path<(String,)>,
) -> impl Responder {
    let routing_key: RoutingKey = format!("{: >255}", routing_key_string)
        .chars()
        .collect::<Vec<char>>()
        .try_into()
        .unwrap();
    state.broadcaster.new_client(&routing_key).await
}

pub async fn broadcast_msg(
    state: web::Data<AppState>,
    Path((routing_key_string,)): Path<(String,)>,
    Path((msg,)): Path<(String,)>,
) -> impl Responder {
    // TODO: Replace unwrap, cover this in testing for a good response code.
    let routing_key: RoutingKey = format!("{: >255}", routing_key_string)
        .chars()
        .collect::<Vec<char>>()
        .try_into()
        .unwrap();
    state.broadcaster.broadcast(&routing_key, &msg).await;
    HttpResponse::Ok().body("msg sent")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let broadcaster = Broadcaster::create();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                broadcaster: Arc::clone(&broadcaster),
            }))
            // This route is used to listen to events/ sse events
            .route("/events/{routing_key}{_:/?}", web::get().to(sse_client))
            // This route will create a notification
            // TODO: Replace with RabbitMQ, refactor after Dockerizing and adding component test.
            .route("/events/{routing_key}/{msg}", web::get().to(broadcast_msg))
    })
    .bind(format!("{}:{}", "127.0.0.1", "8000"))?
    .run()
    .await
}
