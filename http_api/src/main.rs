#![allow(unused_must_use)]
use chrono::prelude::*;
use clap::{load_yaml, App};
use eth2_simulator::simulator;
use eth2_simulator::simulator::Simulator;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::{thread, time};
use thiserror::Error;
use tokio::sync::Mutex;
use warp::{http::StatusCode, reject, Filter};

pub use common::bid::Bid;
pub use common::eth2_config::*;
pub use common::eth2_types::*;

pub type SharedSimulator = Arc<Mutex<Simulator>>;

/// Config for the auto mode.
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    auto: bool,
    /// Slot time in seconds.
    slot_time: u64,
    /// Failure rate in the simulation.
    failure_rate: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto: false,
            slot_time: SECONDS_PER_SLOT,
            failure_rate: 0.0,
        }
    }
}
/// Extended config for the auto mode.
#[derive(Clone)]
pub struct ExtendedConfig {
    config: Config,
    /// The time when the auto mode started.
    start_time: time::Instant,
    /// The number of slots processed after the auto mode started.
    processed_slot: u32,
}

impl Default for ExtendedConfig {
    fn default() -> Self {
        Self {
            config: Config::default(),
            start_time: time::Instant::now(),
            processed_slot: 0,
        }
    }
}

impl ExtendedConfig {
    fn restart_auto(&mut self) {
        self.start_time = time::Instant::now();
        self.processed_slot = 0;
    }
}

pub type SharedConfig = Arc<Mutex<ExtendedConfig>>;

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

    let mut config = ExtendedConfig::default();
    if matches.is_present("auto") {
        config.config.auto = true;

        if let Some(val) = matches.value_of("slot-time") {
            config.config.slot_time = val.parse().expect("SLOT_TIME must be `u64`.");
        }

        if let Some(val) = matches.value_of("failure-rate") {
            let failure_rate = val.parse().expect("FAILURE_RATE must be `f32`.");
            assert!(
                (0.0..=1.0).contains(&failure_rate),
                "FAILURE_RATE must be a positive float <= 1.0."
            );
            config.config.failure_rate = failure_rate;
        }
        println!("Simulator started in auto mode.");
    } else {
        println!("Simulator started in manual mode.");
    };

    let mut simulator = Simulator::new();
    // Process the genesis slot.
    simulator.process_slots_happy(0);
    println!("Slot 0 is automatically processed.");

    let shared_simulator = Arc::new(Mutex::new(simulator));
    let shared_config = Arc::new(Mutex::new(config));

    let simulator = shared_simulator.clone();
    let config = shared_config.clone();
    tokio::spawn(async move {
        process_auto(simulator, config).await;
    });

    let shared_request_logs = Arc::new(Mutex::new(Vec::<RequestLog>::new()));

    let routes = filters(shared_simulator, shared_request_logs, shared_config)
        .recover(handle_rejection)
        .with(cors());

    let port = if let Some(port) = matches.value_of("port") {
        port.parse().expect("`port` must be a positive integer")
    } else {
        3030
    };

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

async fn process_auto(simulator: SharedSimulator, config: SharedConfig) {
    let ten_millis = time::Duration::from_millis(10);
    loop {
        let mut config = config.lock().await;
        let next_slot_time = config.start_time
            + time::Duration::from_secs(config.config.slot_time) * config.processed_slot;
        if !config.config.auto || time::Instant::now() < next_slot_time {
            // Wait 0.01 seconds.
            thread::sleep(ten_millis);
            continue;
        }
        let mut simulator = simulator.lock().await;
        let slot = simulator.slot;
        println!("Auto processing. Slot {}", slot);
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0..1.0) < config.config.failure_rate {
            // TODO: Remove happy case from `process_random`.
            simulator.process_slots_random(slot);
        } else {
            simulator.process_slots_happy(slot);
        };
        config.processed_slot += 1;
    }
}

