use std::{sync::Arc, collections::HashMap, convert::Infallible};

use futures_util::{StreamExt, FutureExt};
use log::{error, info};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{ws::{WebSocket, Message}, Filter, Rejection};

#[derive(Debug, Clone)]
pub struct WsClient {
  pub client_id: String,
  pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>
}

impl WsClient {
  pub fn new(client_id: String, sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>) -> Self {
    WsClient { client_id, sender }
  }
}

pub type WsClients = Arc<Mutex<HashMap<String, WsClient>>>;
pub type Result<T> = std::result::Result<T, Rejection>;

pub fn with_clients(clients: WsClients) -> impl Filter<Extract = (WsClients,), Error = Infallible> + Clone {
  warp::any().map(move || clients.clone())
}

pub async fn client_connection(ws: WebSocket, client_id: String, clients: WsClients) {
  let (client_ws_sender, mut client_ws_receiver) = ws.split();
  let (client_sender, client_receiver) = mpsc::unbounded_channel();

  let client_receiver = UnboundedReceiverStream::new(client_receiver);

  let _ = tokio::task::spawn(client_receiver.forward(client_ws_sender)).map(|result| {
    if let Err(e) = result {
      error!("Error while sending websocket message: {:?}", e);
    }
  });

  let new_client = WsClient::new(client_id.clone(), Some(client_sender));

  clients.lock().await.insert(client_id.clone(), new_client);

  while let Some(result) = client_ws_receiver.next().await {
    let msg = match result {
      Ok(msg) => msg,
      Err(e) => {
        error!("Error receiving message for id {}: {}", client_id.clone(), e);
        break;
      }
    };
    client_message(&client_id, msg, &clients).await;
  }

  clients.lock().await.remove(&client_id);
  info!("{} disconnected", &client_id);
}

async fn client_message(client_id: &str, msg: Message, clients: &WsClients) {
  info!("Received message from {}: {:?}", client_id, msg);

  let message = match msg.to_str() {
    Ok(v) => v,
    Err(_) => return
  };

  if message == "ping" || message == "ping\n" {
    let locked = clients.lock().await;
    match locked.get(client_id) {
      Some(client) => {
        if let Some(sender) = &client.sender {
          info!("Sending pong to {}", client_id);
          let _ = sender.send(Ok(Message::text("pong")));
        }
      },
      None => return
    }
  }
}
