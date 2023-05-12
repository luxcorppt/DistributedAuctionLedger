use std::cmp::Ordering;
use serde_derive::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::mem;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use itertools::{EitherOrBoth, Itertools};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use crate::{ALPHA, BUCKET_REFRESH_INTERVAL, K, KadError, MAX_MESSAGE_BUFFER, REQUEST_TIMEOUT, Result, rpc};
use crate::buckets::KadBucket;
use crate::kadid::{bucket, KadID};
use crate::rpc::data::{KadFindValueResponse, KadMessage, KadMessagePayload, KadRequest, KadRequestFunction, KadResponse, KadResponseFunction};

#[derive(Serialize,Deserialize,Debug, PartialEq, Eq, Clone, Hash)]
pub struct NodeInfo {
    socket_addr: SocketAddr,
    id: KadID,
}

impl NodeInfo {
    pub fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    pub fn id(&self) -> &KadID {
        &self.id
    }
}

#[derive(Eq, Clone, Debug)]
struct LookupQElement {
    node: NodeInfo,
    distance: [u8; 20]
}

impl PartialEq for LookupQElement {
    fn eq(&self, other: &Self) -> bool {
        self.node.eq(&other.node)
    }
}

impl PartialOrd for LookupQElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.distance.partial_cmp(&self.distance)
    }
}

impl Ord for LookupQElement {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

pub enum LookupResult {
    Nodes(Vec<NodeInfo>),
    Value(Vec<u8>)
}

#[derive(Debug, Serialize,Deserialize)]
pub struct LocalNodeData {
    local_id: KadID,
    buckets: Vec<KadBucket>, // Vec aqui é ineficiente, mas é simples de implementar
    storage: HashMap<KadID, Vec<u8>>,
    // OPTIONAL: Implement Lookup caching
}

impl LocalNodeData {
    fn empty() -> Self {
        LocalNodeData {
            local_id: KadID::zeroes(),
            buckets: Vec::new(),
            storage: HashMap::new()
        }
    }

    fn update_node(&mut self, node_data: NodeInfo) {
        let distance = &self.local_id ^ node_data.id();
        let target_bucket = bucket(&distance);
        let target_bucket = &mut self.buckets[target_bucket as usize];
        target_bucket.update_node(node_data);
    }

    fn get_closest_nodes(&self, key: &KadID, count: usize) -> Vec<NodeInfo> {
        let index = bucket(key.as_ref()) as usize;
        let mut res = Vec::new();

        // the closest are in the bucket of the key
        res.extend_from_slice(self.buckets[index].nodes());

        if res.len() < count {
            // Design choice: pull from the two closest buckets above and below, always
            let iter_down = self.buckets[0..index].iter().rev();
            let iter_up = self.buckets[index+1..].iter();
            let iter = iter_down.zip_longest(iter_up);

            for v in iter {
                let tuple = match v {
                    EitherOrBoth::Both(a, b) => { (Some(a), Some(b)) }
                    EitherOrBoth::Left(a) => { (Some(a), None) }
                    EitherOrBoth::Right(b) => { (None, Some(b)) }
                };

                if let Some(a) = tuple.0 {
                    res.extend_from_slice(a.nodes());
                };
                if let Some(b) = tuple.1 {
                    res.extend_from_slice(b.nodes());
                };

                if res.len() >= count {
                    break;
                }
            }
        }

        res.sort_by_key(|info| info.id() ^ key);
        res.truncate(count);
        res
    }

    fn get_stale_indexes(&self) -> Vec<usize> {
        let mut res= Vec::new();
        for (i, b) in self.buckets.iter().enumerate() {
            if b.is_stale() {
                res.push(i);
            }
        }
        res
    }

    fn delete_node(&mut self, node: &NodeInfo) {
        let idx = bucket(&(&node.id ^ &self.local_id));
        self.buckets[idx as usize].delete_node(node);
    }
}

pub struct LocalNodeBuilder {
    local_socket: UdpSocket,
    data: LocalNodeData,
    bootstrap: bool,
    bootstrap_addr: Option<SocketAddr>
}

impl LocalNodeBuilder {
    pub fn start_empty(local_socket: UdpSocket) -> Self {
        let id = KadID::random();
        let mut b: Vec<_> = Default::default();
        b.resize_with(160, Default::default);
        let s = Default::default();

        LocalNodeBuilder {
            local_socket,
            data: LocalNodeData {
                local_id: id,
                buckets: b,
                storage: s,
            },
            bootstrap: true,
            bootstrap_addr: None
        }
    }

