mod ws;
mod handlers;
mod mqtt;

use std::net::SocketAddr;
use std::{sync::Arc, collections::HashMap};
use std::env;

use futures_util::Future;
use log::info;
use rumqttc::{AsyncClient, QoS};
use tokio::sync::Mutex;
use warp::Filter;
use ws::WsClients;


#[tokio::main]
async fn main() {
  init_log_and_environment();
  info!("Starting the application...");

  let clients = Arc::new(Mutex::new(HashMap::new()));
  let server_future = create_server(&clients);
  
  let (mqtt_client, eventloop) = mqtt::create_mqtt_client_and_eventloop().unwrap();
  subscribe_to_topic(&mqtt_client).await;

  info!("Starting mqtt event handler...");
  handlers::eventloop_handler(clients.clone(), eventloop);

  info!("Starting server...");
  server_future.await;
}

fn init_log_and_environment() {
  if let Err(e) = log4rs::init_file("config/log4rs.yml", Default::default()) {
    panic!("Cannot initialize log4rs logger because {}. Exit...", e);
  }

  if let Err(e) = dotenv::dotenv() {
    panic!("Cannot initialize environment variables because {}. Exit...", e);
  }
}

fn create_server(clients: &WsClients) -> impl Future<Output = ()> {
  let server_address: SocketAddr = match env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:3030".into()).parse() {
    Ok(addr) => addr,
    Err(e) => {
      panic!("Cannot parse SERVER_ADDRESS environment variable: {}. Exit...", e);
    }
  };

  let ws_routes = warp::path!("ws" / String)
    .and(warp::ws())
    .and(ws::with_clients(clients.clone()))
    .and_then(handlers::ws_handler);
  let routes = ws_routes.with(warp::cors().allow_any_origin());

  warp::serve(routes).run(server_address)
}

async fn subscribe_to_topic(mqtt_client: &AsyncClient) {
  let topic = match env::var("MQTT_TOPIC") {
    Ok(t) => t,
    Err(_) => panic!("MQTT_TOPIC must be set")
  };
  let _ = mqtt_client.subscribe(&topic, QoS::ExactlyOnce).await;
}
