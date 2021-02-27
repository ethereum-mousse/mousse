#![allow(unused_must_use)]
use chrono::prelude::*;
use clap::{load_yaml, App};
use eth2_simulator::simulator::Simulator;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::convert::Infallible;
use std::convert::TryFrom;
use std::sync::Arc;
use std::time;
use tokio::sync::Mutex;
use warp::{http::StatusCode, reject, Filter};

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
    // $ RUST_LOG=trace cargo run
    pretty_env_logger::init();

    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    // TODO: Define default values
    // let mut slot_time: u64 = SECONDS_PER_SLOT;
    // let mut failure_rate: f32 = 1.0;
    let simulator = if let Some(mut vals) = matches.values_of("auto") {
        let slot_time = vals
            .next()
            .unwrap()
            .parse()
            .expect("SLOT_TIME must be `u64`.");
        let failure_rate = vals
            .next()
            .unwrap()
            .parse()
            .expect("FAILURE_RATE must be `f32`.");
        assert!(
            (0.0..=1.0).contains(&failure_rate),
            "FAILURE_RATE must be a positive float <= 1.0."
        );
        let shared_simulator = Arc::new(Mutex::new(Simulator::new()));
        let simulator = shared_simulator.clone();

        tokio::spawn(async move {
            process_auto(simulator, slot_time, failure_rate).await;
        });
        println!("Simulator started in auto mode.");
        shared_simulator
    } else {
        println!("Simulator started in manual mode.");
        Arc::new(Mutex::new(Simulator::new()))
    };

    // Run Eth2 simulator
    let request_logs = Arc::new(Mutex::new(Vec::<RequestLog>::new()));

    let routes = filters(simulator, request_logs)
        .recover(handle_rejection)
        .with(cors());

    let port = if let Some(port) = matches.value_of("port") {
        port.parse().expect("`port` must be a positive integer")
    } else {
        3030
    };

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

async fn process_auto(simulator: SharedSimulator, slot_time: u64, failure_rate: f32) {
    let slot_time = time::Duration::from_secs(slot_time);
    let start_time = time::Instant::now();
    loop {
        let mut simulator = simulator.lock().await;
        if time::Instant::now() < start_time + slot_time * u32::try_from(simulator.slot).unwrap() {
            continue;
        }
        let slot = simulator.slot;
        println!("Auto processing. Slot {}", slot);
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0..1.0) < failure_rate {
            // TODO: Remove happy case from `process_random`.
            simulator.process_slots_random(slot);
        } else {
            simulator.process_slots_happy(slot);
        };
    }
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
        .or(beacon_states(simulator.clone(), request_logs.clone()))
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
            simulator,
            request_logs.clone(),
        ))
        .or(utils_data_commitment(request_logs.clone()))
        .or(utils_request_logs(request_logs))
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
        .allow_credentials(true)
        .allow_headers(vec!["Content-Type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
}

fn log(request_logs: &mut tokio::sync::MutexGuard<Vec<RequestLog>>, endpoint: String) {
    let log_id = request_logs.len();
    request_logs.push(RequestLog {
        log_id,
        date: Local::now().to_string(),
        endpoint,
    });
}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_rejection(err: reject::Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(GivenSlotIsLowerThanAndEqualToCurrentSlot) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "BAD_REQUEST: The given slot <= the current slot.";
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
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
}

pub async fn get_beacon_blocks_head(
    simulator: SharedSimulator,
    // request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    // let mut request_logs = request_logs.lock().await;
    // log(&mut request_logs, String::from("GET /beacon/blocks/head"));
    let simulator = simulator.lock().await;
    let head = simulator.beacon_chain.blocks.last();
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

/// GET /beacon/states
pub fn beacon_states(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("beacon" / "states"))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(get_beacon_states)
}

pub async fn get_beacon_states(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(&mut request_logs, String::from("GET /beacon/states"));
    let simulator = simulator.lock().await;
    let beacon_states = simulator.beacon_chain.states.clone();
    Ok(warp::reply::json(&beacon_states))
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
    Ok(StatusCode::OK)
}

#[derive(Debug)]
struct GivenSlotIsLowerThanAndEqualToCurrentSlot;
impl reject::Reject for GivenSlotIsLowerThanAndEqualToCurrentSlot {}

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
}

pub async fn process_slots(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!("POST /simulator/slot/process/{}", slot),
    );
    let mut simulator = simulator.lock().await;
    if simulator.process_slots_happy(slot).is_err() {
        Err(reject::custom(GivenSlotIsLowerThanAndEqualToCurrentSlot))
    } else {
        Ok(StatusCode::OK)
    }
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
}

pub async fn process_slots_without_shard_data_inclusion(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_shard_data_inclusion/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    if simulator
        .process_slots_without_shard_data_inclusion(slot)
        .is_err()
    {
        Err(reject::custom(GivenSlotIsLowerThanAndEqualToCurrentSlot))
    } else {
        Ok(StatusCode::OK)
    }
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
}

pub async fn process_slots_without_shard_blob_proposal(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_shard_blob_proposal/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    if simulator
        .process_slots_without_shard_blob_proposal(slot)
        .is_err()
    {
        Err(reject::custom(GivenSlotIsLowerThanAndEqualToCurrentSlot))
    } else {
        Ok(StatusCode::OK)
    }
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
}

pub async fn process_slots_without_shard_header_inclusion(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_shard_header_inclusion/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    if simulator
        .process_slots_without_shard_header_inclusion(slot)
        .is_err()
    {
        Err(reject::custom(GivenSlotIsLowerThanAndEqualToCurrentSlot))
    } else {
        Ok(StatusCode::OK)
    }
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
}

pub async fn process_slots_without_shard_header_confirmation(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_shard_header_confirmation/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    if simulator
        .process_slots_without_shard_header_confirmation(slot)
        .is_err()
    {
        Err(reject::custom(GivenSlotIsLowerThanAndEqualToCurrentSlot))
    } else {
        Ok(StatusCode::OK)
    }
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
}

pub async fn process_slots_without_beacon_chain_finality(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_beacon_chain_finality/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    if simulator
        .process_slots_without_beacon_chain_finality(slot)
        .is_err()
    {
        Err(reject::custom(GivenSlotIsLowerThanAndEqualToCurrentSlot))
    } else {
        Ok(StatusCode::OK)
    }
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
}

pub async fn process_slots_without_beacon_block_proposal(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "POST /simulator/slot/process_without_beacon_block_proposal/{}",
            slot
        ),
    );
    let mut simulator = simulator.lock().await;
    if simulator
        .process_slots_without_beacon_block_proposal(slot)
        .is_err()
    {
        Err(reject::custom(GivenSlotIsLowerThanAndEqualToCurrentSlot))
    } else {
        Ok(StatusCode::OK)
    }
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
}

pub async fn process_slots_random(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!("POST /simulator/slot/process_random/{}", slot),
    );
    let mut simulator = simulator.lock().await;
    if simulator.process_slots_random(slot).is_err() {
        Err(reject::custom(GivenSlotIsLowerThanAndEqualToCurrentSlot))
    } else {
        Ok(StatusCode::OK)
    }
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
    let base64string = params.data.unwrap_or_default();
    let bytes = base64::decode(base64string).unwrap_or_default();
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
}

pub async fn get_utils_request_logs(
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let request_logs = request_logs.lock().await;
    let request_logs = request_logs.clone();
    Ok(warp::reply::json(&request_logs))
}
