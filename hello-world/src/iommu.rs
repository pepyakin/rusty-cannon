//! This module is used when the state machine compiled into MIPS to interact with the host
//! environment. The host environment is either the prover or the onchain verifier.

use bigint::H256;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

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
        *(PTR_OUTPUT_HASH as *mut [u8; 32]) = hash.0;
        *(PTR_MAGIC as *mut u32) = 0x1337f00d;
    }
    halt();
}

/// Normal stop of the execution.
fn halt() -> ! {
    unsafe {
        libc::exit(0);
    }
}

/// Request the preimage from the host.
pub fn preimage(hash: H256) -> Option<Vec<u8>> {
    // The cache of all requested preimages to avoid going via the host boundary every time.
    //
    // Under MIPS this is running exclusively in single-threaded mode. We could've avoided using
    // a Mutex, but it seems to be fine. Uncontended use is just atomic writes.
    static PREIMAGE_CACHE: Lazy<Mutex<HashMap<H256, Vec<u8>>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    // Check if the preimage is already cached.
    unsafe {
        let mut preimage_cache = PREIMAGE_CACHE.lock().unwrap();
        if let Some(preimage) = preimage_cache.get(&hash) {
            return Some(preimage.clone());
        }

        *(PTR_PREIMAGE_ORACLE_HASH as *mut [u8; 32]) = hash.0;

        // Issue the `getpid` syscall. In unicorn emu this will cause the host to read the hash
        // and write the result.
        //
        // `getpid` is intentionally not cached by `libc`s. In case this
        // does not work, we should issue the syscall directly.
        let _ = libc::getpid();

        // Read the size of the preimage. It seems to be BE, so no conversion needed.
        let size = *(PTR_PREIMAGE_ORACLE_SIZE as *const u32);
        if size == 0 {
            return None;
        }

        // Read the preimage.
        //
        // SAFETY: The pointer is aligned by definition and is not null.
        let preimage =
            std::slice::from_raw_parts(PTR_PREIMAGE_ORACLE_DATA as *const u8, size as usize)
                .to_vec();

        // TODO: check that the hash of the preimage actually corresponds to the requested hash.

        preimage_cache.insert(hash, preimage.clone());

        Some(preimage)
    }
}

