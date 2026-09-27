//! Decoded image textures, so a build does not decode them again. It also keeps
//! an unchanged texture the same `Arc` from build to build, which is what lets
//! a running render keep its texture atlas.

use std::error::Error;
use std::fs;
use std::time::SystemTime;

use moka::policy::EvictionPolicy;
use moka::sync::Cache;
use once_cell::sync::Lazy;
use solstrale::material::texture::ImageMap;

use crate::model::ModelError;

/// Bytes of decoded texture kept. The textures of the scene being edited have
/// to fit, or every build decodes them again.
const CAPACITY: u64 = 2 << 30;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Kind {
    Color,
    Normal,
}

/// Keyed on the file's modification time too, so a texture edited on disk is
/// loaded again
static TEXTURES: Lazy<Cache<(Kind, String, SystemTime), ImageMap>> = Lazy::new(|| {
    Cache::builder()
        .weigher(|_, texture: &ImageMap| {
            u32::try_from(texture.get_image().as_raw().len()).unwrap_or(u32::MAX)
        })
        .max_capacity(CAPACITY)
        .eviction_policy(EvictionPolicy::lru())
        .build()
});

/// The texture in `path`, loaded by `load` unless it is cached. A failed load
/// is not cached.
pub fn texture(
    kind: Kind,
    path: &str,
    load: impl FnOnce(&str) -> Result<ImageMap, Box<dyn Error>>,
) -> Result<ImageMap, Box<dyn Error>> {
    let Ok(modified) = fs::metadata(path).and_then(|m| m.modified()) else {
        // Leaves the error message to the loader
        return load(path);
    };
    TEXTURES
        .try_get_with((kind, path.to_string(), modified), || {
            load(path).map_err(ModelError::new_from_err)
        })
        .map_err(|e| Box::new((*e).clone()) as Box<dyn Error>)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    fn write_png(path: &std::path::Path, width: u32) {
        image::RgbImage::new(width, 1).save(path).unwrap();
    }

    #[test]
    fn a_texture_is_decoded_once_until_its_file_changes() {
        let dir = std::env::temp_dir().join("solstrale_desktop_texture_cache_test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tex.png");
        write_png(&path, 2);
        let path = path.to_str().unwrap();

        let a = texture(Kind::Color, path, ImageMap::load).unwrap();
        let b = texture(Kind::Color, path, ImageMap::load).unwrap();
        assert!(Arc::ptr_eq(&a.get_image(), &b.get_image()));

        let normal = texture(Kind::Normal, path, ImageMap::load).unwrap();
        assert!(!Arc::ptr_eq(&a.get_image(), &normal.get_image()));

        // Past the file system's timestamp resolution
        std::thread::sleep(Duration::from_millis(20));
        write_png(std::path::Path::new(path), 3);
        let c = texture(Kind::Color, path, ImageMap::load).unwrap();
        assert_eq!(3, c.get_image().width());
    }

    #[test]
    fn a_failed_load_is_not_cached() {
        let dir = std::env::temp_dir().join("solstrale_desktop_texture_cache_test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("not-yet-an-image.png");
        fs::write(&path, "not an image").unwrap();
        let path = path.to_str().unwrap();

        assert!(texture(Kind::Color, path, ImageMap::load).is_err());
        let modified = fs::metadata(path).unwrap().modified().unwrap();
        write_png(std::path::Path::new(path), 1);
        // Even with the same modification time as the failed load
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(modified)
            .unwrap();
        assert!(texture(Kind::Color, path, ImageMap::load).is_ok());
    }

    #[test]
    fn a_missing_file_is_the_loaders_error() {
        let err = texture(Kind::Color, "/no/such/texture.png", ImageMap::load)
            .unwrap_err()
            .to_string();
        assert!(err.contains("/no/such/texture.png"), "{}", err);
    }
}
