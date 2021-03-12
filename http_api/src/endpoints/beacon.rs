pub use crate::*;

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
