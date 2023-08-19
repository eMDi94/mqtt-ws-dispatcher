use crate::ws;
use crate::mqtt;

use rumqttc::{EventLoop, Event, Incoming};
use log::{info, error};
use warp::Reply;

pub async fn ws_handler(client_id: String, ws: warp::ws::Ws, clients: ws::WsClients) -> ws::Result<impl Reply> {
  info!("Calling ws handler for client with id {}", &client_id);

  Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, client_id, clients)))
}

pub fn eventloop_handler(clients: ws::WsClients, mut eventloop: EventLoop) {
  tokio::spawn(async move {
    while let Ok(event) = eventloop.poll().await {
      if let Event::Incoming(Incoming::Publish(packet)) = event {
        info!("Received incoming event");
        match mqtt::MqttMessage::<String>::try_from(packet.payload.as_ref()) {
          Ok(message) => {
            info!("Received message {:?}", message);
            match clients.lock().await.get(&message.client_id) {
              Some(client) => {
                if let Some(sender) = &client.sender {
                  let _ = sender.send(Ok(warp::ws::Message::text(message.message)));
                }
              },
              None => info!("No client found with client_id {}", &message.client_id)
            }
          },
          Err(e) => error!("Error while decoding the mqtt message {:?}", e)
        }
      }
    }
  });
}
