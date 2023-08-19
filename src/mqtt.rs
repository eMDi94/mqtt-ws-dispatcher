use std::{time::Duration, env::VarError};
use std::env;

use rumqttc::{MqttOptions, AsyncClient, EventLoop};
use anyhow::Result;

pub fn create_mqtt_client_and_eventloop() -> Result<(AsyncClient, EventLoop)> {
  let client_name = env::var("MQTT_CLIENT_NAME")
    .map_err(|_| anyhow::Error::msg("MQTT_CLIENT_NAME must be set"))?;
  let mqtt_broker_host = env::var("MQTT_BROKER_HOST")
    .map_err(|_| anyhow::Error::msg("MQTT_BROKER_HOST must be set"))?;
  let mqtt_broker_port = env::var("MQTT_BROKER_PORT")
    .map_err(|_| anyhow::Error::msg("MQTT_BROKER_PORT must be set"))
    .and_then(|port| port.parse::<u16>().map_err(|_| anyhow::Error::msg("MQTT_BROKER_PORT unparsable")))?;
  let mqtt_keep_alive = env::var("MQTT_KEEP_ALIVE")
    .and_then(|keep| keep.parse::<u64>().map_err(|_| VarError::NotPresent))
    .unwrap_or(5);
  let mqtt_channel_capacity = env::var("MQTT_CHANNEL_CAPACITY")
    .and_then(|keep| keep.parse::<usize>().map_err(|_| VarError::NotPresent))
    .unwrap_or(10);


  let mut options = MqttOptions::new(client_name, mqtt_broker_host, mqtt_broker_port);
  options.set_keep_alive(Duration::from_secs(mqtt_keep_alive));

  Ok(AsyncClient::new(options, mqtt_channel_capacity))
}

pub fn extract_client_from_topic(topic: &str) -> Option<&str> {
  topic.split("/").last()
}
