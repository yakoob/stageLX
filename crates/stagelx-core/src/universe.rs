/// Number of channels in a single DMX universe.
pub const DMX_CHANNELS: usize = 512;

/// A single 512-byte DMX universe buffer.
#[derive(Debug, Clone)]
pub struct DmxBuffer {
    data: [u8; DMX_CHANNELS],
}

impl Default for DmxBuffer {
    fn default() -> Self {
        Self { data: [0; DMX_CHANNELS] }
    }
}

impl DmxBuffer {
    /// Set channel `ch` (1-based) to `value`.
    pub fn set(&mut self, ch: u16, value: u8) {
        if let Some(slot) = ch.checked_sub(1).and_then(|i| self.data.get_mut(i as usize)) {
            *slot = value;
        }
    }

    /// Get channel `ch` (1-based).
    pub fn get(&self, ch: u16) -> u8 {
        ch.checked_sub(1)
            .and_then(|i| self.data.get(i as usize).copied())
            .unwrap_or(0)
    }

    pub fn as_bytes(&self) -> &[u8; DMX_CHANNELS] {
        &self.data
    }

    /// HTP merge: keep the higher value for each channel.
    pub fn merge_htp(&mut self, other: &DmxBuffer) {
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a = (*a).max(*b);
        }
    }

    /// LTP merge: unconditionally overwrite every channel with `other`.
    /// Zero is a valid LTP value (explicit blackout), so no zero-suppression.
    pub fn merge_ltp(&mut self, other: &DmxBuffer) {
        self.data.copy_from_slice(&other.data);
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Overwrite channels from a raw slice; zeros any channels beyond the slice length.
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        let n = src.len().min(DMX_CHANNELS);
        self.data[..n].copy_from_slice(&src[..n]);
        self.data[n..].fill(0);
    }
}

/// Manages multiple DMX universes indexed by universe number.
#[derive(Debug, Default)]
pub struct UniverseSet {
    buffers: std::collections::HashMap<u16, DmxBuffer>,
}

impl UniverseSet {
    pub fn get_or_insert(&mut self, universe: u16) -> &mut DmxBuffer {
        self.buffers.entry(universe).or_insert_with(DmxBuffer::default)
    }

    pub fn get(&self, universe: u16) -> Option<&DmxBuffer> {
        self.buffers.get(&universe)
    }

    pub fn universes(&self) -> impl Iterator<Item = (u16, &DmxBuffer)> {
        self.buffers.iter().map(|(id, buf)| (*id, buf))
    }
}
