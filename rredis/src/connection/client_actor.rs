use crate::connection::Connection;
use rredis_wire::protocol::error::{NetworkErrorKind, RRError};
use rredis_wire::protocol::{Data, Frame, NetworkFrame, Request};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::select;

pub trait ClientActor {
    fn get_handle(&self) -> impl ActorHandle + Clone;
    fn run(self) -> tokio::task::JoinHandle<()>;
}

pub trait ActorHandle {
    fn message(
        &mut self,
        frame: Frame<impl Into<String> + Send>,
    ) -> impl Future<Output=NetworkFrame> + Send;
    fn register<T: Into<String> + Send>(
        &mut self,
        keys: impl Into<Vec<T>> + Send,
        handler: impl Fn(Data, Option<Data>) + Send + Sync + 'static,
        label: &'static str,
        initial_trigger: bool,
    ) -> impl Future<Output=Result<(), RRError>> + Send;
}

/// transports the response of a request back to the handle that sent it;
/// errors travel as [`Request::Error`] frames, a dropped sink means the actor is gone
pub type ResponseSink = tokio::sync::oneshot::Sender<NetworkFrame>;

pub enum HandlerRequest {
    AddListener(Vec<String>, Arc<dyn Fn(Data, Option<Data>) + Send + Sync>, &'static str, bool),
    RemoveListener(&'static str),
    SendReceive(NetworkFrame, Option<ResponseSink>),
}

pub struct Listener {
    pub label: &'static str,
    pub handler: Arc<dyn Fn(Data, Option<Data>) + Send + Sync>,
    pub initial_trigger: bool,
}

pub struct DefaultClientActor {
    connection: Connection,
    listener_map: HashMap<String, Vec<Listener>>,
    tx: tokio::sync::mpsc::Sender<HandlerRequest>,
    rx: tokio::sync::mpsc::Receiver<HandlerRequest>,
}

#[derive(Clone)]
pub struct DefaultHandle {
    actor_tx: tokio::sync::mpsc::Sender<HandlerRequest>,
}

impl ActorHandle for DefaultHandle {
    async fn message(&mut self, frame: Frame<impl Into<String> + Send>) -> NetworkFrame {
        let (tx, rx) = tokio::sync::oneshot::channel();
        if self
            .actor_tx
            .send(HandlerRequest::SendReceive(frame.into_network(), Some(tx)))
            .await
            .is_err()
        {
            return Frame::new_error_request(actor_gone_error(), None);
        }
        match rx.await {
            Ok(response) => response,
            Err(_) => Frame::new_error_request(actor_gone_error(), None),
        }
    }

    async fn register<T: Into<String> + Send>(
        &mut self,
        keys: impl Into<Vec<T>> + Send,
        handler: impl Fn(Data, Option<Data>) + Send + Sync + 'static,
        label: &'static str,
        initial_trigger: bool,
    ) -> Result<(), RRError> {
        let keys = keys.into().into_iter().map(Into::into).collect();
        self.actor_tx
            .send(HandlerRequest::AddListener(keys, Arc::new(handler), label, initial_trigger))
            .await
            .map_err(|_| actor_gone_error())
    }
}

/// the actor task is gone, so its mpsc receiver / oneshot sender got dropped
fn actor_gone_error() -> RRError {
    RRError::new(
        NetworkErrorKind::ConnectionFailed.into(),
        Some("client actor is not running".into()),
    )
}

impl ClientActor for DefaultClientActor {
    fn get_handle(&self) -> impl ActorHandle + Clone {
        DefaultHandle {
            actor_tx: self.tx.clone(),
        }
    }

    fn run(self) -> tokio::task::JoinHandle<()> {
        let DefaultClientActor {
            mut rx,
            mut tx,
            mut listener_map,
            mut connection,
        } = self;

        let mut sinks: HashMap<u64, ResponseSink> = HashMap::new();

        tokio::spawn(async move {
            loop {
                select! {
                    res = rx.recv() => {
                        match res {
                            Some(request) => handle_request(request, &mut listener_map, &mut sinks, &mut tx, &mut connection).await,
                            None => break,
                        }
                    }

                    res = connection.read_frame() => {
                        match res {
                            Ok(request) => {
                                let (req, payload, id) = request.clone().decompose();
                                if let Some(handler) = sinks.remove(&id){
                                    let _ = handler.send(request);
                                }
                                else if let Request::Data {key, value} = req{
                                    for (listeners_key,handlers) in listener_map.iter_mut(){
                                        if key[..] == listeners_key[..]{
                                            for h in handlers{
                                                if h.initial_trigger{
                                                    (*h.handler)(value.clone(),payload.clone());
                                                    h.initial_trigger = false
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            Err(_) => break,
                        }
                    }
                }
            }
        })
    }
}

/// processes one [`HandlerRequest`] coming from a handle
async fn handle_request(
    request: HandlerRequest,
    listener_map: &mut HashMap<String, Vec<Listener>>,
    sinks: &mut HashMap<u64, ResponseSink>,
    tx: &mut tokio::sync::mpsc::Sender<HandlerRequest>,
    connection: &mut Connection,
) {
    match request {
        HandlerRequest::AddListener(keys, handler, label, initial_trigger) => {
            for key in keys {
                listener_map.entry(key.clone()).or_default().push(Listener {
                    label,
                    handler: Arc::clone(&handler),
                    initial_trigger,
                });

                if initial_trigger {
                    let _ = tx.send(HandlerRequest::SendReceive(
                        Frame::new_get_request(key, None),
                        None,
                    )).await;
                }
            }
        }

        HandlerRequest::RemoveListener(label) => {
            listener_map.retain(|_, listeners| {
                listeners.retain(|l| l.label != label);
                !listeners.is_empty()
            });
        }

        HandlerRequest::SendReceive(frame, sink) => {
            let id = frame.get_id();
            if connection.write_frame(frame).await.is_ok() {
                if let Some(sink) = sink {
                    sinks.insert(id, sink);
                }
            }
        }
    }
}
impl DefaultClientActor {
    pub fn new(connection: Connection) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(12);
        Self {
            connection,
            tx,
            rx,
            listener_map: HashMap::new(),
        }
    }
}
