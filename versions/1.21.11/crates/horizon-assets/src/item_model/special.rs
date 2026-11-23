//! `special` item models: the block-entity style renderers an item can ask
//! for (`SpecialModelRenderers`), their codec fields and what each reads from
//! the stack (`extractArgument`). 1.21.11's registry holds 13 types; the
//! 26.x bell, book and end_cube kinds are absent, so a model naming one
//! fails to decode here, as it does in vanilla. Bed, standing-sign and
//! hanging-sign types exist in 1.21.11's registry but have no kind here yet:
//! a model naming one fails to decode and the item falls back to the missing
//! model (a known gap, not a 1.21.11 behaviour).
use super::{Result, float, get, ident, object, required, string};
use crate::json::error as err;
use serde_json::{Map, Value, json};

#[derive(Clone, Debug, PartialEq)]
pub enum SpecialKind {
    Banner {
        color: String,
    },
    Conduit,
    Chest {
        texture: String,
        openness: f32,
    },
    CopperGolemStatue {
        texture: String,
        pose: String,
    },
    DecoratedPot,
    Head {
        kind: String,
        texture: Option<String>,
        animation: f32,
    },
    PlayerHead,
    ShulkerBox {
        texture: String,
        openness: f32,
        orientation: String,
    },
    Shield,
    Trident,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SpecialModel {
    pub kind: SpecialKind,
}

const DYES: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];
fn one_of(v: &Value, names: &[&str]) -> Result<String> {
    let s = string(v)?;
    if names.contains(&s.as_str()) {
        Ok(s)
    } else {
        Err(err("invalid enum name"))
    }
}

