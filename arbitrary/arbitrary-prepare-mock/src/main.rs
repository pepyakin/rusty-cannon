use arbitrary_state_machine::{
    apply_txn, build_genesis, Backend, Block, InMemoryBackend, State, Txn, ALICE, BOB, CHARLIE,
    DAVE, EVE, H256,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

struct MockBlockchain {
    blocks: Vec<Block>,
    states: BTreeMap<H256, State<InMemoryBackend>>,
}

impl MockBlockchain {
    /// Creates an empty blockchain with a genesis block.
    pub fn new() -> Self {
        let (block0, state0) = build_genesis();
        let root0 = state0.root();
        Self {
            blocks: vec![block0],
            states: vec![(root0, state0)].into_iter().collect(),
        }
    }

    /// Adds a new block with the given transactions. Returns the number of the newly created block.
    /// Panics, if any of the transactions is invalid.
    pub fn new_block(&mut self, txns: Vec<Txn>) -> usize {
        let block_num = self.blocks.len();
        let parent = self.best_block().hash();
        let mut state = State::clone(&self.state_at(self.best_block().state_root));
        for txn in &txns {
            apply_txn(&mut state, txn).unwrap();
        }
        let root = state.root();
        self.blocks.push(Block {
            number: block_num as u64,
            parent,
            state_root: root,
            txns,
        });
        self.states.insert(root, state);
        block_num
    }

    pub fn record_trie_nodes(&self, block_num: usize) -> BTreeMap<H256, Vec<u8>> {
        if block_num == 0 {
            panic!("cannot record trie nodes for genesis block");
        }
        let block = &self.blocks[block_num];
        let parent_block = &self.blocks[block_num - 1];
        let pre_state = State::clone(&self.state_at(parent_block.state_root));
        let recording_backend = RecordingBackend::new(pre_state.into_backend());
        let mut state = State::with_root(recording_backend, parent_block.state_root);
        for txn in &block.txns {
            apply_txn(&mut state, txn).unwrap();
        }
        assert_eq!(state.root(), block.state_root);
        let (_, nodes) = state.into_backend().into_inner();
        nodes
    }

    pub fn block(&self, block_num: usize) -> &Block {
        &self.blocks[block_num]
    }

    pub fn best_block_num(&self) -> usize {
        self.blocks.len() - 1
    }

    pub fn best_block(&self) -> &Block {
        self.blocks.last().unwrap()
    }

    pub fn state_at(&self, state_root: H256) -> &State {
        self.states.get(&state_root).unwrap()
    }
}

fn demo_blockchain() -> MockBlockchain {
    let mut blockchain = MockBlockchain::new();
    blockchain.new_block(vec![Txn::new(ALICE, BOB, 13), Txn::new(BOB, ALICE, 37)]);
    blockchain.new_block(vec![
        Txn::new(ALICE, ALICE, 2),
        Txn::new(BOB, ALICE, 2),
        Txn::new(EVE, ALICE, 8),
    ]);
    blockchain.new_block(vec![
        Txn::new(DAVE, ALICE, 1),
        Txn::new(DAVE, ALICE, 1),
        Txn::new(DAVE, ALICE, 1),
        Txn::new(BOB, DAVE, 2),
    ]);
    blockchain.new_block(vec![Txn::new(CHARLIE, ALICE, 1)]);
    blockchain
}

/// A wrapper backend that records all the accessed nodes.
struct RecordingBackend<B> {
    inner: B,
    // The accessed nodes.
    // The key is the node hash, the value is the node preimage.
    // RefCell is required here because the backend self is passed via a shared reference.
    nodes: RefCell<BTreeMap<H256, Vec<u8>>>,
}

impl<B> RecordingBackend<B> {
    fn new(inner: B) -> Self {
        RecordingBackend {
            inner,
            nodes: RefCell::new(BTreeMap::new()),
        }
    }

    fn into_inner(self) -> (B, BTreeMap<H256, Vec<u8>>) {
        (self.inner, self.nodes.into_inner())
    }
}

impl<B: Backend> Backend for RecordingBackend<B> {
    fn get(&self, key: H256) -> Option<&[u8]> {
        let r = self.inner.get(key);
        self.nodes
            .borrow_mut()
            .insert(key, r.unwrap_or_default().to_vec());
        r
    }
    fn apply_changes(&mut self, adds: BTreeMap<H256, Vec<u8>>, removes: BTreeSet<H256>) {
        self.inner.apply_changes(adds, removes);
    }
}

fn dump_block(root: &Path, block_num: usize, blockchain: &MockBlockchain) {
    let parent = blockchain.block(block_num - 1);
    let block = blockchain.block(block_num);

    println!("block {}", block_num);
    println!("  hash: {:?}", block.hash());
    println!("  parent: {:?}", parent.hash());
    println!("  state root: {:?}", block.state_root);
    println!();

    std::fs::create_dir_all(root.clone()).unwrap();
    std::fs::write(root.join("input"), block.hash()).unwrap();
    std::fs::write(root.join("output"), block.state_root).unwrap();

    // Serialize the current and parent block preimages
    std::fs::write(
        root.join(format!("0x{:?}", parent.hash())),
        &parent.serialize(),
    )
    .unwrap();
    std::fs::write(
        root.join(format!("0x{:?}", block.hash())),
        &block.serialize(),
    )
    .unwrap();
    std::fs::write(root.join("block"), &block.serialize()).unwrap();

    // Serialize the trie nodes
    let nodes = blockchain.record_trie_nodes(block_num);
    for (key, value) in nodes {
        std::fs::write(root.join(format!("0x{:?}", key)), &value).unwrap();
    }
}

fn main() {
    let b = demo_blockchain();
    for i in 0..b.best_block_num() {
        // Cannon is a bit weird in how it identifies challenges.
        //
        // Per Cannon, challenging the block N means challenging the state transition from the
        // block N to N+1. That means:
        //
        // a) even though we say challenging `N`, `N` is the last known good block.
        // b) N+1 is the actually the block the results of which are challenged.
        //
        // Most importantly, that implies that here we need to put all the data required for
        // re-execution of the block N+1 into the directory for block N.
        let last_good_block_num = i;
        let challenged_block_num = i + 1;
        let root = PathBuf::from(format!("/tmp/cannon/0_{last_good_block_num}"));
        dump_block(&root, challenged_block_num, &b);
    }
}
