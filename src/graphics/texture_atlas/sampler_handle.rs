/// A handle which can provide the texture index for a push constant.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SamplerHandle(u32);

impl SamplerHandle {
    /// Return the raw index for this sampler listing.
    pub(super) fn index(&self) -> u32 {
        let SamplerHandle(index) = self;
        *index
    }
}

impl Default for SamplerHandle {
    /// Return a sampler handle for the texture atlas's default sampler.
    fn default() -> Self {
        SamplerHandle(0)
    }
}
