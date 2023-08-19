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
        match mqtt::extract_client_from_topic(&packet.topic) {
          Some(client_id) => {
            match String::from_utf8(packet.payload.as_ref().to_vec()) {
              Ok(message) => {
                info!("Received message to send to {}", client_id);
                match clients.lock().await.get(client_id) {
                  Some(client) => {
                    if let Some(sender) = &client.sender {
                      let _ = sender.send(Ok(warp::ws::Message::text(message)));
                    }
                  },
                  None => error!("No client found with client_id {}", &client_id)
                }
              },
              Err(e) => error!("Cannot convert message to UTF-8. {:?}", e)
            }
          }
          None => error!("Received mqtt packet without topic")
        }
      }
    }
  });
}
