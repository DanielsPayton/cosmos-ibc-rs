# ADR 009: Revamp IBC integration test framework

## Changelog

- 04-03-2024: Initial draft

## Context

The current framework in the IBC testkit uses
[existing types and injects state or dependency
manually](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/tests/core/ics02_client/update_client.rs#L574-L578).
Sometimes, it uses
[semantically wrong data as mock data](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/testapp/ibc/core/types.rs#L320).
Because of this, tests with customizable steps and fixed fixtures became ad-hoc
and unmaintainable.

To overcome this, we need to improve our test framework that allows:

- testing different implementations (traits).
- succinct tests (useful `util` methods).
- improving test coverage (Merkle proof generation).
- integration tests exercising the IBC workflow (relayer-like interface)

## Decision

The main goal of this proposal is to create a test framework that is modular and
closer to the real blockchain environment. This should also make the existing
tests succinct and readable. Instead of bootstrapping the mock data, we should
use valid steps to generate it - so that we know the exact steps to reach a
state to reproduce in a real environment.

To achieve this, we have broken down the proposal into sub-proposals.

### Adopt a Merkle store for the test framework

The current framework uses `HashMap` and `HashSet` to store data. This works for
many test scenarios, but it fails to test proof-sensitive scenarios. Because of
this, we don't have any connection, channel handshake or packet relay tests for
Tendermint light client.

We generalize
[`MockContext`](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/testapp/ibc/core/types.rs#L103)
to use a Merkle store which is used for IBC Context's store. For concrete or
default implementation, we can use the IAVL Merkle store implementation from
`informalsystems/basecoin-rs`.

### Modularize the host environment

Currently, we are using `Mock` and `SyntheticTendermint`
[variants](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/hosts/block.rs#L33-L36)
as the host environments. To manage these two different environments, we also
introduced
[`HostBlocks`](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/hosts/block.rs#L72-75)
for corresponding host variants.

This creates friction if we have to add a new host variant. It creates the same
problem as why we have `ibc-derive` crate for `ClientState` and
`ConsensusState`. This should be refactored by a generic `TestHost` trait that
maintains its corresponding types e.g. `Block` types, via associated types.
Finally, we generalize the `MockContext` once more to use this `TestHost` trait
instead of a concrete enum variant.

This `TestHost` trait should be responsible for generating blocks, headers,
client and consensus states specific to that host environment.

```rs
/// TestHost is a trait that defines the interface for a host blockchain.
pub trait TestHost: Default + Debug + Sized {
    /// The type of block produced by the host.
    type Block: TestBlock;

    /// The type of client state produced by the host.
    type ClientState: Into<AnyClientState> + Debug;

    /// The type of block parameters to produce a block
    type BlockParams: Debug + Default;

    /// The type of light client parameters to produce a light client state
    type LightClientParams: Debug + Default;

    /// The history of blocks produced by the host chain.
    fn history(&self) -> &VecDeque<Self::Block>;

    /// Triggers the advancing of the host chain, by extending the history of blocks (or headers).
    fn advance_block(
        &mut self,
        commitment_root: Vec<u8>,
        block_time: Duration,
        params: &Self::BlockParams,
    );

    /// Generate a block at the given height and timestamp, using the provided parameters.
    fn generate_block(
        &self,
        commitment_root: Vec<u8>,
        height: u64,
        timestamp: Timestamp,
        params: &Self::BlockParams,
    ) -> Self::Block;

    /// Generate a client state using the block at the given height and the provided parameters.
    fn generate_client_state(
        &self,
        latest_height: &Height,
        params: &Self::LightClientParams,
    ) -> Self::ClientState;
}

/// TestBlock is a trait that defines the interface for a block produced by a host blockchain.
pub trait TestBlock: Clone + Debug {
    /// The type of header can be extracted from the block.
    type Header: TestHeader;

    /// The height of the block.
    fn height(&self) -> Height;

    /// The timestamp of the block.
    fn timestamp(&self) -> Timestamp;

    /// Extract the header from the block.
    fn into_header(self) -> Self::Header;
}

/// TestHeader is a trait that defines the interface for a header produced by a host blockchain.
pub trait TestHeader: Clone + Debug + Into<Any> {
    /// The type of consensus state can be extracted from the header.
    type ConsensusState: ConsensusState + Into<AnyConsensusState> + From<Self> + Clone + Debug;

    /// The height of the block, as recorded in the header.
    fn height(&self) -> Height;

    /// The timestamp of the block, as recorded in the header.
    fn timestamp(&self) -> Timestamp;

    /// Extract the consensus state from the header.
    fn into_consensus_state(self) -> Self::ConsensusState;
}
```

### Decoupling IbcContext and Host environment

Currently, `MockContext` implements the top validation and execution context of
`ibc-rs`. It contains other host-specific data e.g. `host_chain_id`,
`block_time` - that are irrelevant to the IBC context directly. If we think
`MockContext` as a real blockchain context, the `MockContext` represents the top
runtime - which contains the IBC module. So we implement the validation and
execution context on `MockIbcStore`, instead of `MockContext`.

With this, the `MockContext` contains two decoupled parts - the host and the IBC
module.

### Chain-like interface for `MockContext`

With the above changes, we can refactor the `MockContext` to have
blockchain-like interfaces.

The `MockContext` should have `end_block`, `produce_block` and `begin_block` to
mimic the real blockchain environment such as Cosmos-SDK.

```rs
impl<S, H> MockContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    pub fn ibc_store_mut(&mut self) -> &mut MockIbcStore<S>;
    pub fn host_mut(&mut self) -> &mut H;

    pub fn generate_genesis_block(&mut self, genesis_time: Timestamp);
    pub fn begin_block(&mut self);
    pub fn end_block(&mut self);
    pub fn produce_block(&mut self, block_time: Duration);
}
```

### ICS23 compatible proof generation

With the new ability of proof generation, we can now test the Tendermint light
clients. But we need our proofs to be ICS23 compatible. ICS23 expects the IBC
store root to be committed at a commitment prefix at a top store in the host
environment.

For this, we add an extra store in `MockContext` where the `MockIbcStore`
commits its storage root at its
[commitment prefix](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/testapp/ibc/core/core_ctx.rs#L127-L129)
key.

So the `MockContext` is finalized as:

```rs
pub struct MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost
{
    pub main_store: S,
    pub host: H,
    pub ibc_store: MockIbcStore<S>,
}
```

Now the `MockIbcStore` can generate proofs that contain the proofs in its store
and commitment prefix. But it has to know the proofs of its commitment prefix of
the previous heights.

So we add an extra store in `MockIbcStore` to store the proofs from previous
heights. This is similar to storing `HostConsensusState` of previous heights.

```rs
#[derive(Debug)]
pub struct MockIbcStore<S>
where
    S: ProvableStore + Debug,
{
    ...
    /// Map of host consensus states
    pub host_consensus_states: Arc<Mutex<BTreeMap<u64, AnyConsensusState>>>,
    /// Map of proofs of ibc commitment prefix
    pub ibc_commiment_proofs: Arc<Mutex<BTreeMap<u64, CommitmentProof>>>,
}
```

The storing of the IBC store root at the IBC commitment prefix happens in the
end block. The storing of proofs and host consensus states happens in the
`begin_block` of the `MockContext`.

### Integration Test via `RelayerContext`

With all the above changes, we can now write an integration test that tests the
IBC workflow - client creation, connection handshake, channel handshake and
packet relay for any host environment that implements `TestHost`.

This can be done by reading the
[IBC events](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/testapp/ibc/core/types.rs#L95)
from `MockIbcStore` and creating and sending the IBC messages via
[`MockContext::deliver`](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/testapp/ibc/core/types.rs#L696).

### Miscellaneous

To achieve blockchain-like interfaces, we removed `max_history_size` and
`host_chain_id` from `MockContext`.

- `max_history_size`: We generate all the blocks till a block height. This gives
  us reproducibility. If we need to prune some older block data, we use a
  dedicated `prune_block_till` to prune older blocks. This makes our tests more
  descriptive about the assumption of the test scenarios.
- `host_chain_id`: The IBC runtime does not depend on `host_chain_id` directly.
  The `TestHost` trait implementation is responsible for generating the blocks
  with the necessary data.

Also to minimize verbosity while writing tests (as Rust doesn't support default
arguments to function parameters), we want to use some parameter builders. For
that, we can use [`TypedBuilder`](https://crates.io/crates/typed-builder) crate.

## Status

Proposed

## Consequences

This ADR pays the technical debt of the existing testing framework.

### Positive

The future tests will be more readable and maintainable. The test framework
becomes modular and leverages Rust's trait system. Even the `ibc-rs` users may
benefit from this framework to test their implementations of `ibc-rs`
components.

### Negative

This requires a significant refactoring of the existing tests. Since this may
take some time, the parallel development on `main` branch may conflict with this
work.

## References

This work is being tracked at
[cosmos/ibc-rs#1109](https://github.com/cosmos/ibc-rs/pull/1109)