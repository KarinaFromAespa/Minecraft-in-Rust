//! `AreaEffectCloud` save data: reading a cloud from NBT and writing it back.

use super::{CloudSettings, PotionCloud, brewing};
use crate::game::dragon::Breath;
use horizon_client::text::Text;
use horizon_core::geometry::Vec3;
use horizon_world::nbt::Tag;

/// `AreaEffectCloud.readAdditionalSaveData` for the fields a client cloud keeps:
/// its position, lifetime, radius and use settings, and the potion it carries.
/// The owner reference is not modelled on a client cloud, and the entity id
/// is the caller's (`load_potion_cloud`).
pub(in crate::game) fn potion_cloud_from_nbt(tag: &Tag) -> Option<(Vec3, Breath)> {
    use horizon_client::save::{float, number, text};
    let at = horizon_client::entity_nbt::position(tag)?;
    let int = |key, default| number(tag.get(key)).map_or(default, |v| v as i32);
    let real = |key, default| float(tag.get(key)).map_or(default, |v| v as f32);
    let contents = tag.get("potion_contents");
    let named = contents
        .and_then(|c| text(c.get("potion")))
        .and_then(|id| brewing::potion(&id));
    let has_named = named.is_some();
    // `AreaEffectCloud.serverTick` (1.21.4): a named potion's effects apply
    // at a quarter of their duration (the `mapDuration` lambda, plain
    // integer division), while custom effects apply unscaled — there is no
    // `potionDurationScale` field at this pin, so the quarter is taken here,
    // at load, and the cloud applies its effects as stored.
    let mut effects = named.map_or_else(Vec::new, |p| {
        p.effects
            .iter()
            .map(|(id, duration, amp)| (*id, duration / 4, *amp))
            .collect()
    });
    let (mut hidden, mut ambient) = (Vec::new(), Vec::new());
    let mut custom_resolved = false;
    if let Some(Tag::List(custom)) = contents.and_then(|c| c.get("custom_effects")) {
        for effect in custom {
            let Some(def) = text(effect.get("id")).and_then(|id| brewing::effect(&id)) else {
                continue;
            };
            custom_resolved = true;
            let duration = number(effect.get("duration")).unwrap_or(0) as i32;
            let amp = number(effect.get("amplifier")).unwrap_or(0).clamp(0, 255) as i32;
            effects.push((def.id, duration, amp));
            if number(effect.get("show_particles")) == Some(0) {
                hidden.push(def.id);
            }
            if number(effect.get("ambient")).unwrap_or(0) != 0 {
                ambient.push(def.id);
            }
        }
    }
    let has_custom_color = contents.and_then(|c| c.get("custom_color")).is_some();
    // `AreaEffectCloud.updateColor`: an empty potion recolours the particle
    // to black (0), not the base potion colour `effects_color` falls back to.
    let potion_empty = !has_named && !custom_resolved && !has_custom_color;
    let color = if potion_empty {
        0
    } else {
        contents
            .and_then(|c| number(c.get("custom_color")))
            .map_or_else(
                || {
                    horizon_client_core::potions::effects_color(
                        effects
                            .iter()
                            .filter(|(id, ..)| !hidden.contains(id))
                            .map(|(id, _, amp)| (*id, *amp)),
                    )
                },
                |color| color as u32,
            )
    };
    Some((
        at,
        Breath {
            id: 0,
            at: [at.x, at.y, at.z],
            // `readAdditionalSaveData` (1.21.4) reads with plain `getInt` /
            // `getFloat`, so a missing key reads 0 — the 1.21.5 `getIntOr` /
            // `getFloatOr` defaults (radius 3.0, duration -1, wait and
            // reapplication delay 20) do not apply here.
            radius: real("Radius", 0.0).clamp(0.0, 32.0),
            age: int("Age", 0),
            duration: int("Duration", 0),
            from_dragon: false,
            victims: Vec::new(),
            potion: Some(PotionCloud {
                settings: CloudSettings {
                    wait_time: int("WaitTime", 0),
                    reapplication_delay: int("ReapplicationDelay", 0),
                    radius_on_use: real("RadiusOnUse", 0.0),
                    duration_on_use: int("DurationOnUse", 0),
                    // No `potion_duration_scale` key in 1.21.4: the quarter
                    // duration is already taken above, at load.
                    duration_scale: 1.0,
                },
                effects,
                hidden,
                ambient,
                radius_per_tick: real("RadiusPerTick", 0.0),
                color,
                particle: super::cloud_particle::load_particle(tag.get("Particle").cloned(), color),
                victims: Vec::new(),
                source: Text::translate("entity.minecraft.area_effect_cloud", &[]),
                owner: None,
                guest: None,
            }),
        },
    ))
}