pub fn filters(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    root()
        .or(beacon_blocks(simulator.clone(), request_logs.clone()))
        .or(beacon_blocks_head(simulator.clone(), request_logs.clone()))
        .or(beacon_finalized_blocks(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(beacon_states(simulator.clone(), request_logs.clone()))
        .or(beacon_finalized_checkpoint(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(shards_bid(simulator.clone(), request_logs.clone()))
        .or(shards_bid_with_data(
            simulator.clone(),
            request_logs.clone(),
        ))
        .or(config_get(request_logs.clone(), config.clone()))
        .or(config_set(request_logs.clone(), config.clone()))
        .or(simulator_init(
            simulator.clone(),
            request_logs.clone(),
            config.clone(),
        ))
        .or(simulator_slot_process(
            simulator.clone(),
            request_logs.clone(),
            config.clone(),
        ))
        .or(simulator_slot_process_without_shard_data_inclusion(
            simulator.clone(),
            request_logs.clone(),
            config.clone(),
        ))
        .or(simulator_slot_process_without_shard_blob_proposal(
            simulator.clone(),
            request_logs.clone(),
            config.clone(),
        ))
        .or(simulator_slot_process_without_shard_header_inclusion(
            simulator.clone(),
            request_logs.clone(),
            config.clone(),
        ))
        .or(simulator_slot_process_without_shard_header_confirmation(
            simulator.clone(),
            request_logs.clone(),
            config.clone(),
        ))
        .or(simulator_slot_process_without_beacon_chain_finality(
            simulator.clone(),
            request_logs.clone(),
            config.clone(),
        ))
        .or(simulator_slot_process_without_beacon_block_proposal(
            simulator.clone(),
            request_logs.clone(),
            config.clone(),
        ))
        .or(simulator_slot_process_random(
            simulator.clone(),
            request_logs.clone(),
            config.clone(),
        ))
        .or(utils_current_status_for_polling(simulator, config))
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

fn with_config(
    config: SharedConfig,
) -> impl Filter<Extract = (SharedConfig,), Error = Infallible> + Clone {
    warp::any().map(move || config.clone())
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
        message = "NOT_FOUND".into();
    } else if let Some(e) = err.find::<SlotProcessingError>() {
        code = StatusCode::BAD_REQUEST;
        message = format!("BAD_REQUEST: {:?}", e);
    } else if let Some(e) = err.find::<BidPublicationError>() {
        code = StatusCode::BAD_REQUEST;
        message = format!("BAD_REQUEST: {:?}", e);
    } else if let Some(e) = err.find::<ConfigSetError>() {
        code = StatusCode::BAD_REQUEST;
        message = format!("BAD_REQUEST: {:?}", e);
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".into();
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION".into();
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}

#[derive(Debug)]
pub struct SlotProcessingError(pub simulator::SlotProcessingError);

impl reject::Reject for SlotProcessingError {}

pub fn slot_processing_error(e: simulator::SlotProcessingError) -> reject::Rejection {
    warp::reject::custom(SlotProcessingError(e))
}

#[derive(Debug)]
pub struct BidPublicationError(pub simulator::BidPublicationError);

impl reject::Reject for BidPublicationError {}

pub fn bid_publication_error(e: simulator::BidPublicationError) -> reject::Rejection {
    reject::custom(BidPublicationError(e))
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failure rate must be a positive integer <= 1.0 (found {found:?})")]
    InvalidFailureRate { found: f32 },
}
#[derive(Debug)]
pub struct ConfigSetError(pub ConfigError);

impl reject::Reject for ConfigSetError {}

pub fn config_set_error(e: ConfigError) -> reject::Rejection {
    reject::custom(ConfigSetError(e))
}

/// GET /
/// For ping
pub fn root() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path::end().map(|| "root"))
}

#[derive(Serialize, Deserialize)]
pub struct CountAndPageParams {
    count: Option<Slot>,
    page: Option<usize>,
}

/// GET /beacon/blocks
pub fn beacon_blocks(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("beacon" / "blocks"))
        .and(warp::query::<CountAndPageParams>())
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(get_beacon_blocks)
}

pub async fn get_beacon_blocks(
    params: CountAndPageParams,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "GET /beacon/blocks?{}",
            serde_qs::to_string(&params).unwrap()
        ),
    );
    let simulator = simulator.lock().await;
    let count = params.count.unwrap_or(100);
    let beacon_blocks = if simulator.beacon_chain.blocks.len() < count as usize {
        simulator.beacon_chain.blocks.clone()
    } else {
        let page = params.page.unwrap_or(0);
        let last_slot = simulator.beacon_chain.blocks.last().unwrap().slot;
        simulator
            .beacon_chain
            .blocks
            .iter()
            .filter(|block| {
                last_slot < block.slot + count * (page + 1) as Slot
                    && block.slot + count * page as Slot <= last_slot
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    Ok(warp::reply::json(&beacon_blocks))
}

/// GET /beacon/blocks/head
pub fn beacon_blocks_head(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("beacon" / "blocks" / "head"))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(get_beacon_blocks_head)
}

pub async fn get_beacon_blocks_head(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(&mut request_logs, String::from("GET /beacon/blocks/head"));
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
        .and(warp::query::<CountAndPageParams>())
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(get_beacon_finalized_blocks)
}

pub async fn get_beacon_finalized_blocks(
    params: CountAndPageParams,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "GET /beacon/finalized_blocks?{}",
            serde_qs::to_string(&params).unwrap()
        ),
    );
    let simulator = simulator.lock().await;
    let count = params.count.unwrap_or(100);
    let beacon_blocks = simulator.beacon_chain.get_finalized_blocks();
    let beacon_blocks = if beacon_blocks.len() < count as usize {
        beacon_blocks.clone()
    } else {
        let page = params.page.unwrap_or(0);
        let last_slot = beacon_blocks.last().unwrap().slot;
        beacon_blocks
            .iter()
            .filter(|block| {
                last_slot < block.slot + count * (page + 1) as Slot
                    && block.slot + count * page as Slot <= last_slot
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    Ok(warp::reply::json(&beacon_blocks))
}

/// GET /beacon/states
pub fn beacon_states(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("beacon" / "states"))
        .and(warp::query::<CountAndPageParams>())
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(get_beacon_states)
}

pub async fn get_beacon_states(
    params: CountAndPageParams,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!(
            "GET /beacon/states?{}",
            serde_qs::to_string(&params).unwrap()
        ),
    );
    let simulator = simulator.lock().await;
    let count = params.count.unwrap_or(100);
    let beacon_states = if simulator.beacon_chain.states.len() < count as usize {
        simulator.beacon_chain.states.clone()
    } else {
        let page = params.page.unwrap_or(0);
        let last_slot = simulator.beacon_chain.states.last().unwrap().slot;
        simulator
            .beacon_chain
            .states
            .iter()
            .filter(|state| {
                last_slot < state.slot + count * (page + 1) as Slot
                    && state.slot + count * page as Slot <= last_slot
            })
            .cloned()
            .collect::<Vec<_>>()
    };
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

/// POST /shards/{shard}/bid
/// $ curl -X POST -d '{"shard":0,"slot":1,"commitment":{"point":[138,242,160,225,209,236,53,174,172,15,28,234,190,70,242,28,171,189,72,181,73,85,194,175,243,3,178,236,97,160,135,229,227,245,224,250,13,243,208,141,120,70,177,2,18,36,183,67],"length":1},"fee":0}' -H 'Content-Type: application/json' http://localhost:3030/shards/0/bid
pub fn shards_bid(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("shards" / Shard / "bid"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(publish_bid)
}

pub async fn publish_bid(
    shard: Shard,
    bid: Bid,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(&mut request_logs, String::from("POST /shards/{shard}/bid"));

    if shard != bid.shard {
        return Err(bid_publication_error(
            simulator::BidPublicationError::InvalidShard {
                expect: shard,
                found: bid.shard,
            },
        ));
    }

    let mut simulator = simulator.lock().await;
    simulator.publish_bid(bid.clone());
    Ok(StatusCode::OK)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BidWithData {
    bid: Bid,
    data: String,
}

/// POST /shards/{shard}/bid_with_data
/// $ curl -X POST -d '{"bid":{"shard":0,"slot":1,"commitment":{"point":[138,242,160,225,209,236,53,174,172,15,28,234,190,70,242,28,171,189,72,181,73,85,194,175,243,3,178,236,97,160,135,229,227,245,224,250,13,243,208,141,120,70,177,2,18,36,183,67],"length":1},"fee":0},"data":"bW91c3Nl"}' -H 'Content-Type: application/json' http://localhost:3030/shards/0/bid_with_data
pub fn shards_bid_with_data(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("shards" / Shard / "bid_with_data"))
        .and(warp::body::content_length_limit(1024 * 1024))
        .and(warp::body::json())
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and_then(publish_bid_with_data)
}

pub async fn publish_bid_with_data(
    shard: Shard,
    bid_with_data: BidWithData,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        String::from("POST /shards/{shard}/bid_with_data"),
    );
    if shard != bid_with_data.bid.shard {
        return Err(bid_publication_error(
            simulator::BidPublicationError::InvalidShard {
                expect: shard,
                found: bid_with_data.bid.shard,
            },
        ));
    }
    let mut simulator = simulator.lock().await;
    let data = base64::decode(&bid_with_data.data).unwrap_or_default();
    match simulator.publish_bid_with_data(bid_with_data.bid, &data) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(bid_publication_error(e)),
    }
}

/// GET /config
pub fn config_get(
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("config"))
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(get_config)
}

