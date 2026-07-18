//! The format of image flags in a `Lepton3` image

/// Flags for a Lepton3 image
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageFlags(u16);

impl ImageFlags {
    /// This image carries the debug info table
    /// which should be parsed in the image.
    pub const DEBUG_INFO: u16 = 0x0001;

    #[must_use]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn to_raw(self) -> u16 {
        self.0
    }

    /// Check if the image has a flag
    #[must_use]
    pub const fn has(self, flag: u16) -> bool {
        self.0 & flag != 0
    }

    /// Set a flag to true on the image
    pub const fn set(&mut self, flag: u16) {
        self.0 |= flag;
    }

    /// Clear a flag/set a flag to false on the image
    pub const fn clear(&mut self, flag: u16) {
        self.0 &= !flag;
    }
}
