use criterion::{criterion_group, criterion_main, Criterion};
use eth2_simulator::*;
use simulator::Simulator;


fn process_1000_slots_happy() {
    let mut simulator = Simulator::new();
    let end_slot: Slot = 1000;
    let result = simulator.process_slots_happy(end_slot);
    assert!(result.is_ok());
}

fn process_1000_slots_happy_with_bids() {    
    let mut simulator = Simulator::new();
    let end_slot: Slot = 1000;

    for slot in 0..end_slot {
        for shard in 0..SHARD_NUM as Shard {
            // Publish a bid with low fee and high fee.
            let low_fee_bid = Bid {
                shard: shard,
                slot: slot,
                commitment: DataCommitment::dummy_from_bytes(
                    &String::from(format!("Bid with a low fee: Slot {}, Shard {}", slot, shard)).into_bytes()
                ),
                fee: 1,
            };
            let high_fee_bid = Bid {
                    shard: shard,
                    slot: slot,
                    commitment: DataCommitment::dummy_from_bytes(
                        &String::from(format!("Bid with a high fee: Slot {}, Shard {}", slot, shard)).into_bytes()
                    ),
                    fee: 21000 * 100,
                };
            let result = simulator.publish_bid(low_fee_bid);
            assert!(result.is_ok());
            let result = simulator.publish_bid(high_fee_bid);
            assert!(result.is_ok());
        }
    }

    let result = simulator.process_slots_happy(end_slot);
    assert!(result.is_ok());
}

fn process_1000_slots_random() {
    let mut simulator = Simulator::new();
    let end_slot: Slot = 1000;
    let result = simulator.process_slots_random(end_slot);
    assert!(result.is_ok());
}

fn process_1000_slots_random_with_bids() {
    let mut simulator = Simulator::new();
    let end_slot: Slot = 1000;

    for slot in 0..end_slot {
        for shard in 0..SHARD_NUM as Shard {
            // Publish a bid with low fee and high fee.
            let low_fee_bid = Bid {
                shard: shard,
                slot: slot,
                commitment: DataCommitment::dummy_from_bytes(
                    &String::from(format!("Bid with a low fee: Slot {}, Shard {}", slot, shard)).into_bytes()
                ),
                fee: 1,
            };
            let high_fee_bid = Bid {
                    shard: shard,
                    slot: slot,
                    commitment: DataCommitment::dummy_from_bytes(
                        &String::from(format!("Bid with a high fee: Slot {}, Shard {}", slot, shard)).into_bytes()
                    ),
                    fee: 21000 * 100,
                };
            let result = simulator.publish_bid(low_fee_bid);
            assert!(result.is_ok());
            let result = simulator.publish_bid(high_fee_bid);
            assert!(result.is_ok());
        }
    }

    let result = simulator.process_slots_random(end_slot);
    assert!(result.is_ok());
}


fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("group-example");
    group.bench_function("Bench process_slots_happy(1000)", |b| b.iter(|| process_1000_slots_happy()));
    group.bench_function("Bench process_slots_happy(1000) with bids", |b| b.iter(|| process_1000_slots_happy_with_bids()));
    group.bench_function("Bench process_slots_random(1000)", |b| b.iter(|| process_1000_slots_random()));
    group.bench_function("Bench process_slots_random(1000) with bids", |b| b.iter(|| process_1000_slots_random_with_bids()));

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