    pub fn start_from_data(data: &str, local_socket: UdpSocket) -> Result<Self> {
        Ok(LocalNodeBuilder {
            local_socket,
            data: serde_json::from_str(data).map_err(KadError::LocalDeserializeError)?,
            bootstrap: true,
            bootstrap_addr: None
        })
    }

    pub fn bootstrap(&mut self, val: bool) -> &mut Self {
        self.bootstrap = val;
        self
    }

    pub fn bootstrap_addr(&mut self, addr: SocketAddr) -> &mut Self {
        self.bootstrap_addr = Some(addr);
        self
    }

    pub async fn build(mut self) -> LocalNode {
        let local_info = NodeInfo {
            id: self.data.local_id.clone(),
            socket_addr: self.local_socket.local_addr().unwrap()
        };
        let tasks = Vec::new();
        let pending_reqs = Mutex::new(HashMap::new());
        let data = mem::replace(&mut self.data, LocalNodeData::empty());

        let result = Arc::new(LocalNodeInner {
            data: Mutex::new(data),
            local_info,
            local_socket: Arc::new(self.local_socket),
            tasks: Mutex::new(tasks),
            pending_reqs
        });

        let (message_tx, message_rx) = channel(MAX_MESSAGE_BUFFER);

        {
            let mut tasks = result.tasks.lock().unwrap();

            let h0 = tokio::spawn(rpc::exec::wait_message_recv(result.local_socket.clone(), message_tx));
            tasks.push(h0);
            let h1 = tokio::spawn(result.clone().message_handler(message_rx));
            tasks.push(h1);
            let h2 = tokio::spawn(result.clone().bucket_refresher());
            tasks.push(h2);
            if self.bootstrap {
                let h3 = tokio::spawn(result.clone().bootstrap_routing_table(self.bootstrap_addr));
                tasks.push(h3);
            }
        }

        LocalNode {
            inner: result
        }
    }
}

type PendingReqsMap = Mutex<HashMap<[u8;20],Sender<(KadResponse,NodeInfo)>>>;

pub struct LocalNode {
    inner: Arc<LocalNodeInner>
}

struct LocalNodeInner {
    local_info: NodeInfo,
    local_socket: Arc<UdpSocket>,
    data: Mutex<LocalNodeData>,
    tasks: Mutex<Vec<JoinHandle<()>>>,
    pending_reqs: PendingReqsMap
}

impl LocalNodeInner {
    async fn message_handler(self: Arc<Self>, mut rx: Receiver<KadMessage>) {
        loop {
            let message = match rx.recv().await {
                // if this closes, terminate
                None => {break;}
                Some(m) => m,
            };
            match message.payload {
                KadMessagePayload::Request(r) => {
                    self.clone().handle_request(r, message.sender).await;
                }
                KadMessagePayload::Response(r) => {
                    self.clone().handle_response(r, message.sender).await;
                }
            }
        }
    }

    async fn bucket_refresher(self: Arc<Self>) {
        loop {
            tokio::time::sleep(Duration::from_secs(BUCKET_REFRESH_INTERVAL as u64)).await;
            let stale_indexes = {
                self.data.lock().unwrap().get_stale_indexes()
            };

            for index in stale_indexes {
                // TODO: maybe do something with this
                let _ = tokio::spawn(Self::lookup_nodes_impl(
                    self.clone(),
                    KadID::random_in_range(&self.local_info.id, index),
                    false
                ));
            }
        }
    }

    async fn bootstrap_routing_table(self: Arc<Self>, bootstrap_addr: Option<SocketAddr>) {
        if let Some(addr) = bootstrap_addr {
            let target_info = NodeInfo {
                id: KadID::zeroes(),
                socket_addr: addr,
            };
            let (tx, mut rx) = channel(10);

            self.clone().do_rpc_request_impl(target_info, KadRequestFunction::Ping, tx).await;

            match tokio::time::timeout(Duration::from_secs(REQUEST_TIMEOUT as u64), rx.recv()).await {
                Ok(Some(_)) => {
                    // if we got here, then we succeeded to ping, nd have already updated our routing table
                }
                Ok(None) => {
                    eprintln!("Failed to bootstrap with {}. The channel closed.", addr);
                }
                Err(_) => {
                    eprintln!("Timed out when trying to bootstrap with {}. Continuing anyway.", addr);
                }
            }
        }

        // first, lookup ourselves
        let _ = Self::lookup_nodes_impl(
            self.clone(),
            self.local_info.id.clone(),
            false
        ).await;

        // then, refresh all buckets
        let count = {
            self.data.lock().unwrap().buckets.len()
        };
        for b in 0..count {
            Self::lookup_nodes_impl(
                self.clone(),
                KadID::random_in_range(&self.local_info.id, b),
                false
            ).await;
        }
    }