pub async fn get_config(
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(&mut request_logs, String::from("GET /config"));
    let config = config.lock().await;
    Ok(warp::reply::json(&config.config))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigOptions {
    auto: Option<bool>,
    slot_time: Option<u64>,
    failure_rate: Option<f32>,
}
/// POST /config
/// $ curl -X POST -d '{"auto":true, "slot_time":1,"failure_rate":0}' -H 'Content-Type: application/json' http://localhost:3030/config
pub fn config_set(
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("config"))
        .and(warp::body::content_length_limit(1024 * 1024))
        .and(warp::body::json())
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(set_config)
}

/// Note: Auto processing restarts when new config is set.
pub async fn set_config(
    config_options: ConfigOptions,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(&mut request_logs, String::from("POST /config"));

    let mut config = config.lock().await;
    if let Some(auto) = config_options.auto {
        config.config.auto = auto;
    }
    if let Some(slot_time) = config_options.slot_time {
        config.config.slot_time = slot_time;
    }
    if let Some(failure_rate) = config_options.failure_rate {
        config.config.failure_rate = failure_rate;
        if !(0.0..=1.0).contains(&failure_rate) {
            return Err(config_set_error(ConfigError::InvalidFailureRate {
                found: failure_rate,
            }));
        }
    }
    if config.config.auto {
        // Reset these time variables when new config is set.
        config.restart_auto();
    }

    Ok(StatusCode::OK)
}

