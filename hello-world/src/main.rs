#![no_std]
#![no_main]

mod iommu;

#[derive(Eq, PartialEq, Hash)]
pub struct H256(pub [u8; 32]);

#[no_mangle] 
pub extern "C" fn _start() {
    let input_hash = iommu::input_hash();
    // let mut v = Vec::with_capacity(32);
    // v.extend_from_slice(&input_hash.0);
    iommu::output(H256([1; 32]));
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { iommu::halt() }
}
