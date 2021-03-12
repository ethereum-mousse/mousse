pub use crate::*;

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
