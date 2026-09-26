//! The scene outline: the scene's settings and a tree of its hittables.

use std::collections::BTreeSet;

use eframe::egui::{self, Color32, Id, RichText, Ui, collapsing_header::CollapsingState};

use crate::editor::label_for;
use crate::model::hittable::Hittable;
use crate::model::num::Num;
use crate::model::one_of::OneOf;
use crate::model::pos::Pos;
use crate::model::repeat::Repeat;
use crate::model::scene::Scene;

/// What the inspector shows
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Selection {
    Scene,
    Camera,
    RenderConfig,
    /// Index into the world, then into each enclosing repeat's world
    Hittable(Vec<usize>),
}

/// A change to the tree of hittables, collected while drawing and applied after.
#[derive(Clone, Debug, PartialEq)]
pub enum TreeOp {
    /// Appends to the world, or to the repeat at `parent`
    Add {
        parent: Vec<usize>,
        hittable: Box<Hittable>,
    },
    Remove(Vec<usize>),
    Duplicate(Vec<usize>),
    Move {
        path: Vec<usize>,
        delta: isize,
    },
    /// Moves the item from `from` to be the item at index `to` of the world or
    /// repeat at `parent`, where `to` counts before the move
    MoveTo {
        from: Vec<usize>,
        parent: Vec<usize>,
        to: usize,
    },
    WrapInRepeat {
        path: Vec<usize>,
        variable: String,
    },
    Unwrap(Vec<usize>),
}

/// The world, or the world of the repeat at `parent`
pub fn children<'a>(world: &'a [Hittable], parent: &[usize]) -> Option<&'a [Hittable]> {
    let mut list = world;
    for &i in parent {
        match list.get(i)? {
            Hittable::Repeat(r) => list = &r.world,
            _ => return None,
        }
    }
    Some(list)
}

pub fn children_mut<'a>(
    world: &'a mut Vec<Hittable>,
    parent: &[usize],
) -> Option<&'a mut Vec<Hittable>> {
    let mut list = world;
    for &i in parent {
        match list.get_mut(i)? {
            Hittable::Repeat(r) => list = &mut r.world,
            _ => return None,
        }
    }
    Some(list)
}

pub fn get<'a>(world: &'a [Hittable], path: &[usize]) -> Option<&'a Hittable> {
    let (last, parent) = path.split_last()?;
    children(world, parent)?.get(*last)
}

pub fn get_mut<'a>(world: &'a mut Vec<Hittable>, path: &[usize]) -> Option<&'a mut Hittable> {
    let (last, parent) = path.split_last()?;
    children_mut(world, parent)?.get_mut(*last)
}

/// Loop variables of the repeats enclosing `path`, outermost first
pub fn enclosing_loops<'a>(world: &'a [Hittable], path: &[usize]) -> Vec<&'a Repeat> {
    let mut loops = Vec::new();
    let mut list = world;
    for &i in path.iter().take(path.len().saturating_sub(1)) {
        match list.get(i) {
            Some(Hittable::Repeat(r)) => {
                loops.push(r);
                list = &r.world;
            }
            _ => break,
        }
    }
    loops
}

fn with_index(parent: &[usize], i: usize) -> Vec<usize> {
    let mut p = parent.to_vec();
    p.push(i);
    p
}

