use crc16;

use std::hash::{Hash, Hasher};

pub struct CrcHasher {
    state: crc16::State<crc16::XMODEM>,
}

impl Hasher for CrcHasher {
    fn finish(&self) -> u64 {
        self.state.get() as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        self.state.update(bytes);
    }
}

impl Default for CrcHasher {
    fn default() -> Self {
        CrcHasher {
            state: crc16::State::new(),
        }
    }
}

// Ring is a consist hash map
pub struct Ring<T: Sync> {
    node: Vec<T>,
}

impl<T: Sync> Ring<T> {
    pub fn new(node: Vec<T>) -> Ring<T> {
        Ring { node: node }
    }

    pub fn get<K: Hash>(&self, key: K) -> &T {
        let mut hasher = CrcHasher::default();
        key.hash(&mut hasher);
        let pos = hasher.finish() as usize % self.node.len();
        self.node.get(pos).unwrap()
    }
}

#[cfg(test)]
mod ring_test {
    use self::super::*;
    use std::mem;
    use std::sync::Arc;
    use test::Bencher;

    #[bench]
    fn test(b: &mut Bencher) {
        let node: Vec<_> = (0..100).map(|idx| format!("{}", idx)).collect();
        let ring = Ring::new(node);
        let keylist: Vec<_> = (0..10000)
            .into_iter()
            .map(|idx| format!("{}", idx))
            .collect();
        let ring = Arc::new(ring);

        b.iter(|| {
            keylist.iter().map(|key| ring.get(key)).for_each(mem::drop);
        });
    }

    #[test]
    fn test_crc_hasher_equal() {
        let data = b"equal";

        let mut first = CrcHasher::default();
        first.write(data);
        let mut second = CrcHasher::default();
        second.write(data);
        assert_eq!(first.finish(), second.finish());
    }

}
