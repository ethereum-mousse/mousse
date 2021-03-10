# Eth2 Data Sharding Simulator
The Eth2 data sharding simulator is the core part of Mousse.
It simulates the progress of the beacon chain and the interaction to put data on shards.

## Background
Data sharding allows users to put arbitrary data on shards, which is mainly useful for rollups. In every shard, a shard "data blob" is proposed by a validator at each slot, and the shard header with the cryptographic commitment to that data is included by a beacon block. The data's availability is verified by the shard committee (i.e., the validators allocated to that shard). If a sufficient number of attestations to the shard header by the committee is observed, the header becomes "confirmed" in the beacon state. (For some educational resources on Eth2 data sharding, please refer to the contents of [the Eth2 online workshop in Feb 2021](https://hackmd.io/@hww/workshop_feb_2021)).

Data sharding is supposed to co-exist with the Eth1 chain at first. Therefore, there is nothing similar to Eth1's smart contracts in Eth2, and any application, including rollups, uses Eth2 for the data availability engine and Eth1 for execution.

The simulator is based on the data sharding spec in the two pull requests ([#2146](https://github.com/ethereum/eth2.0-specs/pull/2146) and [#2172](https://github.com/ethereum/eth2.0-specs/pull/2172)) in the Eth2 spec repository.

Also, it is assumed that users pay validators in return for including their data in shard blobs. For this, users publish "bids" to request a validator to put a certain data and pay the validator the fee via the "fee market manager contract" in Eth1. See [this post](https://ethresear.ch/t/a-fee-market-contract-for-eth2-shards-in-eth1/8124) for the background.

## The scope of the simulator
Mousse simulator covers the following things:
- Beacon chain 
  - Beacon chain consensus
  - Transitions of beacon state
- Shard data blob proposal
- Bidding and data publication on shards
  - Note: Unlike bids, the actual shard data is not stored in the simulator.


On the other hand, it does NOT simulate the following things:
- P2P network (gossip protocols, peer discovery, etc.)
- Validators
  - Validator-related fields (e.g., balances) are removed from the beacon state.
- Attestations, fork-choice rules
  - The consensus is quite abstracted in the simulation. Each checkpoint block can just be marked as finalized after two or more epochs.
- Data availability mechanism

Also, in the current implementation, cryptographic primitives such as BLS Signatures and polynomial commitments are replaced by dummy values (basic hashes of the target data)


## Simulation model
The simulator performs a discrete-event simulation, which proceeds with slots. You can always register a bid by specifying the target shard, target slot, and data commitment. Then, what happened at each slot are the followings:
- Generate the shard header of each shard.
  - Here, one bid is selected, and its data commitment is written in the shard header.
- A new beacon block can be proposed, including the shard headers published but not included yet.
- The shard headers can be confirmed on the beacon state.

### Failures
For testing applications with Mousse, the simulator supports multiple failure cases of the Eth2 system. Specifically, it can simulate the following failure scenarios in each slot:
- Beacon block is not proposed.
- Beacon chain finality is delayed.
- Shard header is not included.
- Shard header confirmation is delayed
- Shard blob does not include data.
(Also, it can randomly pick one from the above.)

Currently, there is no way to set shard-by-shard configuration, i.e., all the shards succeed or fail at the same time.

The simulator assumes there is no equivocating validator, i.e., at most one beacon block and one blob per shard is proposed per slot.
