//! WebSocket communication with browser instances.
//!
//! This module handles the low-level communication with browser instances
//! using the WebSocket protocol.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;
use log::{debug, error, info, warn};

/// Errors that can occur during protocol communication.
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("WebSocket connection error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("Request timeout")]
    Timeout,
    
    #[error("Channel closed")]
    ChannelClosed,
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Error response: {0}")]
    ErrorResponse(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Request message sent to the browser.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// Response message received from the browser.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<ResponseError>,
}

/// Error details in a response.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseError {
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Event message received from the browser.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// Command for the connection worker.
enum ConnectionCommand {
    SendRequest {
        request: Request,
        response_sender: oneshot::Sender<Result<Response, ProtocolError>>,
    },
    Subscribe {
        method: String,
        event_sender: mpsc::Sender<Event>,
    },
    Unsubscribe {
        method: String,
        id: usize,
    },
    Close,
}

type PendingRequests = HashMap<String, oneshot::Sender<Result<Response, ProtocolError>>>;
type EventSubscribers = HashMap<String, Vec<(usize, mpsc::Sender<Event>)>>;

/// A connection to a browser instance.
pub struct Connection {
    command_sender: mpsc::Sender<ConnectionCommand>,
    next_subscriber_id: Arc<Mutex<usize>>,
}

impl Connection {
    /// Creates a new connection to a browser instance.
    pub async fn connect(url: &str) -> Result<Self, ProtocolError> {
        let (ws_stream, _) = connect_async(url).await?;
        
        let (command_sender, command_receiver) = mpsc::channel(100);
        let next_subscriber_id = Arc::new(Mutex::new(0));
        
        // Spawn a worker task to handle the connection
        tokio::spawn(Self::connection_worker(ws_stream, command_receiver));
        
        Ok(Self {
            command_sender,
            next_subscriber_id,
        })
    }
    
    /// Sends a request to the browser and waits for a response.
    pub async fn send_request(
        &self,
        method: String,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, ProtocolError> {
        let id = Uuid::new_v4().to_string();
        let request = Request {
            id: id.clone(),
            method,
            params,
        };
        
        let (response_sender, response_receiver) = oneshot::channel();
        
        self.command_sender
            .send(ConnectionCommand::SendRequest {
                request,
                response_sender,
            })
            .await
            .map_err(|_| ProtocolError::ChannelClosed)?;
        
        let response = response_receiver.await.map_err(|_| ProtocolError::ChannelClosed)?;
        let response = response?;
        
        if let Some(error) = response.error {
            return Err(ProtocolError::ErrorResponse(error.message));
        }
        
        response.result.ok_or_else(|| ProtocolError::Unknown("No result in response".to_string()))
    }
    
    /// Subscribes to events of a specific type.
    pub async fn subscribe(&self, method: String) -> Result<mpsc::Receiver<Event>, ProtocolError> {
        let (event_sender, event_receiver) = mpsc::channel(100);
        
        self.command_sender
            .send(ConnectionCommand::Subscribe {
                method,
                event_sender,
            })
            .await
            .map_err(|_| ProtocolError::ChannelClosed)?;
        
        Ok(event_receiver)
    }
    
    /// Unsubscribes from events of a specific type.
    pub async fn unsubscribe(&self, method: String, id: usize) -> Result<(), ProtocolError> {
        self.command_sender
            .send(ConnectionCommand::Unsubscribe { method, id })
            .await
            .map_err(|_| ProtocolError::ChannelClosed)?;
        
        Ok(())
    }
    
    /// Closes the connection.
    pub async fn close(&self) -> Result<(), ProtocolError> {
        self.command_sender
            .send(ConnectionCommand::Close)
            .await
            .map_err(|_| ProtocolError::ChannelClosed)?;
        
        Ok(())
    }
    
    async fn connection_worker(
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        command_receiver: mpsc::Receiver<ConnectionCommand>,
    ) {
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let event_subscribers = Arc::new(Mutex::new(HashMap::<String, Vec<(usize, mpsc::Sender<Event>)>>::new()));
        
        let (ws_send_queue_tx, mut ws_send_queue_rx) = mpsc::channel::<Message>(100);
        
        // Task for sending messages
        let sender_pending_requests = Arc::clone(&pending_requests);
        tokio::spawn(async move {
            while let Some(message) = ws_send_queue_rx.recv().await {
                if let Err(e) = ws_sender.send(message).await {
                    error!("Failed to send WebSocket message: {}", e);
                    
                    // Notify all pending requests of the error
                    let mut pending = sender_pending_requests.lock().unwrap();
                    for (_, sender) in pending.drain() {
                        let _ = sender.send(Err(ProtocolError::WebSocketError(e.clone())));
                    }
                    
                    break;
                }
            }
        });
        
        // Task for receiving messages
        let receiver_pending_requests = Arc::clone(&pending_requests);
        let receiver_event_subscribers = Arc::clone(&event_subscribers);
        tokio::spawn(async move {
            while let Some(message_result) = ws_receiver.next().await {
                match message_result {
                    Ok(message) => {
                        if let Message::Text(text) = message {
                            debug!("Received WebSocket message: {}", text);
                            
                            // Try to parse as a response
                            if let Ok(response) = serde_json::from_str::<Response>(&text) {
                                let mut pending = receiver_pending_requests.lock().unwrap();
                                if let Some(sender) = pending.remove(&response.id) {
                                    let _ = sender.send(Ok(response));
                                }
                            } 
                            // Try to parse as an event
                            else if let Ok(event) = serde_json::from_str::<Event>(&text) {
                                let subscribers = receiver_event_subscribers.lock().unwrap();
                                if let Some(senders) = subscribers.get(&event.method) {
                                    for (_, sender) in senders {
                                        let _ = sender.try_send(event.clone());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("WebSocket receive error: {}", e);
                        
                        // Notify all pending requests of the error
                        let mut pending = receiver_pending_requests.lock().unwrap();
                        for (_, sender) in pending.drain() {
                            let _ = sender.send(Err(ProtocolError::WebSocketError(e.clone())));
                        }
                        
                        break;
                    }
                }
            }
        });
        
        // Main command processing loop
        let mut command_queue = VecDeque::new();
        let mut command_fut = command_receiver.recv();
        
        loop {
            tokio::select! {
                command = &mut command_fut, if command_queue.is_empty() => {
                    match command {
                        Some(cmd) => {
                            command_queue.push_back(cmd);
                            command_fut = command_receiver.recv();
                        }
                        None => break,
                    }
                }
                _ = async {}, if !command_queue.is_empty() => {
                    match command_queue.pop_front().unwrap() {
                        ConnectionCommand::SendRequest { request, response_sender } => {
                            debug!("Sending request: {:?}", request);
                            let message = serde_json::to_string(&request).unwrap();
                            
                            // Store the response sender before sending the message
                            {
                                let mut pending = pending_requests.lock().unwrap();
                                pending.insert(request.id.clone(), response_sender);
                            }
                            
                            if let Err(e) = ws_send_queue_tx.send(Message::Text(message)).await {
                                error!("Failed to queue WebSocket message: {}", e);
                                
                                // Remove the request from pending
                                let mut pending = pending_requests.lock().unwrap();
                                if let Some(sender) = pending.remove(&request.id) {
                                    let _ = sender.send(Err(ProtocolError::ChannelClosed));
                                }
                            }
                        }
                        ConnectionCommand::Subscribe { method, event_sender } => {
                            let id = {
                                let mut next_id = event_subscribers.lock().unwrap().len();
                                let mut subscribers = event_subscribers.lock().unwrap();
                                let entry = subscribers.entry(method.clone()).or_default();
                                entry.push((next_id, event_sender));
                                next_id
                            };
                            
                            debug!("Subscribed to event: {} with id: {}", method, id);
                        }
                        ConnectionCommand::Unsubscribe { method, id } => {
                            let mut subscribers = event_subscribers.lock().unwrap();
                            if let Some(senders) = subscribers.get_mut(&method) {
                                senders.retain(|(sid, _)| *sid != id);
                                if senders.is_empty() {
                                    subscribers.remove(&method);
                                }
                            }
                            
                            debug!("Unsubscribed from event: {} with id: {}", method, id);
                        }
                        ConnectionCommand::Close => {
                            break;
                        }
                    }
                }
            }
        }
        
        // Cleanup
        let _ = ws_send_queue_tx.send(Message::Close(None)).await;
        info!("Connection closed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;
    use std::net::SocketAddr;
    use std::str::FromStr;
    
    async fn setup_test_server() -> SocketAddr {
        let addr = SocketAddr::from_str("127.0.0.1:0").unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let ws_stream = accept_async(stream).await.unwrap();
                let (mut sender, mut receiver) = ws_stream.split();
                
                // Echo server with response formatting
                tokio::spawn(async move {
                    while let Some(Ok(msg)) = receiver.next().await {
                        if let Message::Text(text) = msg {
                            if let Ok(req) = serde_json::from_str::<Request>(&text) {
                                // Create a response
                                let resp = Response {
                                    id: req.id,
                                    result: Some(serde_json::json!({"status": "ok"})),
                                    error: None,
                                };
                                
                                let resp_text = serde_json::to_string(&resp).unwrap();
                                let _ = sender.send(Message::Text(resp_text)).await;
                                
                                // Send a test event
                                let event = Event {
                                    method: "test.event".to_string(),
                                    params: Some(serde_json::json!({"type": "test"})),
                                };
                                
                                let event_text = serde_json::to_string(&event).unwrap();
                                let _ = sender.send(Message::Text(event_text)).await;
                            }
                        }
                    }
                });
            }
        });
        
        addr
    }
    
    #[tokio::test]
    async fn test_connection() {
        let addr = setup_test_server().await;
        let url = format!("ws://{}", addr);
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let conn = Connection::connect(&url).await.unwrap();
        
        // Test sending a request
        let result = conn.send_request(
            "test.method".to_string(),
            Some(serde_json::json!({"param": "value"})),
        ).await.unwrap();
        
        assert_eq!(result["status"], "ok");
        
        // Test subscribing to events
        let mut event_receiver = conn.subscribe("test.event".to_string()).await.unwrap();
        
        // Send another request to trigger an event
        let _ = conn.send_request(
            "another.method".to_string(),
            None,
        ).await.unwrap();
        
        // Wait for the event
        let event = timeout(Duration::from_secs(1), event_receiver.recv()).await.unwrap().unwrap();
        assert_eq!(event.method, "test.event");
        assert_eq!(event.params.unwrap()["type"], "test");
        
        // Close the connection
        conn.close().await.unwrap();
    }
} 