use crate::model::material::Material;
use crate::model::num::{VisitNums, visit_nums};
use crate::model::transformation::{Transformation, create_transformation, rotates_off_axis};
use crate::model::{Creator, CreatorContext, ModelError};
use moka::policy::EvictionPolicy;
use moka::sync::Cache;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solstrale::geo::transformation::NopTransformer;
use solstrale::hittable::{Bvh, Hittables};
use solstrale::loader::Loader;
use solstrale::loader::obj::Obj;
use solstrale::material::texture::SolidColor;
use std::error::Error;

/// Triangles kept per tier. The models of the scene being edited have to fit,
/// or every build loads them again: San Miguel is 10M.
const CAPACITY: u64 = 16_000_000;

fn cache<V: Clone + Send + Sync + 'static>(
    triangles: impl Fn(&V) -> usize + Send + Sync + 'static,
) -> Cache<String, V> {
    Cache::builder()
        .weigher(move |_, v| u32::try_from(triangles(v)).unwrap_or(u32::MAX))
        .max_capacity(CAPACITY)
        // A drag leaves a trail of models where they no longer are, which have
        // to go before the ones still in the scene
        .eviction_policy(EvictionPolicy::lru())
        .build()
}

/// Models as loaded, before their transformations, so moving one re-bakes it
/// rather than parsing its file again
static OBJECT_SPACE: Lazy<Cache<String, Bvh>> = Lazy::new(|| cache(Bvh::primitive_count));

/// Models where the scene has them. An untouched model is the same tree on
/// every build, which is what lets a running render keep it.
static WORLD_SPACE: Lazy<Cache<String, Placed>> =
    Lazy::new(|| cache(|p: &Placed| p.bvh.primitive_count()));

#[derive(Clone)]
struct Placed {
    bvh: Bvh,
    /// Refitted after turning off its axes, and so looser than built again
    loose: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ObjModel {
    pub path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transformations: Vec<Transformation>,
}

visit_nums!(ObjModel, material, transformations);

impl ObjModel {
    /// The material is baked in on load, so it is part of the key
    fn object_space_key(&self, ctx: &CreatorContext) -> String {
        format!(
            "{:?} {:?} {:?} {}",
            self.path,
            self.name,
            self.material,
            ctx.scope.fingerprint(&self.material.free_vars())
        )
    }

    /// The same model can evaluate differently, e.g. in each iteration of a
    /// repeat, so the key includes the values of the variables it reads.
    fn world_space_key(&self, ctx: &CreatorContext) -> String {
        format!("{:?} {}", self, ctx.scope.fingerprint(&self.free_vars()))
    }

