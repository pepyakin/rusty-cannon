// Copyright 2015-2017 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "std")] use std::collections::{HashMap as Map};
#[cfg(not(feature = "std"))] use alloc::{collections::btree_map::BTreeMap as Map};
#[cfg(not(feature = "std"))] use alloc::vec::Vec;
#[cfg(feature = "std")]
use common::{BLOCKS_RLP_SWAPPER, SNAPSHOT_RLP_SWAPPER};
use {UntrustedRlp, Compressible, encode, RlpStream};

/// Stores RLPs used for compression
pub struct InvalidRlpSwapper<'a> {
	invalid_to_valid: Map<&'a [u8], &'a [u8]>,
	valid_to_invalid: Map<&'a [u8], &'a [u8]>,
}

impl<'a> InvalidRlpSwapper<'a> {
	/// Construct a swapper from a list of common RLPs
	pub fn new(rlps_to_swap: &[&'a [u8]], invalid_rlps: &[&'a [u8]]) -> Self {
		if rlps_to_swap.len() > 0x7e {
			panic!("Invalid usage, only 127 RLPs can be swappable.");
		}
		let mut invalid_to_valid = Map::new();
		let mut valid_to_invalid = Map::new();
		for (&rlp, &invalid) in rlps_to_swap.iter().zip(invalid_rlps.iter()) {
			invalid_to_valid.insert(invalid, rlp);
			valid_to_invalid.insert(rlp, invalid);
		}
		InvalidRlpSwapper {
			invalid_to_valid: invalid_to_valid,
			valid_to_invalid: valid_to_invalid
		}
	}
	/// Get a valid RLP corresponding to an invalid one
	fn get_valid(&self, invalid_rlp: &[u8]) -> Option<&[u8]> {
		self.invalid_to_valid.get(invalid_rlp).cloned()
	}
	/// Get an invalid RLP corresponding to a valid one
	fn get_invalid(&self, valid_rlp: &[u8]) -> Option<&[u8]> {
		self.valid_to_invalid.get(valid_rlp).cloned()
	}
}

/// Type of RLP indicating its origin database.
pub enum RlpType {
	/// RLP used in blocks database.
	Blocks,
	/// RLP used in snapshots.
	Snapshot,
}

fn to_elastic(slice: &[u8]) ->  Vec<u8> {
	let mut out = Vec::new();
	out.extend_from_slice(slice);
	out
}

fn map_rlp<F>(rlp: &UntrustedRlp, f: F) -> Option<Vec<u8>> where
	F: Fn(&UntrustedRlp) -> Option<Vec<u8>> {
	match rlp.iter()
		.fold((false, RlpStream::new_list(rlp.item_count().unwrap_or(0))),
		|(is_some, mut acc), subrlp| {
  		let new = f(&subrlp);
  		if let Some(ref insert) = new {
  			acc.append_raw(&insert[..], 1);
  		} else {
  			acc.append_raw(subrlp.as_raw(), 1);
  		}
  		(is_some || new.is_some(), acc)
  	  }) {
  	(true, s) => Some(s.drain()),
  	_ => None,
  }
}

/// Replace common RLPs with invalid shorter ones.
fn simple_compress(rlp: &UntrustedRlp, swapper: &InvalidRlpSwapper) -> Vec<u8> {
	if rlp.is_data() {
		to_elastic(swapper.get_invalid(rlp.as_raw()).unwrap_or_else(|| rlp.as_raw()))
	} else {
		map_rlp(rlp, |r| Some(simple_compress(r, swapper))).unwrap_or_else(|| to_elastic(rlp.as_raw()))
	}
}

/// Recover valid RLP from a compressed form.
fn simple_decompress(rlp: &UntrustedRlp, swapper: &InvalidRlpSwapper) -> Vec<u8> {
	if rlp.is_data() {
		to_elastic(swapper.get_valid(rlp.as_raw()).unwrap_or_else(|| rlp.as_raw()))
	} else {
		map_rlp(rlp, |r| Some(simple_decompress(r, swapper))).unwrap_or_else(|| to_elastic(rlp.as_raw()))
	}
}

/// Replace common RLPs with invalid shorter ones, None if no compression achieved.
/// Tries to compress data insides.
fn deep_compress(rlp: &UntrustedRlp, swapper: &InvalidRlpSwapper) -> Option<Vec<u8>> {
	let simple_swap = ||
		swapper.get_invalid(rlp.as_raw()).map(to_elastic);
	if rlp.is_data() {
		// Try to treat the inside as RLP.
		return match rlp.payload_info() {
			// Shortest decompressed account is 70, so simply try to swap the value.
			Ok(ref p) if p.value_len < 70 => simple_swap(),
			_ => {
				if let Ok(d) = rlp.data() {
					let internal_rlp = UntrustedRlp::new(d);
					if let Some(new_d) = deep_compress(&internal_rlp, swapper) {
						// If compressed put in a special list, with first element being invalid code.
						let mut rlp = RlpStream::new_list(2);
						rlp.append_raw(&[0x81, 0x7f], 1);
						rlp.append_raw(&new_d[..], 1);
						return Some(rlp.drain());
					}
				}
				simple_swap()
			},
		};
	}
	// Iterate through RLP while checking if it has been compressed.
	map_rlp(rlp, |r| deep_compress(r, swapper))
}