impl SpecialModel {
    /// A special model type of another namespace, which Horizon has no renderer for.
    pub fn is_foreign(v: &Value) -> bool {
        v.get("type")
            .and_then(Value::as_str)
            .and_then(|t| super::identifier(t).ok())
            .is_some_and(|t| super::foreign::is_foreign(&t))
    }
    pub fn decode(v: &Value) -> Result<Self> {
        let m = object(v)?;
        let float_or = |key: &str, default: f32| get(m, key).map_or(Ok(default), float);
        let kind = match ident(required(m, "type")?)?.as_str() {
            // 1.21.11's banner codec holds only `color`: the 26.x `attachment`
            // field is gone (`BannerRenderer.submitSpecial` takes no
            // attachment), so a model naming one renders standing.
            "minecraft:banner" => SpecialKind::Banner {
                color: one_of(required(m, "color")?, &DYES)?,
            },
            "minecraft:conduit" => SpecialKind::Conduit,
            // 1.21.11's chest codec holds only `texture` and `openness`:
            // the 26.x `chest_type` field is gone (bake always uses
            // `ModelLayers.CHEST`), so a model naming one renders single.
            "minecraft:chest" => SpecialKind::Chest {
                texture: ident(required(m, "texture")?)?,
                openness: float_or("openness", 0.0)?,
            },
            "minecraft:copper_golem_statue" => SpecialKind::CopperGolemStatue {
                texture: ident(required(m, "texture")?)?,
                pose: one_of(
                    required(m, "pose")?,
                    &["standing", "sitting", "running", "star"],
                )?,
            },
            "minecraft:decorated_pot" => SpecialKind::DecoratedPot,
            "minecraft:head" => SpecialKind::Head {
                kind: one_of(
                    required(m, "kind")?,
                    &[
                        "skeleton",
                        "wither_skeleton",
                        "player",
                        "zombie",
                        "creeper",
                        "piglin",
                        "dragon",
                    ],
                )?,
                texture: get(m, "texture").map(ident).transpose()?,
                animation: float_or("animation", 0.0)?,
            },
            "minecraft:player_head" => SpecialKind::PlayerHead,
            "minecraft:shulker_box" => SpecialKind::ShulkerBox {
                texture: ident(required(m, "texture")?)?,
                openness: float_or("openness", 0.0)?,
                // 1.21.11 added `orientation` (default up); 26.x had none.
                orientation: match get(m, "orientation") {
                    None => "up".to_owned(),
                    Some(o) => one_of(o, &["down", "up", "north", "south", "west", "east"])?,
                },
            },
            "minecraft:shield" => SpecialKind::Shield,
            "minecraft:trident" => SpecialKind::Trident,
            _ => return Err(err("unknown special model type")),
        };
        Ok(Self { kind })
    }
    /// The codec's encoding: optional fields equal to their default are omitted.
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        let mut put = |k: &str, v: Value| {
            m.insert(k.to_owned(), v);
        };
        let ty = match &self.kind {
            SpecialKind::Banner { color } => {
                put("color", json!(color));
                "banner"
            }
            SpecialKind::Conduit => "conduit",
            SpecialKind::Chest { texture, openness } => {
                put("texture", json!(texture));
                if *openness != 0.0 {
                    put("openness", json!(openness));
                }
                "chest"
            }
            SpecialKind::CopperGolemStatue { texture, pose } => {
                put("texture", json!(texture));
                put("pose", json!(pose));
                "copper_golem_statue"
            }
            SpecialKind::DecoratedPot => "decorated_pot",
            SpecialKind::Head {
                kind,
                texture,
                animation,
            } => {
                put("kind", json!(kind));
                if let Some(t) = texture {
                    put("texture", json!(t));
                }
                if *animation != 0.0 {
                    put("animation", json!(animation));
                }
                "head"
            }
            SpecialKind::PlayerHead => "player_head",
            SpecialKind::ShulkerBox {
                texture,
                openness,
                orientation,
            } => {
                put("texture", json!(texture));
                if *openness != 0.0 {
                    put("openness", json!(openness));
                }
                if orientation != "up" {
                    put("orientation", json!(orientation));
                }
                "shulker_box"
            }
            SpecialKind::Shield => "shield",
            SpecialKind::Trident => "trident",
        };
        m.insert("type".to_owned(), json!(format!("minecraft:{ty}")));
        Value::Object(m)
    }
    pub fn type_name(&self) -> String {
        self.to_json()["type"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    }
    /// What `extractArgument` reads: banners their pattern layers, shields
    /// their base colour and layers, decorated pots their sherds and player
    /// heads their profile. Absent components give `null`.
    pub fn argument(&self, component: &dyn Fn(&str) -> Option<Value>) -> Value {
        match &self.kind {
            SpecialKind::Banner { .. } => {
                component("minecraft:banner_patterns").unwrap_or(Value::Null)
            }
            SpecialKind::DecoratedPot => {
                component("minecraft:pot_decorations").unwrap_or(Value::Null)
            }
            SpecialKind::PlayerHead => component("minecraft:profile").unwrap_or(Value::Null),
            SpecialKind::Shield => {
                let mut m = Map::new();
                for key in ["minecraft:banner_patterns", "minecraft:base_color"] {
                    if let Some(v) = component(key) {
                        m.insert(key.to_owned(), v);
                    }
                }
                Value::Object(m)
            }
            _ => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_and_chest_ignore_the_fields_1_21_11_dropped() {
        // 26.x codecs held `attachment` (banner) and `chest_type` (chest);
        // 1.21.11's hold neither, so a model naming one decodes as if the
        // field were absent: standing banner, single chest.
        let banner = SpecialModel::decode(&serde_json::json!({
            "type": "minecraft:banner",
            "color": "red",
            "attachment": "wall",
        }))
        .unwrap();
        assert_eq!(
            banner.kind,
            SpecialKind::Banner {
                color: "red".to_owned()
            }
        );
        assert_eq!(
            banner.to_json(),
            serde_json::json!({"type": "minecraft:banner", "color": "red"})
        );
        let chest = SpecialModel::decode(&serde_json::json!({
            "type": "minecraft:chest",
            "texture": "minecraft:normal",
            "chest_type": "left",
        }))
        .unwrap();
        assert_eq!(
            chest.kind,
            SpecialKind::Chest {
                texture: "minecraft:normal".to_owned(),
                openness: 0.0,
            }
        );
        assert_eq!(
            chest.to_json(),
            serde_json::json!({"type": "minecraft:chest", "texture": "minecraft:normal"})
        );
    }

    #[test]
    fn shulker_orientation_defaults_to_up_and_round_trips() {
        let plain = SpecialModel::decode(&serde_json::json!({
            "type": "minecraft:shulker_box",
            "texture": "minecraft:shulker_white",
        }))
        .unwrap();
        assert_eq!(
            plain.kind,
            SpecialKind::ShulkerBox {
                texture: "minecraft:shulker_white".to_owned(),
                openness: 0.0,
                orientation: "up".to_owned(),
            }
        );
        assert_eq!(
            plain.to_json(),
            serde_json::json!({
                "type": "minecraft:shulker_box",
                "texture": "minecraft:shulker_white",
            })
        );
        let east = SpecialModel::decode(&serde_json::json!({
            "type": "minecraft:shulker_box",
            "texture": "minecraft:shulker_white",
            "orientation": "east",
        }))
        .unwrap();
        assert_eq!(
            east.kind,
            SpecialKind::ShulkerBox {
                texture: "minecraft:shulker_white".to_owned(),
                openness: 0.0,
                orientation: "east".to_owned(),
            }
        );
        assert_eq!(east.to_json()["orientation"], serde_json::json!("east"));
        assert!(
            SpecialModel::decode(&serde_json::json!({
                "type": "minecraft:shulker_box",
                "texture": "minecraft:shulker_white",
                "orientation": "sideways",
            }))
            .is_err()
        );
    }
}
