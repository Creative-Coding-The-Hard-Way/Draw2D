//! Functions to check if a set of vulkan layers are available for the
//! instance.

use anyhow::{bail, Result};
use ash::{version::EntryV1_0, Entry};

/// Bail if any of the required layers is not supported by the instance.
pub fn check_layers(
    entry: &Entry,
    required_layers: &Vec<String>,
) -> Result<()> {
    let missing = missing_layers(entry, required_layers)?;
    if !missing.is_empty() {
        bail!("some required layers were not found!\n{:?}", missing);
    }
    Ok(())
}

/// Get a list of all layers which are required but not avaialable for this
/// vulkan instance.
fn missing_layers(
    entry: &Entry,
    required_layers: &Vec<String>,
) -> Result<Vec<String>> {
    let available_layer_properties =
        entry.enumerate_instance_layer_properties()?;

    let available_names: Vec<String> = available_layer_properties
        .iter()
        .map(|layer| {
            String::from_utf8(
                layer.layer_name.iter().map(|c| *c as u8).collect(),
            )
            .unwrap()
        })
        .collect();

    log::info!("Available layers {}", available_names.join("\n"));

    Ok(required_layers
        .iter()
        .cloned()
        .filter(|name| available_names.contains(name))
        .collect())
}
