//! Various types that constitute a block chain.

use alloc::vec::Vec;
use bigint::H256;

pub struct Txn {
    pub from: H256,
    pub to: H256,
    pub value: u64,
}

impl rlp::Encodable for Txn {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(3);
        s.append(&self.from);
        s.append(&self.to);
        s.append(&self.value);
    }
}

impl rlp::Decodable for Txn {
    fn decode(rlp: &rlp::UntrustedRlp) -> Result<Self, rlp::DecoderError> {
        Ok(Txn {
            from: rlp.val_at(0)?,
            to: rlp.val_at(1)?,
            value: rlp.val_at(2)?,
        })
    }
}

impl Txn {
    pub fn new(from: H256, to: H256, value: u64) -> Self {
        Txn { from, to, value }
    }
}

pub struct Block {
    pub number: u64,
    pub parent: H256,
    pub state_root: H256,
    pub txns: Vec<Txn>,
}

impl rlp::Encodable for Block {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(3);
        s.append(&self.number);
        s.append(&self.parent);
        s.append(&self.state_root);
        s.append_list(&self.txns);
    }
}

impl rlp::Decodable for Block {
    fn decode(rlp: &rlp::UntrustedRlp) -> Result<Self, rlp::DecoderError> {
        Ok(Block {
            number: rlp.val_at(0)?,
            parent: rlp.val_at(1)?,
            state_root: rlp.val_at(2)?,
            txns: rlp.list_at(3)?,
        })
    }
}

impl Block {
    pub fn hash(&self) -> H256 {
        H256::from_slice(keccak256(&self.serialize()).as_ref())
    }

    pub fn serialize(&self) -> Vec<u8> {
        use rlp::Encodable;
        let mut stream = rlp::RlpStream::new();
        self.rlp_append(&mut stream);
        stream.out()
    }
}

pub fn keccak256(bytes: &[u8]) -> H256 {
    use sha3::{Digest, Keccak256};
    H256::from_slice(Keccak256::digest(bytes).as_ref())
}
