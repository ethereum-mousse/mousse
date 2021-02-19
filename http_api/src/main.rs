#![allow(unused_must_use)]
use base64;
use chrono::prelude::*;
use eth2_simulator::simulator::Simulator;
use pretty_env_logger;
use serde_derive::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

pub use common::bid::Bid;
pub use common::eth2_config::*;
pub use common::eth2_types::*;

pub type SharedSimulator = Arc<Mutex<Simulator>>;

#[derive(Serialize, Clone)]
pub struct RequestLog {
    log_id: usize,
    date: String,
    endpoint: String,
    // request_body: String,
    // response_body: String,
}
pub type SharedRequestLogs = Arc<Mutex<Vec<RequestLog>>>;

#[tokio::main]
async fn main() {
    // Logging
    pretty_env_logger::init();

    // Run Eth2 simulator
    let simulator = Arc::new(Mutex::new(Simulator::new()));
    let request_logs = Arc::new(Mutex::new(Vec::<RequestLog>::new()));

    let routes = filters(simulator, request_logs);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

pub fn filters(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    root()
        .or(beacon_blocks(simulator.clone(), request_logs.clone()))
        .or(beacon_blocks_head(simulator.clone()))
        .or(beacon_finalized_blocks(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(beacon_finalized_checkpoint(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(data_market_bid(simulator.clone(), request_logs.clone()))
        .or(simulator_slot_process(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(simulator_slot_process_without_shard_data_inclusion(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(simulator_slot_process_without_shard_blob_proposal(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(simulator_slot_process_without_shard_header_inclusion(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(simulator_slot_process_without_shard_header_confirmation(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(simulator_slot_process_without_beacon_chain_finality(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(simulator_slot_process_without_beacon_block_proposal(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(simulator_slot_process_random(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(utils_data_commitment(request_logs.clone()))
        .or(utils_request_logs(request_logs.clone()))
}

fn with_simulator(
    simulator: SharedSimulator,
) -> impl Filter<Extract = (SharedSimulator,), Error = Infallible> + Clone {
    warp::any().map(move || simulator.clone())
}

fn with_request_logs(
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = (SharedRequestLogs,), Error = Infallible> + Clone {
    warp::any().map(move || request_logs.clone())
}

fn cors() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type"])
        .allow_methods(vec!["GET", "POST", "DELETE"])
}

fn log(request_logs: &mut tokio::sync::MutexGuard<Vec<RequestLog>>, endpoint: String) {
    let log_id = request_logs.len();
    request_logs.push(RequestLog {
        log_id,
        date: Local::now().to_string(),
        endpoint,
    });
}

/// GET /
/// For ping
pub fn root() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path::end().map(|| "root"))
}

/// GET /beacon/blocks
pub fn beacon_blocks(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("beacon" / "blocks"))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(get_beacon_blocks)
        .with(cors())
}

pub async fn get_beacon_blocks(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(&mut request_logs, String::from("GET /beacon/blocks"));
    let simulator = simulator.lock().await;
    let beacon_blocks = simulator.beacon_chain.blocks.clone();
    Ok(warp::reply::json(&beacon_blocks))
}

/// GET /beacon/blocks/head
pub fn beacon_blocks_head(
    simulator: SharedSimulator,
    // request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("beacon" / "blocks" / "head"))
        .and(with_simulator(simulator))
        // .and(with_request_logs(request_logs))
        .and_then(get_beacon_blocks_head)
        .with(cors())
}

pub async fn get_beacon_blocks_head(
    simulator: SharedSimulator,
    // request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    // let mut request_logs = request_logs.lock().await;
    // log(&mut request_logs, String::from("GET /beacon/blocks/head"));
    let simulator = simulator.lock().await;
    let head = simulator.beacon_chain.blocks.last().clone();
    Ok(warp::reply::json(&head))
}

/// GET /beacon/finalized_blocks
pub fn beacon_finalized_blocks(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("beacon" / "finalized_blocks"))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(get_beacon_finalized_blocks)
        .with(cors())
}

pub async fn get_beacon_finalized_blocks(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        String::from("GET /beacon/finalized_blocks"),
    );
    let simulator = simulator.lock().await;
    let beacon_blocks = simulator.beacon_chain.get_finalized_blocks();
    Ok(warp::reply::json(&beacon_blocks))
}

/// GET /beacon/finalized_checkpoint
pub fn beacon_finalized_checkpoint(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("beacon" / "finalized_checkpoint"))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(get_finalized_checkpoint)
        .with(cors())
}

pub async fn get_finalized_checkpoint(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        String::from("GET /beacon/finalized_checkpoint"),
    );
    let simulator = simulator.lock().await;
    let finalized_checkpoint = simulator.beacon_chain.finalized_checkpoint.clone();
    Ok(warp::reply::json(&finalized_checkpoint))
}

/// POST /data_market/bid
/// $ curl -X POST -d '{"shard":0,"slot":0,"commitment":{"point":1337,"length":0},"fee":0}' -H 'Content-Type: application/json' http://localhost:3030/data_market/bid
pub fn data_market_bid(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("data_market" / "bid"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(publish_bid)
        .with(cors())
}

pub async fn publish_bid(
    bid: Bid,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(&mut request_logs, String::from("POST /data_market/bid"));
    let mut simulator = simulator.lock().await;
    simulator.publish_bid(bid.clone());
    Ok(warp::reply::json(&bid))
}

/// POST /simulator/slot/process/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process/1
pub fn simulator_slot_process(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("simulator" / "slot" / "process" / Slot))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(process_slots)
        .with(cors())
}

pub async fn process_slots(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!("POST /simulator/slot/process/{}", slot),
    );
    let mut simulator = simulator.lock().await;
    simulator.process_slots_happy(slot);
    Ok(warp::reply::json(&slot))
}

/// POST /simulator/slot/process_without_shard_data_inclusion/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_shard_data_inclusion/1
pub fn simulator_slot_process_without_shard_data_inclusion(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_shard_data_inclusion" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(process_slots_without_shard_data_inclusion)
        .with(cors())
}

pub async fn process_slots_without_shard_data_inclusion(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_shard_data_inclusion/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    simulator.process_slots_without_shard_data_inclusion(slot);
    Ok(warp::reply::json(&slot))
}

/// POST /simulator/slot/process_without_shard_blob_proposal/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_shard_blob_proposal/1
pub fn simulator_slot_process_without_shard_blob_proposal(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_shard_blob_proposal" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(process_slots_without_shard_blob_proposal)
        .with(cors())
}

pub async fn process_slots_without_shard_blob_proposal(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_shard_blob_proposal/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    simulator.process_slots_without_shard_blob_proposal(slot);
    Ok(warp::reply::json(&slot))
}

/// POST /simulator/slot/process_without_shard_header_inclusion/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_shard_header_inclusion/1
pub fn simulator_slot_process_without_shard_header_inclusion(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_shard_header_inclusion" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(process_slots_without_shard_header_inclusion)
        .with(cors())
}

pub async fn process_slots_without_shard_header_inclusion(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_shard_header_inclusion/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    simulator.process_slots_without_shard_header_inclusion(slot);
    Ok(warp::reply::json(&slot))
}

/// POST /simulator/slot/process_without_shard_header_confirmation/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_shard_header_confirmation/1
pub fn simulator_slot_process_without_shard_header_confirmation(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_shard_header_confirmation" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(process_slots_without_shard_header_inclusion)
        .with(cors())
}

pub async fn process_slots_without_shard_header_confirmation(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_shard_header_confirmation/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    simulator.process_slots_without_shard_header_confirmation(slot);
    Ok(warp::reply::json(&slot))
}

/// POST /simulator/slot/process_without_beacon_chain_finality/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_beacon_chain_finality/1
pub fn simulator_slot_process_without_beacon_chain_finality(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_beacon_chain_finality" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(process_slots_without_beacon_chain_finality)
        .with(cors())
}

pub async fn process_slots_without_beacon_chain_finality(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_beacon_chain_finality/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    simulator.process_slots_without_beacon_chain_finality(slot);
    Ok(warp::reply::json(&slot))
}

/// POST /simulator/slot/process_without_beacon_block_proposal/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_beacon_block_proposal/1
pub fn simulator_slot_process_without_beacon_block_proposal(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_beacon_block_proposal" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(process_slots_without_beacon_block_proposal)
        .with(cors())
}

pub async fn process_slots_without_beacon_block_proposal(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_beacon_block_proposal/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    simulator.process_slots_without_beacon_block_proposal(slot);
    Ok(warp::reply::json(&slot))
}

/// POST /simulator/slot/process_random/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_random/1
pub fn simulator_slot_process_random(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("simulator" / "slot" / "process_random" / Slot))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(process_slots_random)
        .with(cors())
}

pub async fn process_slots_random(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!("POST /simulator/slot/process_random/{}", slot),
    );
    let mut simulator = simulator.lock().await;
    simulator.process_slots_random(slot);
    Ok(warp::reply::json(&slot))
}

/// GET /utils/data_commitment
pub fn utils_data_commitment(
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("utils" / "data_commitment"))
        .and(warp::query::<UtilsDataCommitmentParams>())
        .and(with_request_logs(request_logs))
        .and_then(get_utils_data_commitment)
        .with(cors())
}

#[derive(Deserialize)]
pub struct UtilsDataCommitmentParams {
    data: Option<String>,
}

pub async fn get_utils_data_commitment(
    params: UtilsDataCommitmentParams,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        String::from("GET /utils/data_commitment"),
    );
    let base64string = params.data.unwrap_or(String::new());
    let bytes = base64::decode(base64string).unwrap_or(vec![]);
    let dummy = DataCommitment::dummy_from_bytes(&bytes);
    Ok(warp::reply::json(&dummy))
}

/// GET /utils/request_logs
pub fn utils_request_logs(
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("utils" / "request_logs"))
        .and(with_request_logs(request_logs))
        .and_then(get_utils_request_logs)
        .with(cors())
}

pub async fn get_utils_request_logs(
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let request_logs = request_logs.lock().await;
    let request_logs = request_logs.clone();
    Ok(warp::reply::json(&request_logs))
}
