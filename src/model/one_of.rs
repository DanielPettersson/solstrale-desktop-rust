/// Strips the `r#` prefix `stringify!` leaves on raw identifiers.
pub const fn strip_raw(s: &'static str) -> &'static str {
    match s.as_bytes() {
        [b'r', b'#', rest @ ..] => match std::str::from_utf8(rest) {
            Ok(r) => r,
            Err(_) => s,
        },
        _ => s,
    }
}

/// Declares an enum that reads and writes as a map with exactly one key, e.g.
/// `sphere: {...}`.
///
/// A derived enum does not do this in serde_yaml 0.9, which writes variants as
/// YAML tags (`!sphere`) and refuses untagged maps. So the enum goes through a
/// struct of `Option`s on both sides, which also keeps `deny_unknown_fields`
/// errors for misspelled keys. `empty` is what a map with no key reads as, or
/// `None` to reject it.
macro_rules! one_of {
    (
        $(#[$attr:meta])*
        pub enum $name:ident {
            $($key:ident => $variant:ident($ty:ty)),+ $(,)?
        }
        repr: $repr:ident, $repr_ref:ident;
        empty: $empty:expr;
    ) => {
        $(#[$attr])*
        pub enum $name {
            $($variant($ty)),+
        }

        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct $repr {
            $($key: Option<$ty>),+
        }

        #[derive(serde::Serialize, Default)]
        struct $repr_ref<'a> {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                $key: Option<&'a $ty>
            ),+
        }

        impl $name {
            pub const VARIANTS: &'static [&'static str] =
                &[$($crate::model::one_of::strip_raw(stringify!($key))),+];
        }

        impl TryFrom<$repr> for $name {
            type Error = String;

            fn try_from(r: $repr) -> Result<Self, String> {
                let mut found: Vec<&'static str> = Vec::new();
                let mut value = None;
                $(
                    if let Some(v) = r.$key {
                        found.push($crate::model::one_of::strip_raw(stringify!($key)));
                        value = Some($name::$variant(v));
                    }
                )+
                let expected = || {
                    $name::VARIANTS
                        .iter()
                        .map(|v| format!("`{}`", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                match (found.len(), value) {
                    (1, Some(v)) => Ok(v),
                    (0, _) => {
                        let empty: Option<$name> = $empty;
                        empty.ok_or_else(|| format!("expected one of {}", expected()))
                    }
                    _ => Err(format!(
                        "expected only one of {}, found {}",
                        expected(),
                        found
                            .iter()
                            .map(|v| format!("`{}`", v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let r = <$repr as serde::Deserialize>::deserialize(d)?;
                $name::try_from(r).map_err(serde::de::Error::custom)
            }
        }

        impl $crate::model::num::VisitNums for $name {
            fn visit_nums(&self, f: &mut dyn FnMut(&$crate::model::num::Num)) {
                match self {
                    $($name::$variant(v) => $crate::model::num::VisitNums::visit_nums(v, f)),+
                }
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                let mut r = $repr_ref::default();
                match self {
                    $($name::$variant(v) => r.$key = Some(v)),+
                }
                serde::Serialize::serialize(&r, s)
            }
        }
    };
}

pub(crate) use one_of;
