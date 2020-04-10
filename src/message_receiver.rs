use std::{
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use futures::{future, stream::Stream, sync::oneshot, Future};
use hyper::{service::service_fn, Body, Method, Request, Response, Server, StatusCode};
use serde::Deserialize;

type HyperResponse = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Stop,
    Messages(Vec<RobloxMessage>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum RobloxMessage {
    Output { level: OutputLevel, body: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum OutputLevel {
    Print,
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct MessageReceiverOptions {
    pub port: u16,
    pub server_id: String,
}

pub struct MessageReceiver {
    shutdown_tx: oneshot::Sender<()>,
    message_rx: mpsc::Receiver<Message>,
}

impl MessageReceiver {
    pub fn start(options: MessageReceiverOptions) -> MessageReceiver {
        let (message_tx, message_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let server_id = Arc::new(options.server_id.clone());

        thread::spawn(move || {
            let service = move || {
                let server_id = server_id.clone();
                let message_tx = message_tx.clone();

                service_fn(move |request: Request<Body>| -> HyperResponse {
                    let server_id = server_id.clone();
                    let message_tx = message_tx.clone();
                    let mut response = Response::new(Body::empty());

                    log::debug!("Request: {} {}", request.method(), request.uri().path());

                    match (request.method(), request.uri().path()) {
                        (&Method::GET, "/") => {
                            *response.body_mut() = Body::from(server_id.as_str().to_owned());
                        }
                        (&Method::POST, "/start") => {
                            message_tx.send(Message::Start).unwrap();
                            *response.body_mut() = Body::from("Started");
                        }
                        (&Method::POST, "/stop") => {
                            message_tx.send(Message::Stop).unwrap();
                            *response.body_mut() = Body::from("Finished");
                        }
                        (&Method::POST, "/messages") => {
                            let message_tx = message_tx.clone();

                            let future = request.into_body().concat2().map(move |chunk| {
                                let source = chunk.to_vec();
                                let messages: Vec<RobloxMessage> = serde_json::from_slice(&source)
                                    .expect("Failed deserializing message from Roblox Studio");

                                message_tx.send(Message::Messages(messages)).unwrap();

                                *response.body_mut() = Body::from("Got it!");
                                response
                            });

                            return Box::new(future);
                        }
                        _ => {
                            *response.status_mut() = StatusCode::NOT_FOUND;
                        }
                    }

                    Box::new(future::ok(response))
                })
            };

            let addr = ([127, 0, 0, 1], options.port).into();
            let server = Server::bind(&addr)
                .serve(service)
                .with_graceful_shutdown(shutdown_rx)
                .map_err(|e| eprintln!("server error: {}", e));

            hyper::rt::run(server);
        });

        MessageReceiver {
            shutdown_tx,
            message_rx,
        }
    }

    pub fn recv(&self) -> Message {
        self.message_rx.recv().unwrap()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Option<Message> {
        self.message_rx.recv_timeout(timeout).ok()
    }

    pub fn stop(self) {
        let _dont_care = self.shutdown_tx.send(());
    }
}
