# MQTT to Websocket dispatcher

The websocket clients can subscribe to the path ws://<server_address>/ws/<client_id>. Once subscribed, it is possible to send messages via the MQTT topic. The last part of the topic must be the wildcard # and it is used to identify the client_id of the websocket. The message must be a sequence of UTF-8 encoded string
