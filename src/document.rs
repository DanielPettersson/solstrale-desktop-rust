use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use crate::DEFAULT_SCENE;
use crate::model::scene::Scene;
use crate::model::{parse_scene, scene_to_yaml};

/// The scene being edited, and the file it belongs to.
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub scene: Scene,
    pub path: Option<PathBuf>,
    /// Changed since last loaded or saved
    pub dirty: bool,
}

impl Default for Document {
    fn default() -> Self {
        Document {
            scene: parse_scene(&DEFAULT_SCENE).expect("the default scene parses"),
            path: None,
            dirty: false,
        }
    }
}

impl Document {
    pub fn load(path: &Path) -> Result<Document, Box<dyn Error>> {
        let yaml = fs::read_to_string(path)
            .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;
        let scene =
            parse_scene(&yaml).map_err(|e| format!("Could not load {}:\n{}", path.display(), e))?;
        Ok(Document {
            scene,
            path: Some(path.to_path_buf()),
            dirty: false,
        })
    }

    pub fn save_to(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        fs::write(path, scene_to_yaml(&self.scene)?)
            .map_err(|e| format!("Could not save {}: {}", path.display(), e))?;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        Ok(())
    }

    /// For the window title
    pub fn title(&self) -> String {
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map_or("Untitled".to_string(), |n| n.to_string_lossy().to_string());
        format!("{}{} - Solstrale", name, if self.dirty { " •" } else { "" })
    }

    /// The document as the app last left it. Scene text that no longer parses,
    /// e.g. a Tera template from an older version, is replaced by the default
    /// scene; it is returned too so it can be kept, with the error.
    pub fn restore(
        yaml: Option<String>,
        path: Option<String>,
        dirty: bool,
    ) -> (Document, Option<(String, String)>) {
        let Some(yaml) = yaml else {
            return (Document::default(), None);
        };
        match parse_scene(&yaml) {
            Ok(scene) => (
                Document {
                    scene,
                    path: path.filter(|p| !p.is_empty()).map(PathBuf::from),
                    dirty,
                },
                None,
            ),
            Err(e) => (Document::default(), Some((yaml, e.to_string()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join("solstrale_desktop_document_test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("scene.yaml");

        let mut doc = Document {
            dirty: true,
            ..Document::default()
        };
        doc.save_to(&path).unwrap();
        assert!(!doc.dirty);
        assert_eq!(Some(path.clone()), doc.path);
        assert_eq!("scene.yaml - Solstrale", doc.title());

        let loaded = Document::load(&path).unwrap();
        assert_eq!(doc, loaded);
    }

    #[test]
    fn load_error_names_the_file() {
        let dir = std::env::temp_dir().join("solstrale_desktop_document_test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("broken.yaml");
        fs::write(&path, "camera: [").unwrap();
        let err = Document::load(&path).unwrap_err().to_string();
        assert!(err.contains("broken.yaml"), "{}", err);
    }

    #[test]
    fn restore_falls_back_to_default() {
        let (doc, lost) = Document::restore(
            Some("{% for x in range(end=3) %}".to_string()),
            Some("/tmp/x.yaml".to_string()),
            true,
        );
        assert_eq!(Document::default(), doc);
        let (yaml, err) = lost.unwrap();
        assert!(yaml.starts_with("{%"));
        assert!(err.contains("Tera"), "{}", err);

        let (doc, lost) = Document::restore(
            Some(DEFAULT_SCENE.to_string()),
            Some("/tmp/x.yaml".to_string()),
            true,
        );
        assert!(lost.is_none());
        assert!(doc.dirty);
        assert_eq!(Some(PathBuf::from("/tmp/x.yaml")), doc.path);
    }
}
