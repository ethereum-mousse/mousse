pub use crate::*;

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
