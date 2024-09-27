use crate::{
    error::MaelstromError::{self, EndOfInput, PoisonError, RWLockError, SerdeJsonError},
    message::{
        broadcast::{Handler as BroadcastHandler, Request, RequestBody, SyncBody},
        echo::Handler as EchoHandler,
        g_counter,
        g_counter::Handler as GcounterHandler,
        generate::Handler as GenerateHandler,
        init::Handler as InitHandler,
        send_request, Body, Message, MsgId, RequestTypes, WorkloadHandler,
    },
    server::router::{HandlerFn, HandlerMap, RouterLayer},
};
use futures::future::{ready, Ready};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    io::{stdout, BufRead, BufWriter, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, time};
use tower::{Service, ServiceBuilder};

#[derive(Clone, Debug)]
pub struct StdOutService<O>
where
    O: Write,
{
    inner: O,
}

impl<O: Write> StdOutService<O> {
    pub fn new(inner: O) -> Self {
        Self { inner }
    }
}

impl<O: Write> Service<String> for StdOutService<O> {
    type Response = String;
    type Error = MaelstromError;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, output_string: String) -> Self::Future {
        if output_string.is_empty() {
            return ready(Ok(output_string));
        }

        let output = &mut self.inner;
        let result = output.write(output_string.as_bytes());

        if let Err(e) = result {
            return ready(Err(e.into()));
        }
        let _ = output.write(b"\n");

        if let Err(e) = output.flush() {
            ready(Err(e.into()))
        } else {
            ready(Ok(output_string))
        }
    }
}

pub type SharedIoServerContext = Arc<RwLock<IoServerContext>>;

pub type NumericMessage = usize;
pub type MessageList = HashSet<NumericMessage>;
pub type NodeCounters = HashMap<String, usize>;
pub type NodeMessages = HashMap<String, MessageList>;

#[derive(Debug, Clone)]
pub struct IoServerContext {
    node_id: String,
    neighbors: Vec<String>,
    messages_queued: NodeMessages,
    messages_saved: MessageList,
    counter: Arc<AtomicUsize>,
    node_counters: NodeCounters,
    messages_neighbors_have: NodeMessages,
    message_type: RequestTypes,
    last_sync: Instant,
    msg_id: MsgId,
}

impl Default for IoServerContext {
    fn default() -> Self {
        Self {
            node_id: String::new(),
            neighbors: Vec::default(),
            messages_queued: HashMap::default(),
            messages_saved: HashSet::default(),
            counter: Arc::new(AtomicUsize::new(0)),
            node_counters: HashMap::default(),
            messages_neighbors_have: HashMap::default(),
            message_type: RequestTypes::default(),
            last_sync: Instant::now(),
            msg_id: 0,
        }
    }
}

pub trait BroadcastContext {
    fn add_message(&mut self, source: String, message: NumericMessage);
    #[must_use]
    fn messages(self) -> Vec<NumericMessage>;
}

impl BroadcastContext for IoServerContext {
    fn add_message(&mut self, source: String, message: NumericMessage) {
        self.messages_saved.insert(message);
        self.synced(source, HashSet::from([message]));
    }

    #[must_use]
    fn messages(self) -> Vec<NumericMessage> {
        let mut list: Vec<NumericMessage> = self.messages_saved.into_iter().collect();
        list.sort_unstable();
        list
    }
}
pub trait GCounterContext {
    fn add_node_counter(&mut self, node: String, counter: NumericMessage);
    fn cas_local_node_counter(&mut self, value: NumericMessage) -> usize;
    fn counter(&self) -> NumericMessage;
    fn max_node_counter(&mut self) -> usize;
    fn node_counters(&self) -> &NodeCounters;
    fn set_counter(&mut self, value: NumericMessage);
    fn update_all_node_counters(&mut self, new_counters: NodeCounters);
    fn update_local_node_counter(&mut self, delta: NumericMessage);
}

impl GCounterContext for IoServerContext {
    fn add_node_counter(&mut self, node: String, counter: NumericMessage) {
        self.node_counters
            .entry(node.clone())
            .and_modify(|v| *v += counter)
            .or_insert(counter);

        let max_counter = self.max_node_counter();
        self.cas_local_node_counter(counter.max(max_counter));
    }

    fn cas_local_node_counter(&mut self, value: NumericMessage) -> usize {
        self.counter.fetch_max(value, Ordering::SeqCst);
        self.counter()
    }

    #[must_use]
    fn counter(&self) -> NumericMessage {
        self.counter.load(Ordering::SeqCst)
    }

    fn max_node_counter(&mut self) -> usize {
        self.node_counters.values().sum::<usize>()
    }

    #[must_use]
    fn node_counters(&self) -> &NodeCounters {
        &self.node_counters
    }

    fn set_counter(&mut self, value: NumericMessage) {
        self.counter.store(value, Ordering::SeqCst);
    }

