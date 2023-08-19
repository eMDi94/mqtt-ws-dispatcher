# MQTT to Websocket dispatcher

The websocket clients can subscribe to the path ws://<server_address>/ws/<client_id>. Once subscribed, it is possible to send messages via the MQTT topic (right now supports only onw topic). The message is a JSON message with the following structure:

```
{
  "clientId": String,
  "message": String
}
```

Using the clientId, the message is then forwarded to the right websocket.