    async fn do_rpc_request_impl(
        self: Arc<Self>,
        target: NodeInfo,
        payload: KadRequestFunction,
        callback: Sender<Option<(KadResponse, NodeInfo)>>
    ) {
        let (response_send, mut response_recv) = channel(1);
        let mut uid = KadID::random();
        {
            let mut pending = self.pending_reqs.lock().unwrap();
            while pending.contains_key(uid.as_ref()) {
                uid = KadID::random();
            }
            pending.insert(*uid.as_ref(), response_send);
        }


        let msg = KadMessage {
            sender: self.local_info.clone(),
            payload: KadMessagePayload::Request(KadRequest {
                uid: *uid.as_ref(),
                function: payload,
            })
        };

        rpc::exec::send_message(&self.local_socket, &msg, &target).await.unwrap();

        let response_fut = response_recv.recv();
        match tokio::time::timeout(Duration::from_secs(REQUEST_TIMEOUT as u64), response_fut).await {
            Ok(Some(response)) => {
                match callback.send(Some(response)).await {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("Callback not available, ignoring...")
                    }
                }
            }
            Ok(None) => {
                panic!("Unexpected channel close.")
            }
            Err(_) => {
                eprintln!("A request timed out: {} did not respond", &target.socket_addr);
                // TODO: Maybe Return a Result instead
                match callback.send(None).await {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("Callback not available, ignoring...")
                    }
                }
                {
                    let mut node_data = self.data.lock().unwrap();
                    node_data.delete_node(&target);
                }
            }
        }
        {
            let mut pending = self.pending_reqs.lock().unwrap();
            pending.remove(uid.as_ref());
        }
    }

    // Since finding a node or finding a value are almost the same, EXCEPT for the RPC used,
    // we do it all here, switching the RPCs sent out according to what we're doing.
    async fn lookup_nodes_impl(
        self: Arc<Self>,
        key: KadID,
        get_value: bool
    ) -> LookupResult {
        // base step
        let start_list = self.data.lock().unwrap().get_closest_nodes(&key, ALPHA as usize);
        let mut distance_cutoff = match start_list.iter().map(|node| {
            &node.id ^ &key
        }).min() {
            None => {
                todo!("Handle lookup when no nodes to start")
            }
            Some(m) => m,
        };
        let (tx, mut rx) = channel((10 * ALPHA) as usize);
        let mut concurrency_count= 0;

        let mut query_queue: BinaryHeap<LookupQElement> = BinaryHeap::from(
            start_list.into_iter()
                .map(|node| LookupQElement {
                    distance: &node.id ^ &key,
                    node
                })
                .collect::<Vec<_>>()
        );
        let mut queried_nodes = HashSet::new();
        queried_nodes.insert(self.local_info.clone());
        let mut found_nodes = HashSet::new();
        found_nodes.insert(self.local_info.clone());

        // explicit iteration =  guarantees concurrency
        for _ in 0..ALPHA {
            Self::internal_lookup_nodes_spawn_rpc(
                self.clone(),
                &key,
                get_value,
                tx.clone(),
                &mut concurrency_count,
                &mut query_queue
            );
        }

        let mut terminate = false;
        while concurrency_count > 0 && !terminate {
            terminate = true;
            while concurrency_count < ALPHA && !query_queue.is_empty() {
                Self::internal_lookup_nodes_spawn_rpc(
                    self.clone(),
                    &key,
                    get_value,
                    tx.clone(),
                    &mut concurrency_count,
                    &mut query_queue
                );
            }

            let response = rx.recv().await.unwrap();
            concurrency_count -= 1;

            match response {
                None => {/* TODO: Do something when no response, [TRUST]*/}
                Some(response) => match response.0.function {
                    KadResponseFunction::Ping => {
                        // wtf why
                        eprintln!("Why did I get a ping??? 100% a bug");
                        terminate = false;
                    }
                    KadResponseFunction::FindNode { nodes } => {
                        // TODO: [TRUST] Find Node returned
                        Self::internal_onfindnode_stage1(
                            &key,
                            &mut distance_cutoff,
                            &mut query_queue,
                            &mut queried_nodes,
                            &mut found_nodes,
                            &mut terminate,
                            response.1,
                            nodes
                        );
                    }
                    KadResponseFunction::FindValue(KadFindValueResponse::Next(nodes)) => {
                        // TODO: [TRUST] Find Node Returned
                        Self::internal_onfindnode_stage1(
                            &key,
                            &mut distance_cutoff,
                            &mut query_queue,
                            &mut queried_nodes,
                            &mut found_nodes,
                            &mut terminate,
                            response.1,
                            nodes
                        );
                    }
                    KadResponseFunction::FindValue(KadFindValueResponse::Found(value)) => {
                        return LookupResult::Value(value);
                    }
                }
            }
        }

        // At this point, we can't really go lower with the distance, so we resort to exausting our search space
        // We need at least K nodes to fulfill our request, or as many as we can get
        while queried_nodes.len() < K as usize {
            while concurrency_count < ALPHA && !query_queue.is_empty() {
                Self::internal_lookup_nodes_spawn_rpc(
                    self.clone(),
                    &key,
                    get_value,
                    tx.clone(),
                    &mut concurrency_count,
                    &mut query_queue
                );
            }
            if concurrency_count == 0 { // ran out of stuff to do
                break;
            }

            let response = rx.recv().await.unwrap();
            concurrency_count -= 1;

            match response {
                None => {/* TODO: Do something when no response, [TRUST] */}
                Some(response) => match response.0.function {
                    KadResponseFunction::Ping => {
                        // wtf why
                        eprintln!("Why did I get a ping??? 100% a bug");
                        // however, do nothing if we do
                    }
                    KadResponseFunction::FindNode { nodes } => {
                        Self::internal_onfindnode_stage2(&key, &mut query_queue, &mut queried_nodes, &mut found_nodes, response.1, nodes);
                    }
                    KadResponseFunction::FindValue(KadFindValueResponse::Next(nodes)) => {
                        Self::internal_onfindnode_stage2(&key, &mut query_queue, &mut queried_nodes, &mut found_nodes, response.1, nodes);
                    }
                    KadResponseFunction::FindValue(KadFindValueResponse::Found(value)) => {
                        return LookupResult::Value(value);
                    }
                }
            }
        }

        LookupResult::Nodes(queried_nodes.into_iter()
            .sorted_by_key(|node| &node.id ^ &key)
            .take(K as usize)
            .collect::<Vec<_>>())
    }

    fn internal_onfindnode_stage2(key: &KadID, query_queue: &mut BinaryHeap<LookupQElement>, queried_nodes: &mut HashSet<NodeInfo>, found_nodes: &mut HashSet<NodeInfo>, node_info: NodeInfo, nodes: Vec<NodeInfo>) {
        queried_nodes.insert(node_info);
        for info in nodes {
            if !found_nodes.contains(&info) {
                found_nodes.insert(info.clone());
                query_queue.push(LookupQElement {
                    distance: &info.id ^ key,
                    node: info
                });
            }
        }
    }

    fn internal_onfindnode_stage1(key: &KadID, distance_cutoff: &mut [u8; 20], query_queue: &mut BinaryHeap<LookupQElement>, queried_nodes: &mut HashSet<NodeInfo>, found_nodes: &mut HashSet<NodeInfo>, terminate: &mut bool, node_info: NodeInfo, nodes: Vec<NodeInfo>) {
        queried_nodes.insert(node_info);
        for info in nodes {
            let info_distance = &info.id ^ key;

            if !found_nodes.contains(&info) {
                if info_distance < *distance_cutoff {
                    *distance_cutoff = info_distance;
                    *terminate = false;
                }

                found_nodes.insert(info.clone());
                query_queue.push(LookupQElement {
                    distance: info_distance,
                    node: info
                });
            }
        }
    }

    // SAFETY: REQUIRES ACTIVE RUNTIME
    fn internal_lookup_nodes_spawn_rpc(
        self: Arc<Self>,
        key: &KadID,
        get_value: bool,
        tx: Sender<Option<(KadResponse, NodeInfo)>>,
        concurrency_count: &mut u8,
        query_queue: &mut BinaryHeap<LookupQElement>
    ) {
        let request_fn =
            if get_value { KadRequestFunction::FindValue { key: key.clone() } } else { KadRequestFunction::FindNode { id: key.clone() } };

        let info = query_queue.pop().unwrap();

        // TODO: maybe do something with this handle eventually?
        let _ = tokio::spawn(self.do_rpc_request_impl(
            info.node,
            request_fn,
            tx
        ));

        *concurrency_count += 1;
    }

    async fn handle_request(self: Arc<Self>, request: KadRequest, origin: NodeInfo) {
        self.clone().update_routing_table(origin.clone());
        let response = match &request.function {
            KadRequestFunction::Ping => {
                KadResponse {
                    uid: request.uid,
                    function: KadResponseFunction::Ping,
                }
            }
            KadRequestFunction::Store { key, value } => {
                self.data.lock().unwrap().storage.insert(key.clone(), value.clone());
                KadResponse {
                    uid: request.uid,
                    function: KadResponseFunction::Ping,
                }
            }
            KadRequestFunction::FindNode { id } => {
                let nodes = self.data.lock().unwrap().get_closest_nodes(id, ALPHA as usize);
                KadResponse {
                    uid: request.uid,
                    function: KadResponseFunction::FindNode {
                        nodes,
                    }
                }
            }
            KadRequestFunction::FindValue { key } => {
                let data = self.data.lock().unwrap();
                let fv_result = if let Some(value) = data.storage.get(key) {
                    KadFindValueResponse::Found(value.clone())
                } else {
                    let nodes = data.get_closest_nodes(key, ALPHA as usize);
                    KadFindValueResponse::Next(nodes)
                };
                KadResponse {
                    uid: request.uid,
                    function: KadResponseFunction::FindValue(fv_result),
                }
            }
        };

        let message = KadMessage {
            sender: self.local_info.clone(),
            payload: KadMessagePayload::Response(response)
        };

        rpc::exec::send_message(&self.local_socket, &message, &origin).await.unwrap()
    }

    async fn handle_response(self: Arc<Self>, response: KadResponse, origin: NodeInfo) {
        self.clone().update_routing_table(origin.clone());
        let sender = {
            let pending = self.pending_reqs.lock().unwrap();
            pending.get(&response.uid).cloned()
        };
        if let Some(sender) = sender {
            sender.send((response,origin)).await.unwrap();
        }
    }

    fn update_routing_table(self: Arc<Self>, node_data: NodeInfo) {
        // let data = Arc::clone(&self.data);
        self.data.lock().unwrap().update_node(node_data);
    }

    async fn request_store(self: Arc<Self>, dest: NodeInfo, key: KadID, value: Vec<u8>) -> Option<(KadResponse, NodeInfo)> {
        let request = KadRequestFunction::Store {
            key,
            value,
        };
        let (tx, mut rx) = channel(1);
        // purposefully not calling tokio::spawn here, let the user of this call it themselves
        self.do_rpc_request_impl(dest, request, tx).await;
        match rx.recv().await {
            None => None,
            Some(x) => x,
        }
    }
}