    fn update_all_node_counters(&mut self, new_counters: NodeCounters) {
        for (key, value) in new_counters {
            // Replace all node counter except self
            if &key != self.node() {
                *self.node_counters.entry(key.to_string()).or_default() = value;
            }
        }
    }

    fn update_local_node_counter(&mut self, delta: NumericMessage) {
        let _max = self.counter.fetch_add(delta, Ordering::SeqCst);
    }
}

impl IoServerContext {
    #[must_use]
    pub fn node(&self) -> &String {
        &self.node_id
    }

    pub fn set_node(&mut self, node: String) {
        self.node_id = node;
    }

    #[must_use]
    pub fn neighbors(&self) -> &Vec<String> {
        &self.neighbors
    }

    pub fn set_neighbors(&mut self, nodes: &[String]) {
        self.neighbors = Vec::from(nodes)
            .into_iter()
            .filter(|n| n != self.node())
            .collect();
    }

    pub fn message_type(&mut self, message_type: RequestTypes) {
        self.message_type = message_type;
    }

    pub fn synced(&mut self, node: String, messages: MessageList) -> &NodeMessages {
        self.messages_saved.extend(messages.clone());
        if self.neighbors.contains(&node) {
            self.messages_neighbors_have
                .entry(node)
                .or_default()
                .extend(messages);
        }
        let mut list = NodeMessages::new();
        for node in &self.neighbors {
            let messages_node_has = self
                .messages_neighbors_have
                .entry(node.clone())
                .or_default()
                .iter()
                .copied()
                .collect::<HashSet<_>>();
            let mut missing_messages = self
                .messages_saved
                .difference(&messages_node_has)
                .copied()
                .collect::<HashSet<_>>();

            let extra_known_messages_limit = (missing_messages.len() * 10 / 100).max(10);
            let extra_known_messages = messages_node_has
                .iter()
                .enumerate()
                .filter(|(count, _)| count < &extra_known_messages_limit)
                .map(|(_, message)| message)
                .copied()
                .collect::<HashSet<_>>();
            missing_messages.extend(extra_known_messages);
            list.insert(node.clone(), missing_messages);
        }
        for (node, messages) in list {
            self.queue_message_to_send(node, &messages);
        }
        &self.messages_queued
    }

    pub fn queue_message_to_send(&mut self, node: String, message: &MessageList) {
        if !message.is_empty() {
            self.messages_queued
                .entry(node)
                .or_default()
                .extend(message);
        }
    }

    pub fn next_msg_id(&mut self) -> MsgId {
        self.msg_id += 1;
        self.msg_id
    }
}

#[derive(Debug, Clone)]
pub struct IoServer<I, O>
where
    I: BufRead,
    O: Write,
{
    input: Arc<RwLock<I>>,
    output: Arc<Mutex<O>>,
    handlers: HandlerMap,
    context: SharedIoServerContext,
}

impl<I, O> IoServer<I, O>
where
    I: BufRead,
    O: Write,
{
    pub fn new(input: I, output: O) -> Self {
        let input = Arc::new(RwLock::new(input));
        let output = Arc::new(Mutex::new(output));
        let mut server = Self {
            input,
            output,
            handlers: HashMap::default(),
            context: Arc::default(),
        };
        server.register(RequestTypes::Init, InitHandler::response);
        server
    }

    pub async fn serve(&mut self) -> Result<(), MaelstromError> {
        let context = self.context.clone();
        tokio::spawn(async move {
            let mut retry_interval = time::interval(Duration::from_millis(125));
            let mut last_tick = Instant::now();
            loop {
                retry_interval.tick().await;
                if last_tick.elapsed() > Duration::from_secs(1) {
                    last_tick = Instant::now();
                    let _ = deliver_counters(&context).await;
                }
                let _ = retry_sync_messages(&context).await;
            }
        });

        loop {
            let context = self.context.clone();
            let handlers = self.handlers.clone();
            let input = self.input.clone();
            let output = self.output.clone();

            if let Err(EndOfInput) = main_loop(input, output, context, handlers.into()).await {
                break;
            }
        }

        Ok(())
    }

    pub fn register(&mut self, name: RequestTypes, handler: HandlerFn) -> &mut Self {
        HandlerMap::insert(&mut self.handlers, name, handler);
        self
    }
}

