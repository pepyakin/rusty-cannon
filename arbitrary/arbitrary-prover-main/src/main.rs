//! This program is compiled into MIPS and is used for the onchain verification.
//! 
//! It just wraps the `arbitrary-state-machine` crate providing the necessary interface with the 
//! host (onchain verfier or offchain prover).

#![feature(alloc_error_handler)] // no_std and allocator support is not stable.
#![feature(stdsimd)] // for `mips::break_`. If desired, this could be replaced with asm.
#![no_std]
#![no_main]

extern crate alloc;
extern crate rlibc; // memcpy, and friends

mod heap;
mod iommu;

use alloc::boxed::Box;
use arbitrary_state_machine::{Backend, Block, State, H256};

use rlp::{Decodable, UntrustedRlp};

/// The trie backend implementation that delegates the trie node requests to the preimage oracle.
struct OracleBackend;
impl Backend for OracleBackend {
    fn get(&self, key: H256) -> Option<&[u8]> {
        iommu::preimage(key)
    }
}

/// Given the blockhash returns the block.
fn lookup_block(hash: H256) -> Option<Block> {
    let block_rlp = iommu::preimage(hash)?;
    UntrustedRlp::new(&block_rlp).as_val().ok()
}

/// Main entrypoint.
#[no_mangle]
pub extern "C" fn _start() {
    unsafe { heap::init() };

    let input_block = iommu::input_hash();

    let block = lookup_block(input_block).unwrap();
    let parent_block = lookup_block(block.parent).unwrap();

    let mut state = State::with_root(OracleBackend, parent_block.state_root);
    arbitrary_state_machine::execute(&mut state, block);
    let output = state.root();

    iommu::output(output);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Uncomment code below if you're in trouble
    /* 
    let msg = alloc::format!("Panic: {}", info);
    iommu::print(&msg);
    */ 

    unsafe {
        core::arch::mips::break_();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(_layout: alloc::alloc::Layout) -> ! {
    // NOTE: avoid `panic!` here, technically, it might not be allowed to panic in an OOM situation.
    //       with panic=abort it should work, but it's no biggie use `break` here anyway.
    unsafe {
        core::arch::mips::break_();
    }
}
