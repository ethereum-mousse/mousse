pub use crate::*;

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
