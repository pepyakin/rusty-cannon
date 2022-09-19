//! This module is used when the state machine compiled into MIPS to interact with the host
//! environment. The host environment is either the prover or the onchain verifier.

use super::H256;
use core::ptr;

/// The address of the input hash.
const PTR_INPUT_HASH: usize = 0x30000000;
/// The address where the output hash is written at the end of execution.
const PTR_OUTPUT_HASH: usize = 0x30000804;
/// The address where a special magic value is written at the end of execution.
const PTR_MAGIC: usize = 0x30000800;
/// The address where the preimage hash for the preimage oracle is written by the guest.
const PTR_PREIMAGE_ORACLE_HASH: usize = 0x30001000;
/// The address where the preimage oracle output size is written by the host.
const PTR_PREIMAGE_ORACLE_SIZE: usize = 0x31000000;
/// The address where the preimage oracle output data is written by the host.
const PTR_PREIMAGE_ORACLE_DATA: usize = 0x31000004;

/// Loads the input hash from the host environment.
pub fn input_hash() -> H256 {
    H256(unsafe { *(PTR_INPUT_HASH as *const [u8; 32]) })
}

/// Prepares the guest envrionment to exiting. Writes the output hash and the magic to be read by
/// the host and then halts the execution.
pub fn output(hash: H256) -> ! {
    // TODO: consider writing the receipts.
    unsafe {
        ptr::write_volatile(PTR_MAGIC as *mut u32, 0x1337f00d);
        ptr::write_volatile(PTR_OUTPUT_HASH as *mut [u8; 32], hash.0);
        halt();
    }
}

extern "C" {
    pub fn halt() -> !;
}