/// Applies `op`, returning what to select afterwards
pub fn apply(world: &mut Vec<Hittable>, op: TreeOp) -> Option<Selection> {
    let hittable = |p: Vec<usize>| Some(Selection::Hittable(p));
    match op {
        TreeOp::Add {
            parent,
            hittable: h,
        } => {
            let list = children_mut(world, &parent)?;
            list.push(*h);
            hittable(with_index(&parent, list.len() - 1))
        }
        TreeOp::Remove(path) => {
            let (&i, parent) = path.split_last()?;
            let list = children_mut(world, parent)?;
            if i >= list.len() {
                return None;
            }
            list.remove(i);
            if list.is_empty() {
                if parent.is_empty() {
                    Some(Selection::Scene)
                } else {
                    hittable(parent.to_vec())
                }
            } else {
                hittable(with_index(parent, i.min(list.len() - 1)))
            }
        }
        TreeOp::Duplicate(path) => {
            let (&i, parent) = path.split_last()?;
            let list = children_mut(world, parent)?;
            let copy = list.get(i)?.clone();
            list.insert(i + 1, copy);
            hittable(with_index(parent, i + 1))
        }
        TreeOp::Move { path, delta } => {
            let (&i, parent) = path.split_last()?;
            let list = children_mut(world, parent)?;
            let j = i.checked_add_signed(delta).filter(|j| *j < list.len())?;
            list.swap(i, j);
            hittable(with_index(parent, j))
        }
        TreeOp::MoveTo { from, parent, to } => {
            // Not into itself or its own children
            if parent.starts_with(&from) {
                return None;
            }
            let (&i, from_parent) = from.split_last()?;
            children_mut(world, &parent)?;
            let item = children_mut(world, from_parent)?.get(i)?.clone();
            children_mut(world, from_parent)?.remove(i);
            // Removing shifts later siblings, and the target path when it
            // runs through one of them
            let mut parent = parent;
            let mut to = to;
            if from_parent.len() < parent.len() && parent.starts_with(from_parent) {
                let k = &mut parent[from_parent.len()];
                if *k > i {
                    *k -= 1;
                }
            } else if from_parent == parent.as_slice() && to > i {
                to -= 1;
            }
            let list = children_mut(world, &parent)?;
            let to = to.min(list.len());
            list.insert(to, item);
            hittable(with_index(&parent, to))
        }
        TreeOp::WrapInRepeat { path, variable } => {
            let h = get_mut(world, &path)?;
            let inner = std::mem::replace(h, Hittable::Repeat(Repeat::default()));
            *h = Hittable::Repeat(Repeat {
                variable,
                world: vec![inner],
                ..Repeat::default()
            });
            hittable(path)
        }
        TreeOp::Unwrap(path) => {
            let (&i, parent) = path.split_last()?;
            let list = children_mut(world, parent)?;
            let Some(Hittable::Repeat(r)) = list.get(i) else {
                return None;
            };
            let inner = r.world.clone();
            let n = inner.len();
            list.splice(i..=i, inner);
            if n > 0 {
                hittable(path)
            } else if list.is_empty() {
                if parent.is_empty() {
                    Some(Selection::Scene)
                } else {
                    hittable(parent.to_vec())
                }
            } else {
                hittable(with_index(parent, i.min(list.len() - 1)))
            }
        }
    }
}

/// A loop variable name not in `taken`
pub fn free_loop_variable(taken: &BTreeSet<String>) -> String {
    ["i", "j", "k", "l", "m", "n"]
        .iter()
        .map(|s| s.to_string())
        .chain((1..).map(|n| format!("i{}", n)))
        .find(|n| !taken.contains(n))
        .expect("an unbounded list has a free name")
}

fn short(n: &Num) -> String {
    match n {
        Num::Lit(v) => format!("{}", (v * 1000.).round() / 1000.),
        Num::Expr(e) => e.src().to_string(),
    }
}

fn short_pos(p: &Pos) -> String {
    format!("{}, {}, {}", short(&p.x), short(&p.y), short(&p.z))
}

/// One line describing a hittable
pub fn summary(h: &Hittable) -> String {
    match h {
        Hittable::Sphere(s) => format!(
            "Sphere  r {}  at {}",
            short(&s.radius),
            short_pos(&s.center)
        ),
        Hittable::Quad(q) => format!("Quad  at {}", short_pos(&q.q)),
        Hittable::Box(b) => format!("Box  {}  to  {}", short_pos(&b.a), short_pos(&b.b)),
        Hittable::Model(m) => {
            if m.name.is_empty() {
                "Model".to_string()
            } else {
                format!("Model  {}", m.name)
            }
        }
        Hittable::Repeat(r) => {
            let from = r.from.as_ref().map_or("0".to_string(), short);
            let step = match &r.step {
                Some(s) => format!(" step {}", short(s)),
                None => String::new(),
            };
            format!(
                "Repeat  {} from {} to {}{}",
                r.variable,
                from,
                short(&r.to),
                step
            )
        }
    }
}

pub struct OutlineCx<'a> {
    pub selection: &'a mut Selection,
    /// The hittable a render error was traced to
    pub error: Option<&'a [usize]>,
    /// Names new loop variables must not take
    pub scene_variables: &'a BTreeSet<String>,
    pub ops: Vec<TreeOp>,
}

pub fn outline(ui: &mut Ui, scene: &Scene, cx: &mut OutlineCx) {
    for (label, sel) in [
        ("Scene", Selection::Scene),
        ("Camera", Selection::Camera),
        ("Render configuration", Selection::RenderConfig),
    ] {
        let selected = *cx.selection == sel;
        if ui.selectable_label(selected, label).clicked() {
            *cx.selection = sel;
        }
    }
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(RichText::new("World").strong());
        add_menu(ui, &[], cx);
    });
    if scene.world.is_empty() {
        ui.label(RichText::new("Empty, add a hittable with +").weak());
    }
    let mut taken = cx.scene_variables.clone();
    list(ui, &scene.world, &[], &mut taken, cx);
    drop_zone(ui, &[], scene.world.len(), cx);
}

