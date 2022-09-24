#![no_std]

extern crate alloc;

mod primitives;
mod state;

use alloc::vec::Vec;

pub use bigint::{H256, U256};
pub use primitives::*;
pub use state::{Backend, InMemoryBackend, State};

pub const ALICE: H256 = H256([0x01; 32]);
pub const BOB: H256 = H256([0x02; 32]);
pub const CHARLIE: H256 = H256([0x03; 32]);
pub const DAVE: H256 = H256([0x04; 32]);
pub const EVE: H256 = H256([0x05; 32]);

pub fn execute(state: &mut State<impl Backend>, block: Block) {
    for txn in &block.txns {
        apply_txn(state, txn).unwrap();
    }
}

/// An error that can occur when applying a transaction.
#[derive(Debug)]
pub struct InsufficientFunds;

/// Apply a transaction to the state. Returns an error if the transaction is invalid.
pub fn apply_txn(state: &mut State<impl Backend>, txn: &Txn) -> Result<(), InsufficientFunds> {
    let source = state.get(txn.from).unwrap_or_default();
    if source < txn.value {
        return Err(InsufficientFunds);
    }
    let dest = state.get(txn.to).unwrap_or_default();
    state.set(txn.from, source - txn.value);
    state.set(txn.to, dest + txn.value);
    Ok(())
}

/// Creates the genesis state with filled balances for ALICE and BOB.
pub fn build_genesis() -> (Block, State<InMemoryBackend>) {
    let backend = InMemoryBackend::new();
    let mut state = State::empty(backend);
    state.set(ALICE, 100);
    state.set(BOB, 90);
    state.set(CHARLIE, 80);
    state.set(DAVE, 70);
    state.set(EVE, 60);

    let block = Block {
        number: 0,
        parent: H256::zero(),
        state_root: state.root(),
        txns: Vec::new(),
    };

    (block, state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let (_block, mut state) = build_genesis();
        let genesis_root = state.root();

        // Send a transaction.
        let txn = Txn {
            from: ALICE,
            to: BOB,
            value: 10,
        };
        apply_txn(&mut state, &txn).unwrap();
        assert_eq!(state.get(ALICE), Some(90));
        assert_eq!(state.get(BOB), Some(100));
        assert_ne!(state.root(), genesis_root);

        // Then send the inverse transaction. That should return us to the initial state.
        let txn = Txn {
            from: BOB,
            to: ALICE,
            value: 10,
        };
        apply_txn(&mut state, &txn).unwrap();
        assert_eq!(state.get(ALICE), Some(100));
        assert_eq!(state.get(BOB), Some(90));
        assert_eq!(state.root(), genesis_root);
    }
}
