//! Decides when edits to the scene restart the render, and how.

use std::time::{Duration, Instant};

use crate::editor::outline::Selection;
use crate::model::scene::Scene;

/// Edits closer together than this are rendered together
pub const DEBOUNCE: Duration = Duration::from_millis(250);

#[derive(Debug, PartialEq)]
pub enum Change {
    None,
    /// Only the camera changed, which the running render can take as is
    Camera,
    /// The scene has to be built again
    Full,
}

/// What an edit changed, compared to the scene and frame last rendered
pub fn classify(last: Option<&(Scene, usize)>, scene: &Scene, frame: usize) -> Change {
    match last {
        None => Change::Full,
        Some((_, f)) if *f != frame => Change::Full,
        Some((s, _)) if s == scene => Change::None,
        Some((s, _)) if eq_ignoring_camera(s, scene) => Change::Camera,
        Some(_) => Change::Full,
    }
}

fn eq_ignoring_camera(a: &Scene, b: &Scene) -> bool {
    a.variables == b.variables
        && a.render_configuration == b.render_configuration
        && a.background_color == b.background_color
        && a.world == b.world
}

/// Time left before edits made at `last_edit` should be acted on
pub fn debounce_remaining(last_edit: Instant, now: Instant) -> Option<Duration> {
    DEBOUNCE
        .checked_sub(now.duration_since(last_edit))
        .filter(|d| !d.is_zero())
}

/// The part of the scene an error path points to, e.g.
/// `["world[1]", "repeat (i = 2)", "world[0]", "sphere"]` is the first
/// hittable in the repeat that is the second in the world.
pub fn error_selection(path: &[String]) -> Option<Selection> {
    match path.first().map(String::as_str) {
        Some("camera") => return Some(Selection::Camera),
        Some("render_configuration") => return Some(Selection::RenderConfig),
        Some("variables") | Some("background_color") => return Some(Selection::Scene),
        _ => {}
    }
    let indices: Vec<usize> = path
        .iter()
        .filter_map(|s| s.strip_prefix("world[")?.strip_suffix(']')?.parse().ok())
        .collect();
    (!indices.is_empty()).then_some(Selection::Hittable(indices))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::model::num::Num;
    use crate::model::pos::Pos;

    #[test]
    fn classifies_what_changed() {
        let scene = Document::default().scene;
        let last = (scene.clone(), 0);
        assert_eq!(Change::Full, classify(None, &scene, 0));
        assert_eq!(Change::None, classify(Some(&last), &scene, 0));
        assert_eq!(Change::Full, classify(Some(&last), &scene, 1));

        let mut camera_moved = scene.clone();
        camera_moved.camera.look_from = Pos::new(1., 2., 3.);
        assert_eq!(Change::Camera, classify(Some(&last), &camera_moved, 0));

        let mut world_changed = scene.clone();
        world_changed.world.pop();
        assert_eq!(Change::Full, classify(Some(&last), &world_changed, 0));

        let mut variable_added = scene.clone();
        variable_added
            .variables
            .0
            .push(("a".to_string(), Num::Lit(1.)));
        assert_eq!(Change::Full, classify(Some(&last), &variable_added, 0));
    }

    fn control_after_render(scene: &Scene) -> crate::RenderControl {
        crate::RenderControl {
            last_dispatched: Some((scene.clone(), 0)),
            ..Default::default()
        }
    }

    #[test]
    fn edits_restart_the_render_after_a_pause() {
        let scene = Document::default().scene;
        let mut rc = control_after_render(&scene);
        let t = Instant::now();
        assert_eq!(None, rc.schedule(&scene, 0, t));

        let mut edited = scene.clone();
        edited.world.pop();
        rc.edited(t);
        assert_eq!(Some(DEBOUNCE), rc.schedule(&edited, 0, t));
        assert!(!rc.render_requested);

        assert_eq!(None, rc.schedule(&edited, 0, t + DEBOUNCE));
        assert!(rc.render_requested);
        assert!(rc.last_edit.is_none());
    }

    #[test]
    fn a_build_in_flight_holds_back_the_next() {
        let scene = Document::default().scene;
        let mut rc = control_after_render(&scene);
        rc.build_in_flight = true;
        let mut edited = scene.clone();
        edited.world.pop();
        let t = Instant::now();
        rc.edited(t);
        assert!(rc.schedule(&edited, 0, t + DEBOUNCE).is_some());
        assert!(!rc.render_requested);

        rc.build_in_flight = false;
        rc.schedule(&edited, 0, t + DEBOUNCE);
        assert!(rc.render_requested);
    }

    #[test]
    fn camera_edits_go_to_the_running_render_at_once() {
        let scene = Document::default().scene;
        let mut rc = control_after_render(&scene);
        let (abort_sender, _abort_receiver) = std::sync::mpsc::channel();
        rc.abort_sender = Some(abort_sender);

        let mut moved = scene.clone();
        moved.camera.look_from = Pos::new(1., 2., 3.);
        let t = Instant::now();
        rc.edited(t);
        assert_eq!(None, rc.schedule(&moved, 0, t));
        assert!(!rc.render_requested);
        assert!(rc.camera_updated);
        assert_eq!(
            solstrale::geo::vec3::Vec3::new(1., 2., 3.),
            rc.applied_camera.unwrap().look_from
        );
        assert_eq!(Some(&(moved, 0)), rc.last_dispatched.as_ref());
    }

    #[test]
    fn camera_edits_without_a_render_start_one() {
        let scene = Document::default().scene;
        let mut rc = control_after_render(&scene);
        let mut moved = scene.clone();
        moved.camera.look_from = Pos::new(1., 2., 3.);
        let t = Instant::now();
        rc.edited(t);
        rc.schedule(&moved, 0, t + DEBOUNCE);
        assert!(rc.render_requested);
        assert!(!rc.camera_updated);
    }

    #[test]
    fn debounce_waits_for_a_pause() {
        let t = Instant::now();
        assert_eq!(Some(DEBOUNCE), debounce_remaining(t, t));
        assert_eq!(
            Some(Duration::from_millis(150)),
            debounce_remaining(t, t + Duration::from_millis(100))
        );
        assert_eq!(None, debounce_remaining(t, t + DEBOUNCE));
        assert_eq!(None, debounce_remaining(t, t + Duration::from_secs(1)));
    }

    #[test]
    fn error_paths_point_into_the_scene() {
        let path = |p: &[&str]| p.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            Some(Selection::Hittable(vec![1, 0])),
            error_selection(&path(&["world[1]", "repeat (i = 2)", "world[0]", "sphere"]))
        );
        assert_eq!(Some(Selection::Camera), error_selection(&path(&["camera"])));
        assert_eq!(
            Some(Selection::Scene),
            error_selection(&path(&["variables", "a"]))
        );
        assert_eq!(None, error_selection(&[]));
    }
}
