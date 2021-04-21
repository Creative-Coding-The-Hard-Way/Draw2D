//! Functions to check that a set of extensions are supported by the vulkan
//! instance.

use anyhow::{bail, Result};
use ash::{version::EntryV1_0, Entry};

/// Bail if any of the required extensions is not supported by the instance.
pub fn check_extensions(
    entry: &Entry,
    required_extensions: &Vec<String>,
) -> Result<()> {
    let missing = missing_extensions(entry, required_extensions)?;
    if !missing.is_empty() {
        bail!("Some required extensions were not found!\n{:?}", missing);
    }
    Ok(())
}

/// Get a list of all extensions which are required but not available for this
/// vulkan instance.
fn missing_extensions(
    entry: &Entry,
    required_extensions: &Vec<String>,
) -> Result<Vec<String>> {
    let available_extensions =
        entry.enumerate_instance_extension_properties()?;

    let available_names: Vec<String> = available_extensions
        .iter()
        .map(|ext| {
            String::from_utf8(
                ext.extension_name.iter().map(|c| *c as u8).collect(),
            )
            .unwrap()
        })
        .collect();

    log::info!("Available extensions {}", available_names.join("\n"));

    Ok(required_extensions
        .iter()
        .cloned()
        .filter(|name| available_names.contains(name))
        .collect())
}
