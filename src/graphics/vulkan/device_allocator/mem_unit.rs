/// Units of measurement for memory.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MemUnit {
    /// Bytes
    B(u64),

    /// Kibibytes (Bytes*1024)
    KiB(u64),

    /// Mebibytes (Kibibytes*1024)
    MiB(u64),

    /// Gibibytes (Mebibytes*1024)
    GiB(u64),
}

impl MemUnit {
    const BASE: u64 = 1024;

    /// Compute the raw byte-count for the given memory unit.
    pub fn to_bytes(&self) -> u64 {
        match self {
            MemUnit::B(bytes) => *bytes,
            MemUnit::KiB(kibs) => kibs * Self::BASE,
            MemUnit::MiB(mibs) => mibs * Self::BASE.pow(2),
            MemUnit::GiB(gibs) => gibs * Self::BASE.pow(3),
        }
    }
}
