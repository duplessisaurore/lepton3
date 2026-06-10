//! The format of image flags in a `Lepton3` image

/// Flags for a Lepton3 image
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageFlags(u16);

impl ImageFlags {
    /// This image carries debug info which should
    /// be parsed in the image.
    pub const DEBUG_INFO: u16 = 0x0001;

    #[must_use]
    pub fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    #[must_use]
    pub fn to_raw(self) -> u16 {
        self.0
    }

    #[must_use]
    pub fn has(self, flag: u16) -> bool {
        self.0 & flag != 0
    }

    pub fn set(&mut self, flag: u16) {
        self.0 |= flag;
    }

    pub fn clear(&mut self, flag: u16) {
        self.0 &= !flag;
    }
}
