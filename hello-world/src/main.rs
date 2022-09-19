#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;
extern crate rlibc;

mod heap;
mod iommu;

#[derive(Eq, PartialEq, Hash)]
pub struct H256(pub [u8; 32]);

#[no_mangle] 
pub extern "C" fn _start() {
    unsafe { heap::init() };

    let input_hash = iommu::input_hash();
    let mut v = alloc::vec::Vec::with_capacity(32);
    v.extend_from_slice(&input_hash.0);
    iommu::output(H256([1; 32]));
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { iommu::halt() }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!();
}
