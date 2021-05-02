use super::AtlasVersion;

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
}

#[cfg(test)]
mod test {
    use crate::texture_atlas::AtlasVersion;

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