fn add_menu(ui: &mut Ui, parent: &[usize], cx: &mut OutlineCx) {
    ui.menu_button("+", |ui| {
        for (i, name) in Hittable::variants().iter().enumerate() {
            if ui.button(label_for(name)).clicked() {
                cx.ops.push(TreeOp::Add {
                    parent: parent.to_vec(),
                    hittable: Box::new(Hittable::default_variant(i)),
                });
                ui.close();
            }
        }
    })
    .response
    .on_hover_text("Add a hittable");
}

fn list(
    ui: &mut Ui,
    items: &[Hittable],
    parent: &[usize],
    taken: &mut BTreeSet<String>,
    cx: &mut OutlineCx,
) {
    for (i, h) in items.iter().enumerate() {
        let path = with_index(parent, i);
        drop_zone(ui, parent, i, cx);
        match h {
            Hittable::Repeat(r) => {
                let id = Id::new("outline").with(&path);
                CollapsingState::load_with_default_open(ui.ctx(), id, true)
                    .show_header(ui, |ui| row(ui, h, &path, items.len(), taken, cx))
                    .body(|ui| {
                        let added = taken.insert(r.variable.clone());
                        list(ui, &r.world, &path, taken, cx);
                        drop_zone(ui, &path, r.world.len(), cx);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Add to repeat").weak());
                            add_menu(ui, &path, cx);
                        });
                        if added {
                            taken.remove(&r.variable);
                        }
                    });
            }
            _ => row(ui, h, &path, items.len(), taken, cx),
        }
    }
}

/// The payload of a drag in the outline: the path of the dragged hittable
#[derive(Clone)]
struct DragPath(Vec<usize>);

/// A thin target between rows that a dragged hittable can be dropped on
fn drop_zone(ui: &mut Ui, parent: &[usize], index: usize, cx: &mut OutlineCx) {
    let dragging = egui::DragAndDrop::has_payload_of_type::<DragPath>(ui.ctx());
    if !dragging {
        return;
    }
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.), egui::Sense::hover());
    if let Some(payload) = response.dnd_hover_payload::<DragPath>() {
        let _ = payload;
        ui.painter().hline(
            rect.x_range(),
            rect.center().y,
            egui::Stroke::new(2., ui.visuals().selection.bg_fill),
        );
    }
    if let Some(payload) = response.dnd_release_payload::<DragPath>() {
        cx.ops.push(TreeOp::MoveTo {
            from: payload.0.clone(),
            parent: parent.to_vec(),
            to: index,
        });
    }
}

