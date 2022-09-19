#![no_std]

mod iommu;

#[derive(Eq, PartialEq, Hash)]
pub struct H256(pub [u8; 32]);

// #[no_mangle]
//pub extern "C" fn main() {
fn main() {
    let input_hash = iommu::input_hash();
    // let mut v = Vec::with_capacity(32);
    // v.extend_from_slice(&input_hash.0);
    iommu::output(H256([1; 32]));
}