pub enum IoServerType {
    Echo,
    Broadcast,
    Gcounter,
    Generate,
    Init,
}
pub async fn start_io_server<I: BufRead, O: Write>(
    input: I,
    output: O,
    io_type: IoServerType,
) -> Result<(), MaelstromError> {
    let mut server = IoServer::new(input, output);
    let init = server.register(RequestTypes::Init, InitHandler::response);

    match io_type {
        IoServerType::Echo => server.register(RequestTypes::Echo, EchoHandler::response),
        IoServerType::Broadcast => server
            .register(RequestTypes::Broadcast, BroadcastHandler::response)
            .register(RequestTypes::Read, BroadcastHandler::response)
            .register(RequestTypes::Sync, BroadcastHandler::response)
            .register(RequestTypes::Topology, BroadcastHandler::response),
        IoServerType::Gcounter => server
            .register(RequestTypes::Add, GcounterHandler::response)
            .register(RequestTypes::Read, GcounterHandler::response)
            .register(RequestTypes::SyncCounter, GcounterHandler::response),
        IoServerType::Generate => {
            server.register(RequestTypes::Generate, GenerateHandler::response)
        }
        IoServerType::Init => init,
    }
    .serve()
    .await
}

async fn process_messages<O: Write>(
    context: SharedIoServerContext,
    handlers: Arc<HandlerMap>,
    req: &str,
    output: Arc<Mutex<O>>,
) -> Result<String, MaelstromError> {
    let mut output = output.lock().await;
    let router = RouterLayer::new(context, handlers);
    ServiceBuilder::new()
        .layer(router)
        .service(StdOutService::new(&mut *output))
        .call(req.to_string())
        .await
}

pub async fn send_message<W: Write>(
    writer: W,
    output_message: String,
) -> Result<String, MaelstromError> {
    let output = BufWriter::new(writer);
    ServiceBuilder::new()
        .service(StdOutService::new(output))
        .call(output_message)
        .await
}

async fn retry_sync_messages(context: &SharedIoServerContext) -> Result<usize, MaelstromError> {
    let remaining = context
        .read()
        .map_err(|e| PoisonError(e.to_string()))
        .map(|ctx| ctx.messages_queued.len())?;
    if remaining == 0 {
        return Ok(remaining);
    }
    sync_messages(context).await
}

async fn deliver_counters(context: &SharedIoServerContext) -> Result<(), MaelstromError> {
    let (neighbors, counters) = context
        .read()
        .map(|ctx| (ctx.neighbors().clone(), ctx.node_counters.clone()))
        .map_err(|e| PoisonError(e.to_string()))?;

    // Don't try delivering counter if there are none
    if counters.is_empty() {
        return Ok(());
    }

    for node in neighbors {
        let message = send_request(
            node,
            context,
            g_counter::RequestBody::SyncCounter(counters.clone().into()),
        );

        let sync_result = send_message(stdout(), message.serde_to_string()?).await;
        if sync_result.is_err() {
            // dbg!(&sync_result);
            sync_result?;
        }
    }
    Ok(())
}

async fn sync_messages(context: &SharedIoServerContext) -> Result<usize, MaelstromError> {
    let mut remaining: usize = 0;
    let sync_result = context
        .write()
        .map(|mut ctx| {
            let node = ctx.node_id.clone();
            let mut pending_messages = Vec::new();
            for (dest, messages) in ctx.messages_queued.drain() {
                let mut messages_to_send = Vec::from_iter(messages);
                messages_to_send.sort_unstable();
                if !messages_to_send.is_empty() {
                    let msg = (node.to_string(), dest.to_string(), messages_to_send);
                    pending_messages.push(msg);
                }
            }
            remaining = ctx.messages_queued.len();
            pending_messages
        })
        .map_err(|e| PoisonError(e.to_string()))?;

    let pending_messages = context
        .write()
        .map(|mut ctx| {
            let mut messages = Vec::new();
            for (node, dest, messages_to_send) in sync_result {
                let message = Request::new(Message::new(
                    node.to_string(),
                    dest.to_string(),
                    Body::new(
                        Some(ctx.next_msg_id()),
                        None,
                        RequestBody::Sync(SyncBody::new(messages_to_send)),
                    ),
                ));
                messages.push(message);
            }
            ctx.last_sync = Instant::now();
            messages
        })
        .map_err(|e| PoisonError(e.to_string()))?;

    for message in pending_messages {
        let sync_message = serde_json::to_string(&message).map_err(SerdeJsonError)?;
        let sync_result = send_message(stdout(), sync_message).await;
        if sync_result.is_err() {
            // dbg!(&sync_result);
            sync_result?;
        }
    }

    Ok(remaining)
}

async fn main_loop<I: BufRead, O: Write>(
    reader: Arc<RwLock<I>>,
    writer: Arc<Mutex<O>>,
    context: SharedIoServerContext,
    handlers: Arc<HandlerMap>,
) -> Result<(), MaelstromError> {
    let mut input = String::new();
    let input_result = reader
        .write()
        .map_err(|e| RWLockError(e.to_string()))?
        .read_line(&mut input);
    if let Ok(_size) = input_result {
        if input.is_empty() {
            return Err(EndOfInput);
        }

        let process_result = process_messages(context, handlers, &input, writer).await;
        match process_result {
            Ok(_response) => {}
            Err(e) => return Err(e),
        };
        input.clear();
    }
    Ok(())
}