fn row(
    ui: &mut Ui,
    h: &Hittable,
    path: &[usize],
    siblings: usize,
    taken: &BTreeSet<String>,
    cx: &mut OutlineCx,
) {
    let selected = *cx.selection == Selection::Hittable(path.to_vec());
    let mut text = RichText::new(summary(h));
    if cx.error == Some(path) {
        text = text.color(Color32::from_rgb(230, 80, 80));
    }
    let id = Id::new("outline-row").with(path);
    let response = ui
        .dnd_drag_source(id, DragPath(path.to_vec()), |ui| {
            ui.add(egui::Button::selectable(selected, text).truncate())
        })
        .inner;
    if response.clicked() {
        *cx.selection = Selection::Hittable(path.to_vec());
    }
    if cx.error == Some(path) {
        response
            .clone()
            .on_hover_text("The render error is in here");
    }
    let i = *path.last().expect("rows have a path");
    response.context_menu(|ui| {
        let mut op = |ui: &mut Ui, label: &str, enabled: bool, op: TreeOp| {
            if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                cx.ops.push(op);
                ui.close();
            }
        };
        op(ui, "Duplicate", true, TreeOp::Duplicate(path.to_vec()));
        op(
            ui,
            "Move up",
            i > 0,
            TreeOp::Move {
                path: path.to_vec(),
                delta: -1,
            },
        );
        op(
            ui,
            "Move down",
            i + 1 < siblings,
            TreeOp::Move {
                path: path.to_vec(),
                delta: 1,
            },
        );
        op(
            ui,
            "Wrap in repeat",
            true,
            TreeOp::WrapInRepeat {
                path: path.to_vec(),
                variable: free_loop_variable(taken),
            },
        );
        if let Hittable::Repeat(_) = h {
            op(ui, "Unwrap", true, TreeOp::Unwrap(path.to_vec()));
        }
        ui.separator();
        op(ui, "Delete", true, TreeOp::Remove(path.to_vec()));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::sphere::Sphere;

    fn sphere(r: f64) -> Hittable {
        Hittable::Sphere(Sphere {
            radius: Num::Lit(r),
            ..Sphere::default()
        })
    }

    fn radius(world: &[Hittable], path: &[usize]) -> f64 {
        match get(world, path) {
            Some(Hittable::Sphere(s)) => match s.radius {
                Num::Lit(r) => r,
                _ => panic!(),
            },
            other => panic!("{:?}", other),
        }
    }

    fn repeat(world: Vec<Hittable>) -> Hittable {
        Hittable::Repeat(Repeat {
            world,
            ..Repeat::default()
        })
    }

    #[test]
    fn add_remove_duplicate() {
        let mut w = vec![sphere(1.), repeat(vec![])];
        assert_eq!(
            Some(Selection::Hittable(vec![1, 0])),
            apply(
                &mut w,
                TreeOp::Add {
                    parent: vec![1],
                    hittable: Box::new(sphere(2.))
                }
            )
        );
        assert_eq!(2., radius(&w, &[1, 0]));
        assert_eq!(
            Some(Selection::Hittable(vec![1, 1])),
            apply(&mut w, TreeOp::Duplicate(vec![1, 0]))
        );
        assert_eq!(
            Some(Selection::Hittable(vec![1, 0])),
            apply(&mut w, TreeOp::Remove(vec![1, 1]))
        );
        assert_eq!(
            Some(Selection::Hittable(vec![1])),
            apply(&mut w, TreeOp::Remove(vec![1, 0]))
        );
        assert_eq!(None, apply(&mut w, TreeOp::Remove(vec![0, 0])));
        apply(&mut w, TreeOp::Remove(vec![1]));
        assert_eq!(
            Some(Selection::Scene),
            apply(&mut w, TreeOp::Remove(vec![0]))
        );
    }

    #[test]
    fn move_within_bounds() {
        let mut w = vec![sphere(1.), sphere(2.)];
        assert_eq!(
            None,
            apply(
                &mut w,
                TreeOp::Move {
                    path: vec![0],
                    delta: -1
                }
            )
        );
        apply(
            &mut w,
            TreeOp::Move {
                path: vec![0],
                delta: 1,
            },
        );
        assert_eq!(2., radius(&w, &[0]));
    }

    #[test]
    fn move_to_another_list() {
        let mut w = vec![sphere(1.), repeat(vec![sphere(2.)]), sphere(3.)];
        // Into the repeat, which shifts left when the first sphere is removed
        assert_eq!(
            Some(Selection::Hittable(vec![0, 1])),
            apply(
                &mut w,
                TreeOp::MoveTo {
                    from: vec![0],
                    parent: vec![1],
                    to: 1
                }
            )
        );
        assert_eq!(1., radius(&w, &[0, 1]));
        // Out of the repeat to the end of the world
        assert_eq!(
            Some(Selection::Hittable(vec![2])),
            apply(
                &mut w,
                TreeOp::MoveTo {
                    from: vec![0, 0],
                    parent: vec![],
                    to: 2
                }
            )
        );
        assert_eq!(2., radius(&w, &[2]));
        // Within a list, past itself
        let mut w = vec![sphere(1.), sphere(2.), sphere(3.)];
        apply(
            &mut w,
            TreeOp::MoveTo {
                from: vec![0],
                parent: vec![],
                to: 3,
            },
        );
        assert_eq!(1., radius(&w, &[2]));
        // Not into itself
        let mut w = vec![repeat(vec![])];
        assert_eq!(
            None,
            apply(
                &mut w,
                TreeOp::MoveTo {
                    from: vec![0],
                    parent: vec![0],
                    to: 0
                }
            )
        );
    }

    #[test]
    fn wrap_and_unwrap() {
        let mut w = vec![sphere(1.), sphere(2.)];
        apply(
            &mut w,
            TreeOp::WrapInRepeat {
                path: vec![1],
                variable: "j".to_string(),
            },
        );
        match &w[1] {
            Hittable::Repeat(r) => assert_eq!("j", r.variable),
            other => panic!("{:?}", other),
        }
        assert_eq!(2., radius(&w, &[1, 0]));
        assert_eq!(
            Some(Selection::Hittable(vec![1])),
            apply(&mut w, TreeOp::Unwrap(vec![1]))
        );
        assert_eq!(2., radius(&w, &[1]));
    }

    #[test]
    fn loop_variable_names_avoid_taken() {
        let taken: BTreeSet<String> = ["i", "j"].iter().map(|s| s.to_string()).collect();
        assert_eq!("k", free_loop_variable(&taken));
    }

    #[test]
    fn enclosing_loops_outermost_first() {
        let w = vec![repeat(vec![repeat(vec![sphere(1.)])])];
        assert_eq!(2, enclosing_loops(&w, &[0, 0, 0]).len());
        assert_eq!(0, enclosing_loops(&w, &[0]).len());
    }
}
