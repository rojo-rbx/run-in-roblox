use std::{
    thread,
    time::Duration,
    sync::mpsc,
};

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
    Message(Vec<u8>),
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
                        (&Method::POST, "/message") => {
                            let message_tx = message_tx.clone();

                            let future = request
                                .into_body()
                                .concat2()
                                .map(move |chunk| {
                                    message_tx.send(Message::Message(chunk.to_vec())).unwrap();

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