/// Recover valid RLP from a compressed form, None if no decompression achieved.
/// Tries to decompress compressed data insides.
fn deep_decompress(rlp: &UntrustedRlp, swapper: &InvalidRlpSwapper) -> Option<Vec<u8>> {
	let simple_swap = ||
		swapper.get_valid(rlp.as_raw()).map(to_elastic);
	// Simply decompress data.
	if rlp.is_data() { return simple_swap(); }
	match rlp.item_count().unwrap_or(0) {
		// Look for special compressed list, which contains nested data.
		2 if rlp.at(0).map(|r| r.as_raw() == &[0x81, 0x7f]).unwrap_or(false) =>
			rlp.at(1).ok().map_or(simple_swap(),
			|r| deep_decompress(&r, swapper).map(|d| { let v = d.to_vec(); encode(&v) })),
		// Iterate through RLP while checking if it has been compressed.
		_ => map_rlp(rlp, |r| deep_decompress(r, swapper)),
  	}
}

#[cfg(feature = "std")]
impl<'a> Compressible for UntrustedRlp<'a> {
	type DataType = RlpType;

	fn compress(&self, t: RlpType) -> Vec<u8> {
		match t {
			RlpType::Snapshot => simple_compress(self, &SNAPSHOT_RLP_SWAPPER),
			RlpType::Blocks => deep_compress(self, &BLOCKS_RLP_SWAPPER).unwrap_or_else(|| to_elastic(self.as_raw())),
		}
	}

	fn decompress(&self, t: RlpType) -> Vec<u8> {
		match t {
			RlpType::Snapshot => simple_decompress(self, &SNAPSHOT_RLP_SWAPPER),
			RlpType::Blocks => deep_decompress(self, &BLOCKS_RLP_SWAPPER).unwrap_or_else(|| to_elastic(self.as_raw())),
		}
	}
}

#[cfg(test)]
mod tests {
	use compression::InvalidRlpSwapper;
	use {UntrustedRlp, Compressible, RlpType};

	#[test]
	fn invalid_rlp_swapper() {
		let to_swap: &[&[u8]] = &[&[0x83, b'c', b'a', b't'], &[0x83, b'd', b'o', b'g']];
		let invalid_rlp: &[&[u8]] = &[&[0x81, 0x00], &[0x81, 0x01]];
		let swapper = InvalidRlpSwapper::new(to_swap, invalid_rlp);
		assert_eq!(Some(invalid_rlp[0]), swapper.get_invalid(&[0x83, b'c', b'a', b't']));
		assert_eq!(None, swapper.get_invalid(&[0x83, b'b', b'a', b't']));
		assert_eq!(Some(to_swap[1]), swapper.get_valid(invalid_rlp[1]));
	}

	#[test]
	fn simple_compression() {
		let basic_account_rlp = vec![248, 68, 4, 2, 160, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 160, 197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229, 0, 182, 83, 202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112];
		let rlp = UntrustedRlp::new(&basic_account_rlp);
		let compressed = rlp.compress(RlpType::Snapshot).to_vec();
		assert_eq!(compressed, vec![198, 4, 2, 129, 0, 129, 1]);
		let compressed_rlp = UntrustedRlp::new(&compressed);
		assert_eq!(compressed_rlp.decompress(RlpType::Snapshot).to_vec(), basic_account_rlp);
	}

	#[test]
	fn data_compression() {
		let data_basic_account_rlp = vec![184, 70, 248, 68, 4, 2, 160, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 160, 197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229, 0, 182, 83, 202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112];
		let data_rlp = UntrustedRlp::new(&data_basic_account_rlp);
		let compressed = data_rlp.compress(RlpType::Blocks).to_vec();
		assert_eq!(compressed, vec![201, 129, 127, 198, 4, 2, 129, 0, 129, 1]);
		let compressed_rlp = UntrustedRlp::new(&compressed);
		assert_eq!(compressed_rlp.decompress(RlpType::Blocks).to_vec(), data_basic_account_rlp);
	}

	#[test]
	fn nested_list_rlp() {
		let nested_basic_account_rlp = vec![228, 4, 226, 2, 160, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33];
		let nested_rlp = UntrustedRlp::new(&nested_basic_account_rlp);
		let compressed = nested_rlp.compress(RlpType::Blocks).to_vec();
		assert_eq!(compressed, vec![197, 4, 195, 2, 129, 0]);
		let compressed_rlp = UntrustedRlp::new(&compressed);
		assert_eq!(compressed_rlp.decompress(RlpType::Blocks).to_vec(), nested_basic_account_rlp);
		let compressed = nested_rlp.compress(RlpType::Snapshot).to_vec();
		assert_eq!(compressed, vec![197, 4, 195, 2, 129, 0]);
		let compressed_rlp = UntrustedRlp::new(&compressed);
		assert_eq!(compressed_rlp.decompress(RlpType::Snapshot).to_vec(), nested_basic_account_rlp);
	}

	#[test]
	fn malformed_rlp() {
		let malformed = vec![248, 81, 128, 128, 128, 128, 128, 160, 12, 51, 241, 93, 69, 218, 74, 138, 79, 115, 227, 44, 216, 81, 46, 132, 85, 235, 96, 45, 252, 48, 181, 29, 75, 141, 217, 215, 86, 160, 109, 130, 160, 140, 36, 93, 200, 109, 215, 100, 241, 246, 99, 135, 92, 168, 149, 170, 114, 9, 143, 4, 93, 25, 76, 54, 176, 119, 230, 170, 154, 105, 47, 121, 10, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128];
		let malformed_rlp = UntrustedRlp::new(&malformed);
		assert_eq!(malformed_rlp.decompress(RlpType::Blocks).to_vec(), malformed);
	}
}