/// POST /simulator/init
/// $ curl -X POST http://localhost:3030/simulator/init
pub fn simulator_init(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("simulator" / "init"))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(init_simulator)
}

pub async fn init_simulator(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(&mut request_logs, String::from("POST /simulator/init"));
    let mut simulator = simulator.lock().await;
    *simulator = Simulator::new();
    // Process the genesis slot.
    simulator.process_slots_happy(0);
    println!("Simulator initiated. Slot 0 is automatically processed.");
    let mut config = config.lock().await;
    config.restart_auto();
    Ok(StatusCode::OK)
}

/// POST /simulator/slot/process/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process/1
pub fn simulator_slot_process(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("simulator" / "slot" / "process" / Slot))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(process_slots)
}

pub async fn process_slots(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!("POST /simulator/slot/process/{}", slot),
    );
    let mut simulator = simulator.lock().await;
    let mut config = config.lock().await;
    config.restart_auto();
    match simulator.process_slots_happy(slot) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(slot_processing_error(e)),
    }
}

/// POST /simulator/slot/process_without_shard_data_inclusion/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_shard_data_inclusion/1
pub fn simulator_slot_process_without_shard_data_inclusion(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_shard_data_inclusion" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(process_slots_without_shard_data_inclusion)
}

pub async fn process_slots_without_shard_data_inclusion(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
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
    let mut config = config.lock().await;
    config.restart_auto();
    match simulator.process_slots_without_shard_data_inclusion(slot) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(slot_processing_error(e)),
    }
}

/// POST /simulator/slot/process_without_shard_blob_proposal/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_shard_blob_proposal/1
pub fn simulator_slot_process_without_shard_blob_proposal(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_shard_blob_proposal" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(process_slots_without_shard_blob_proposal)
}

pub async fn process_slots_without_shard_blob_proposal(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
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
    let mut config = config.lock().await;
    config.restart_auto();
    match simulator.process_slots_without_shard_blob_proposal(slot) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(slot_processing_error(e)),
    }
}

/// POST /simulator/slot/process_without_shard_header_inclusion/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_shard_header_inclusion/1
pub fn simulator_slot_process_without_shard_header_inclusion(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_shard_header_inclusion" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(process_slots_without_shard_header_inclusion)
}

