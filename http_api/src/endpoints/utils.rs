pub use crate::*;

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
        .and_then(calc_data_commitment)
}

#[derive(Deserialize)]
pub struct UtilsDataCommitmentBody {
    data: String,
}

pub async fn calc_data_commitment(
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
