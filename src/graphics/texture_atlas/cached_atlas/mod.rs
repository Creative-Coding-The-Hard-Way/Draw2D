mod cached_atlas;

#[cfg(test)]
pub mod stub_atlas;

use std::{collections::HashMap, path::PathBuf};

use super::{TextureAtlas, TextureHandle};

/// A cached atlas keeps track of which files have already been loaded and
/// prevents any single file from being loaded more than once.
pub struct CachedAtlas<Atlas: TextureAtlas> {
    atlas: Atlas,
    cache: HashMap<PathBuf, TextureHandle>,
}
