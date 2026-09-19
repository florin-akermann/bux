//! Bytes, written the way a class file wants them: big-endian and unpadded.

/// A growing run of bytes, with the widths a class file is made of.
#[derive(Debug, Default)]
pub(crate) struct Bytes(Vec<u8>);

impl Bytes {
    pub(crate) fn u1(&mut self, value: u8) {
        self.0.push(value);
    }

    pub(crate) fn u2(&mut self, value: u16) {
        self.0.extend_from_slice(&value.to_be_bytes());
    }

    pub(crate) fn u4(&mut self, value: u32) {
        self.0.extend_from_slice(&value.to_be_bytes());
    }

    pub(crate) fn all(&mut self, values: &[u8]) {
        self.0.extend_from_slice(values);
    }

    /// How many bytes have been written, which is the offset the next one lands at.
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// Replaces the two bytes at `at`, which is how a branch learns where it lands.
    pub(crate) fn patch_u2(&mut self, at: usize, value: u16) {
        self.0[at..at + 2].copy_from_slice(&value.to_be_bytes());
    }

    pub(crate) fn taken(self) -> Vec<u8> {
        self.0
    }
}