pub async fn process_slots_without_shard_header_inclusion(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
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
    let mut config = config.lock().await;
    config.restart_auto();
    match simulator.process_slots_without_shard_header_inclusion(slot) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(slot_processing_error(e)),
    }
}

/// POST /simulator/slot/process_without_shard_header_confirmation/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_shard_header_confirmation/1
pub fn simulator_slot_process_without_shard_header_confirmation(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_shard_header_confirmation" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(process_slots_without_shard_header_inclusion)
}

pub async fn process_slots_without_shard_header_confirmation(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
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
    let mut config = config.lock().await;
    config.restart_auto();
    match simulator.process_slots_without_shard_header_confirmation(slot) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(slot_processing_error(e)),
    }
}

/// POST /simulator/slot/process_without_beacon_chain_finality/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_beacon_chain_finality/1
pub fn simulator_slot_process_without_beacon_chain_finality(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_beacon_chain_finality" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(process_slots_without_beacon_chain_finality)
}

pub async fn process_slots_without_beacon_chain_finality(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
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
    let mut config = config.lock().await;
    config.restart_auto();
    match simulator.process_slots_without_beacon_chain_finality(slot) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(slot_processing_error(e)),
    }
}

/// POST /simulator/slot/process_without_beacon_block_proposal/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_without_beacon_block_proposal/1
pub fn simulator_slot_process_without_beacon_block_proposal(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!(
            "simulator" / "slot" / "process_without_beacon_block_proposal" / Slot
        ))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(process_slots_without_beacon_block_proposal)
}

pub async fn process_slots_without_beacon_block_proposal(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
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
    let mut config = config.lock().await;
    config.restart_auto();
    match simulator.process_slots_without_beacon_block_proposal(slot) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(slot_processing_error(e)),
    }
}

/// POST /simulator/slot/process_random/{slot_num}
/// $ curl -X POST http://localhost:3030/simulator/slot/process_random/1
pub fn simulator_slot_process_random(
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("simulator" / "slot" / "process_random" / Slot))
        .and(with_simulator(simulator))
        .and(with_request_logs(request_logs))
        .and(with_config(config))
        .and_then(process_slots_random)
}

pub async fn process_slots_random(
    slot: Slot,
    simulator: SharedSimulator,
    request_logs: SharedRequestLogs,
    config: SharedConfig,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        format!("POST /simulator/slot/process_random/{}", slot),
    );
    let mut simulator = simulator.lock().await;
    let mut config = config.lock().await;
    config.restart_auto();
    match simulator.process_slots_random(slot) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(slot_processing_error(e)),
    }
}

/// The no logging endpoint for polling,
/// GET /utils/current_status_for_polling
pub fn utils_current_status_for_polling(
    simulator: SharedSimulator,
    config: SharedConfig,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("utils" / "current_status_for_polling"))
        .and(with_simulator(simulator))
        .and(with_config(config))
        .and_then(get_current_status_for_polling)
}

#[derive(Serialize)]
struct CurrentStatusForPolling {
    slot: Option<Slot>,
    config: Config,
}

pub async fn get_current_status_for_polling(
    simulator: SharedSimulator,
    config: SharedConfig,
) -> Result<impl warp::Reply, Infallible> {
    let simulator = simulator.lock().await;
    let config = config.lock().await;
    let slot = if simulator.beacon_chain.slot == 0 {
        None
    } else {
        Some(simulator.beacon_chain.slot - 1)
    };
    Ok(warp::reply::json(&CurrentStatusForPolling {
        slot,
        config: config.config.clone(),
    }))
}

/// POST /utils/data_commitment
pub fn utils_data_commitment(
    request_logs: SharedRequestLogs,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("utils" / "data_commitment"))
        // .and(warp::body::content_length_limit(1024 * 1024))
        .and(warp::body::json())
        .and(with_request_logs(request_logs))
        .and_then(get_utils_data_commitment)
}

#[derive(Deserialize)]
pub struct UtilsDataCommitmentBody {
    data: String,
}

pub async fn get_utils_data_commitment(
    body: UtilsDataCommitmentBody,
    request_logs: SharedRequestLogs,
) -> Result<impl warp::Reply, Infallible> {
    let mut request_logs = request_logs.lock().await;
    log(
        &mut request_logs,
        String::from("POST /utils/data_commitment"),
    );
    let base64string = body.data;
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
