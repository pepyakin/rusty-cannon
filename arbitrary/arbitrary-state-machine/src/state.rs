//! This module describes the state of the blockchain.
//!
//! The state basically stores a mapping from an address to a balance.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use bigint::H256;

#[derive(Clone)]
pub struct State<B = InMemoryBackend> {
    root: H256,
    backend: B,
}

impl<B: Backend> State<B> {
    /// Create a new empty state.
    pub fn empty(backend: B) -> Self {
        Self::with_root(backend, trie::EMPTY_TRIE_HASH)
    }

    /// Creaates a new state at the given root.
    pub fn with_root(backend: B, root: H256) -> Self {
        State { root, backend }
    }

    /// Gets the balance for the specified address.
    pub fn get(&self, key: H256) -> Option<u64> {
        trie::get(self.root, &BackendWrapper(&self.backend), &key.0)
            .unwrap()
            .map(|bytes| {
                let mut le = [0; 8];
                le.copy_from_slice(&bytes);
                u64::from_le_bytes(le)
            })
    }

    /// Sets the balance for the specified address.
    pub fn set(&mut self, key: H256, value: u64) {
        let (root, change) = trie::insert(
            self.root,
            &BackendWrapper(&self.backend),
            &key.0,
            &value.to_le_bytes(),
        )
        .unwrap();
        self.backend.apply_changes(change.adds, change.removes);
        self.root = root;
    }

    /// Returns the root of the state.
    pub fn root(&self) -> H256 {
        self.root
    }

    /// Consume the state and return the backend.
    pub fn into_backend(self) -> B {
        self.backend
    }

    /// Returns a reference to the backend.
    pub fn backend_ref(&self) -> &B {
        &self.backend
    }
}

struct BackendWrapper<'a>(&'a dyn Backend);

impl trie::DatabaseHandle for BackendWrapper<'_> {
    fn get(&self, key: H256) -> Option<&[u8]> {
        self.0.get(key)
    }
}

/// An abstraction for a trie backend. Expected to keep track of the nodes.
pub trait Backend {
    /// Get the given nodes from the backend, or `None` if not present.
    fn get(&self, key: H256) -> Option<&[u8]>;
    /// Apply the given change set to the backend. Technically, a confirming implementation does not
    /// have to remove the nodes in `removes`. The order of processing is `adds` first and then
    /// `removes`.
    fn apply_changes(&mut self, adds: BTreeMap<H256, Vec<u8>>, removes: BTreeSet<H256>) {
        // provided implementation since as shown by the preimage oracle not all backends need
        // to keep track of changes.
        drop((adds, removes));
    }
}

/// A simple trie backend implementation backed by a b-tree map.
#[derive(Clone)]
pub struct InMemoryBackend {
    nodes: BTreeMap<H256, Vec<u8>>,
}

impl InMemoryBackend {
    /// Create a new in-memory trie backend.
    pub fn new() -> InMemoryBackend {
        InMemoryBackend {
            nodes: BTreeMap::new(),
        }
    }
}

impl Backend for InMemoryBackend {
    fn get(&self, key: H256) -> Option<&[u8]> {
        self.nodes.get(&key).map(|v| v.as_ref())
    }
    fn apply_changes(&mut self, adds: BTreeMap<H256, Vec<u8>>, removes: BTreeSet<H256>) {
        for (key, value) in adds {
            self.nodes.insert(key, value);
        }
        for key in removes {
            self.nodes.remove(&key);
        }
    }
}
