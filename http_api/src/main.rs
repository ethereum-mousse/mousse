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
use tokio::sync::Mutex;
use warp::{http::StatusCode, reject, Filter};

pub use common::bid::Bid;
pub use common::eth2_config::*;
pub use common::eth2_types::*;

pub mod errors;
pub use errors::*;
pub mod endpoints;
pub use endpoints::*;

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