    fn object_space(&self, ctx: &CreatorContext) -> Result<Bvh, Box<dyn Error>> {
        OBJECT_SPACE
            .try_get_with(self.object_space_key(ctx), || {
                let material = self.material.as_ref().map_or(
                    Ok(solstrale::material::Lambertian::new(
                        SolidColor::new(1., 1., 1.).into(),
                        None,
                    )
                    .into()),
                    |m| m.create(ctx),
                );
                material
                    .and_then(|m| Obj::new(&self.path, &self.name).load(&NopTransformer(), Some(m)))
                    .map_err(ModelError::new_from_err)
            })
            .map_err(|e| Box::new((*e).clone()) as Box<dyn Error>)
    }
}

impl Creator<Hittables> for ObjModel {
    fn create(&self, ctx: &CreatorContext) -> Result<Hittables, Box<dyn Error>> {
        let key = self.world_space_key(ctx);
        if let Some(placed) = WORLD_SPACE.get(&key)
            && (ctx.refit_models || !placed.loose)
        {
            return Ok(placed.bvh.into());
        }

        let object_space = self.object_space(ctx)?;
        let placed = if self.transformations.is_empty() {
            Placed {
                bvh: object_space,
                loose: false,
            }
        } else {
            let bvh = object_space.transformed(create_transformation(&self.transformations, ctx)?);
            let loose = rotates_off_axis(&self.transformations, ctx)?;
            if loose && !ctx.refit_models {
                Placed {
                    bvh: bvh.rebuilt(),
                    loose: false,
                }
            } else {
                Placed { bvh, loose }
            }
        };
        WORLD_SPACE.insert(key, placed.clone());
        Ok(placed.bvh.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::num::Num;
    use crate::model::pos::Pos;
    use crate::model::scope::Scope;
    use solstrale::hittable::Hittable as _;
    use solstrale::util::wgpu_util::get_wgpu_device_and_queue;
    use std::path::PathBuf;

    /// A one triangle model in a file of its own, as tests share the caches
    fn model(name: &str) -> (ObjModel, PathBuf) {
        let dir = std::env::temp_dir().join("solstrale_desktop_obj_model_test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join(name);
        std::fs::write(&file, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
        let model = ObjModel {
            path: format!("{}/", dir.display()),
            name: name.to_string(),
            ..Default::default()
        };
        (model, file)
    }

    fn at(model: &ObjModel, transformations: Vec<Transformation>) -> ObjModel {
        ObjModel {
            transformations,
            ..model.clone()
        }
    }

    fn moved(model: &ObjModel, x: f64) -> ObjModel {
        at(
            model,
            vec![Transformation::Translation(Pos::new(x, 0., 0.))],
        )
    }

    fn with_ctx<T>(refit_models: bool, f: impl FnOnce(&CreatorContext) -> T) -> T {
        let (device, queue) = get_wgpu_device_and_queue();
        f(&CreatorContext {
            screen_width: 100,
            screen_height: 100,
            device,
            queue,
            scope: &Scope::builtin(0),
            refit_models,
        })
    }

    fn min_x(model: &ObjModel) -> Result<f64, String> {
        with_ctx(true, |ctx| model.create(ctx))
            .map(|h| h.bounding_box().x.min)
            .map_err(|e| e.to_string())
    }

    #[test]
    fn a_moved_model_is_baked_from_the_model_as_loaded() {
        let (model, file) = model("moved.obj");
        assert_eq!(Ok(0.), min_x(&moved(&model, 0.)));
        std::fs::remove_file(file).unwrap();

        let x = min_x(&moved(&model, 10.)).unwrap();
        assert!((x - 10.).abs() < 1e-6, "{}", x);

        // Its material is baked in when loaded
        let other_material = ObjModel {
            material: Some(Material::default()),
            ..model
        };
        assert!(min_x(&other_material).is_err());
    }

    #[test]
    fn an_untouched_model_is_not_baked_again() {
        let (model, file) = model("untouched.obj");
        let model = moved(&model, 5.);
        min_x(&model).unwrap();
        with_ctx(true, |ctx| {
            OBJECT_SPACE.invalidate(&model.object_space_key(ctx))
        });
        std::fs::remove_file(file).unwrap();

        assert!(min_x(&model).is_ok());
        assert!(min_x(&moved(&model, 6.)).is_err());
    }

    #[test]
    fn a_failed_load_is_not_cached() {
        let (model, file) = model("appears.obj");
        std::fs::remove_file(&file).unwrap();
        assert!(min_x(&model).is_err());

        std::fs::write(&file, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
        assert_eq!(Ok(0.), min_x(&model));
    }

    #[test]
    fn a_model_turned_off_its_axes_is_built_again_unless_refitted() {
        let (model, _) = model("turned.obj");
        let loose = |model: &ObjModel, refit_models: bool| {
            with_ctx(refit_models, |ctx| {
                model.create(ctx).unwrap();
                WORLD_SPACE.get(&model.world_space_key(ctx)).unwrap().loose
            })
        };
        let turned = at(&model, vec![Transformation::RotationY(Num::Lit(30.))]);
        assert!(loose(&turned, true));
        assert!(!loose(&turned, false));
        // Taken as it is once built again
        assert!(!loose(&turned, true));

        let quarter = at(&model, vec![Transformation::RotationY(Num::Lit(-90.))]);
        assert!(!loose(&quarter, true));
    }
}
