/// At atlas's version changes any time that the loaded textures are changed
/// in some way.
///
/// Typically this is used to detect when the atlas needs to update as shader's
/// descriptor sets.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AtlasVersion {
    revision_count: u32,
}

impl AtlasVersion {
    /// A binding revision which will always be considered 'out_of_date'
    /// relative to the atlas.
    pub fn new_out_of_date() -> Self {
        Self { revision_count: 0 }
    }

    /// Compare a version with this one, returns true when the versions do not
    /// match.
    ///
    /// Always returns true if the version being compared against was created
    /// with `AtlasVersion::new_out_of_date`
    pub fn is_out_of_date(&self, version: &Self) -> bool {
        version.revision_count == 0
            || self.revision_count != version.revision_count
    }

    /// Create a new atlas version which is more up-to-date than the current.
    pub fn increment(&self) -> Self {
        AtlasVersion {
            revision_count: self.revision_count + 1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn same_versions_should_not_be_out_of_date() {
        let v1 = AtlasVersion { revision_count: 1 };
        let v2 = AtlasVersion { revision_count: 1 };

        assert!(!v1.is_out_of_date(&v2));
    }

    #[test]
    fn a_new_out_of_date_version_should_always_be_out_of_date() {
        let v1 = AtlasVersion { revision_count: 0 };
        let v2 = AtlasVersion::new_out_of_date();

        assert!(v1.is_out_of_date(&v2));
    }

    #[test]
    fn different_versions_should_be_out_of_date() {
        let v1 = AtlasVersion { revision_count: 1 };
        let v2 = AtlasVersion { revision_count: 2 };

        assert!(v1.is_out_of_date(&v2));
    }
}