impl LocalNode {
    pub fn get_id(&self) -> KadID {
        self.inner.local_info.id.clone()
    }

    pub fn get_location(&self) -> Result<SocketAddr> {
        Ok(self.inner.local_socket.local_addr()?)
    }

    pub fn print_buckets_summary(&self) -> Vec<String> {
        let mut result = Vec::new();
        let buckets = &self.inner.data.lock().unwrap().buckets;
        for (i,buck) in buckets.iter().enumerate() {
            let s = format!("Bucket {}: {}", i, buck.summary());
            result.push(s);
        }

        result
    }

    pub fn print_storage(&self) -> String {
        format!("{:?}", self.inner.data.lock().unwrap().storage)
    }

    pub async fn lookup(&self, key: KadID) -> Vec<NodeInfo> {
        match self.inner.clone().lookup_nodes_impl(key, false).await {
            LookupResult::Nodes(e) => {e}
            LookupResult::Value(_) => {
                unimplemented!()
            }
        }
    }

    pub async fn store(&self, key: KadID, value: Vec<u8>) {
        if let LookupResult::Nodes(nodes) = self.inner.clone().lookup_nodes_impl(key.clone(), false).await {
            for dest in nodes {
                tokio::spawn(self.inner.clone().request_store(dest, key.clone(), value.clone()));
            }
        }
    }

    pub async fn get(&self, key: KadID) -> Option<Vec<u8>> {
        if let LookupResult::Value(value) = self.inner.clone().lookup_nodes_impl(key, true).await {
            Some(value)
        } else {
            None
        }
    }

    pub async fn destroy_serialized(self) -> String {
        for task in self.inner.tasks.lock().unwrap().iter() {
            task.abort()
        }

        {
            let data = self.inner.data.lock().unwrap();
            serde_json::to_string_pretty(&*data).unwrap()
        }
    }
}
