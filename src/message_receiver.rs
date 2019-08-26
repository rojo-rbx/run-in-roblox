use std::{
    fmt,
    thread,
    time::Duration,
    sync::mpsc,
};

use serde_derive::Deserialize;
use futures::{
    Future,
    future,
    sync::oneshot,
    stream::Stream,
};

use hyper::{
    Body,
    Request,
    Response,
    StatusCode,
    Method,
    Server,
    service::service_fn,
};

type HyperResponse = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Stop,
    Messages(Vec<RobloxMessage>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum RobloxMessage {
    Output {
        level: OutputLevel,
        body: String,
    },
}

impl fmt::Display for RobloxMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RobloxMessage::Output { body, level } => {
                match level {
                    OutputLevel::Print => write!(f, "{}", body),
                    _ => write!(f, "[{}]: {}", level, body),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum OutputLevel {
    Info,
    Print,
    Warning,
    Error,
}

impl fmt::Display for OutputLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputLevel::Print => write!(f, "PRINT"),
            OutputLevel::Info => write!(f, "INFO"),
            OutputLevel::Warning => write!(f, "WARN"),
            OutputLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug)]
pub struct MessageReceiverOptions {
    pub port: u16,
}

pub struct MessageReceiver {
    shutdown_tx: oneshot::Sender<()>,
    message_rx: mpsc::Receiver<Message>,
}

impl MessageReceiver {
    pub fn start(options: MessageReceiverOptions) -> MessageReceiver {
        let (message_tx, message_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        thread::spawn(move || {
            let service = move || {
                let message_tx = message_tx.clone();

                service_fn(move |request: Request<Body>| -> HyperResponse {
                    let message_tx = message_tx.clone();
                    let mut response = Response::new(Body::empty());

                    match (request.method(), request.uri().path()) {
                        (&Method::GET, "/") => {
                            *response.body_mut() = Body::from("Hey there!");
                        },
                        (&Method::POST, "/start") => {
                            message_tx.send(Message::Start).unwrap();
                            *response.body_mut() = Body::from("Started");
                        },
                        (&Method::POST, "/stop") => {
                            message_tx.send(Message::Stop).unwrap();
                            *response.body_mut() = Body::from("Finished");
                        },
                        (&Method::POST, "/messages") => {
                            let message_tx = message_tx.clone();

                            let future = request
                                .into_body()
                                .concat2()
                                .map(move |chunk| {
                                    let source = chunk.to_vec();
                                    let messages: Vec<RobloxMessage> = serde_json::from_slice(&source)
                                        .expect("Failed deserializing message from Roblox Studio");

                                    message_tx.send(Message::Messages(messages)).unwrap();

                                    *response.body_mut() = Body::from("Got it!");
                                    response
                                });

                            return Box::new(future);
                        },
                        _ => {
                            *response.status_mut() = StatusCode::NOT_FOUND;
                        },
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