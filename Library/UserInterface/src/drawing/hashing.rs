// A hasher for the library's own tables, whose keys are small numbers that
// the library makes itself: a glyph's face, number and size, a widget's Id,
// the hash of a text.
//
// Rust's HashMap hashes with SipHash unless it's told otherwise, which keeps
// keys that someone else chose from all landing in one bucket. For keys this
// short it costs more than the lookup it's for: every glyph of every frame
// is looked up in the atlas, and SipHash was a seventh of a frame's time.
// QuickHasher adds each number and multiplies, as rustc's own tables do
// (FxHash), which is a few instructions a key. Don't use it for keys that
// come from outside the program, like a file's names.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

pub type QuickMap<K, V> = HashMap<K, V, BuildHasherDefault<QuickHasher>>;
pub type QuickSet<K> = HashSet<K, BuildHasherDefault<QuickHasher>>;

#[derive(Clone, Copy, Default)]
pub struct QuickHasher {
    hash: u64,
}

// An odd number with its bits well mixed, so that multiplying by it carries
// every bit of a key into the high bits.
const MULTIPLIER: u64 = 0xf135_7aea_2e62_a9c5;

impl QuickHasher {
    #[inline]
    fn add(&mut self, number: u64) {
        self.hash = self.hash.wrapping_add(number).wrapping_mul(MULTIPLIER);
    }
}

impl Hasher for QuickHasher {
    // A multiplication only carries bits upward, so a key's high bits never
    // reach the hash's low bits, which are the ones that pick a bucket. The
    // last multiplication keeps all 128 bits of its product, and folds the
    // high half, where every bit of the key has been carried to, onto the
    // low half.
    #[inline]
    fn finish(&self) -> u64 {
        let product = self.hash as u128 * MULTIPLIER as u128;
        product as u64 ^ (product >> 64) as u64
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut number = [0; 8];
            number[..chunk.len()].copy_from_slice(chunk);
            self.add(u64::from_le_bytes(number));
        }
    }

    #[inline]
    fn write_u8(&mut self, number: u8) {
        self.add(number as u64);
    }

    #[inline]
    fn write_u16(&mut self, number: u16) {
        self.add(number as u64);
    }

    #[inline]
    fn write_u32(&mut self, number: u32) {
        self.add(number as u64);
    }

    #[inline]
    fn write_u64(&mut self, number: u64) {
        self.add(number);
    }

    #[inline]
    fn write_usize(&mut self, number: usize) {
        self.add(number as u64);
    }
}