/// `AreaEffectCloud.addAdditionalSaveData`: the lifetime, radius and use
/// settings with the effects the cloud applies, as `custom_effects` with the
/// colour they were drawn in. The client keeps no potion id, so a cloud that
/// came from a named potion is written effect by effect; the owner has no
/// counterpart here.
pub(in crate::game) fn potion_cloud_to_nbt(c: &Breath, uuid: [i32; 4], raw: Option<&Tag>) -> Tag {
    use horizon_world::nbt::Compound;
    let Some(potion) = &c.potion else {
        return Tag::End;
    };
    let at = Vec3::new(c.at[0], c.at[1], c.at[2]);
    let mut tag = horizon_client::entity_nbt::entity_base_tag(
        "minecraft:area_effect_cloud",
        at,
        Vec3::ZERO,
        [0.0, 0.0],
        uuid,
        raw,
    );
    let custom: Vec<Tag> = potion
        .effects
        .iter()
        .map(|(id, duration, amplifier)| {
            let mut effect = Compound::new();
            effect.insert("id".into(), Tag::String(brewing::namespaced(id).into()));
            effect.insert("duration".into(), Tag::Int(*duration));
            effect.insert("amplifier".into(), Tag::Int(*amplifier));
            if potion.hidden.contains(id) {
                effect.insert("show_particles".into(), Tag::Byte(0));
            }
            if potion.ambient.contains(id) {
                effect.insert("ambient".into(), Tag::Byte(1));
            }
            Tag::Compound(effect)
        })
        .collect();
    let mut contents = Compound::new();
    contents.insert("custom_effects".into(), Tag::List(custom));
    contents.insert("custom_color".into(), Tag::Int(potion.color as i32));
    let Tag::Compound(map) = &mut tag else {
        return tag;
    };
    map.insert("Age".into(), Tag::Int(c.age));
    map.insert("Duration".into(), Tag::Int(c.duration));
    map.insert("Radius".into(), Tag::Float(c.radius.to_bits()));
    map.insert(
        "RadiusPerTick".into(),
        Tag::Float(potion.radius_per_tick.to_bits()),
    );
    if let Some(particle) = &potion.particle {
        map.insert("Particle".into(), particle.clone());
    }
    map.insert("WaitTime".into(), Tag::Int(potion.settings.wait_time));
    map.insert(
        "ReapplicationDelay".into(),
        Tag::Int(potion.settings.reapplication_delay),
    );
    map.insert(
        "RadiusOnUse".into(),
        Tag::Float(potion.settings.radius_on_use.to_bits()),
    );
    map.insert(
        "DurationOnUse".into(),
        Tag::Int(potion.settings.duration_on_use),
    );
    // No `potion_duration_scale` key in 1.21.4: the quarter duration is
    // already in the stored effect durations (see the load side).
    // Vanilla writes the key only while the contents differ from
    // PotionContents.EMPTY: no effects and the default water colour.
    if !(potion.effects.is_empty()
        && potion.color == horizon_client_core::potions::BASE_POTION_COLOR)
    {
        map.insert("potion_contents".into(), Tag::Compound(contents));
    }
    tag
}
