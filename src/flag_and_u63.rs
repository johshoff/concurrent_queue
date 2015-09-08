
pub struct FlagAndU63 {
    combined: u64, // highest value bit is boolean, remaining 63 bits is u63 value
}

const FLAG_VALUE: u64 = 1 << 63;

impl FlagAndU63 {
    pub fn new(flag: bool, value: u64) -> FlagAndU63 {
        if flag { FlagAndU63 { combined: value | FLAG_VALUE } }
        else    { FlagAndU63 { combined: value } }
    }

    /// Create a FlagAndU63 from the internal representation of one
    pub fn from_repr(repr: u64) -> FlagAndU63 {
        FlagAndU63 { combined: repr }
    }

    pub fn is_flag_set(&self) -> bool {
        self.combined & FLAG_VALUE > 0
    }

    pub fn value(&self) -> u64 {
        self.combined & !FLAG_VALUE
    }

    /// Get both values is (hopefully) one memory read
    pub fn flag_and_value(&self) -> (bool, u64) {
        let current_combined = self.combined;

        (current_combined & FLAG_VALUE > 0, current_combined & !FLAG_VALUE)
    }

    pub fn set_flag(&mut self) {
        self.combined |= FLAG_VALUE;
    }

    pub fn unset_flag(&mut self) {
        self.combined &= !FLAG_VALUE;
    }

    /// Return a reference to the combined representation of flag+u63
    pub fn ref_combined(&self) -> &u64 {
        &self.combined
    }

    /// Return internal combined representation of flag+u63
    pub fn combined(&self) -> u64 {
        self.combined
    }
}

