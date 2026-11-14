//! Entity fluid immersion trackers and current accumulation across update ticks.
//!
//! The 26.2 `EntityFluidInteraction`: one `Tracker` per tag, holding the
//! tag's immersion height, whether the eyes are in it, and the accumulated
//! current the tag's flow pushed. A fluid whose kind no tag holds is skipped
//! entirely; the trackers persist across ticks (26.3 split the tracker per
//! fluid kind, moved the current into a separate per-tag accumulator, and
//! pruned empty trackers on reset).
use crate::fluid::{FluidKind, FluidState};
use horizon_core::{
    geometry::{Aabb, Vec3},
    position::BlockPos,
};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FluidTagId(pub u32);
pub trait FluidMembership {
    fn fluid_in_tag(&self, fluid: FluidKind, tag: FluidTagId) -> bool;
}

/// Where the tags of the mod fluids start: one each, past any tag a caller
/// numbers itself, holding that fluid's source and flowing forms.
const MOD_TAGS: u32 = 1 << 16;

impl FluidTagId {
    /// The tag that holds just this mod fluid (none for vanilla's).
    pub fn of_mod_fluid(fluid: FluidKind) -> Option<Self> {
        (!fluid.is_vanilla()).then(|| Self(MOD_TAGS + fluid.source().index() as u32))
    }
    /// The mod fluid (its source) this tag holds.
    pub fn mod_fluid(self) -> Option<FluidKind> {
        self.0
            .checked_sub(MOD_TAGS)
            .and_then(|i| FluidKind::from_index(i as usize))
    }
}

/// Whether `tag` holds `fluid`: a mod fluid's own tag by the table, the
/// caller's tags by `tags`.
fn holds<W: FluidMembership + ?Sized>(tags: &W, fluid: FluidKind, tag: FluidTagId) -> bool {
    match tag.mod_fluid() {
        Some(source) => fluid.source() == source,
        None => tags.fluid_in_tag(fluid, tag),
    }
}

/// An entity's immersion in one mod fluid, as `Entity.isInFluidType` sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModFluidImmersion {
    /// The fluid's source form.
    pub fluid: FluidKind,
    pub height: f64,
    pub eyes_inside: bool,
}
/// A tag's immersion in one `EntityFluidInteraction`: its height, whether the
/// eyes are in it, and the current its flow accumulated — the 26.2
/// `EntityFluidInteraction$Tracker` (26.3 split the kind's immersion and the
/// tag's current into two structures).
#[derive(Clone, Copy, Debug, Default)]
pub struct FluidTracker {
    pub height: f64,
    pub eyes_inside: bool,
    pub current: CurrentAccumulator,
}
/// The current a tag's flow accumulated, as `Tracker.applyCurrentTo` spends
/// it. `height` is the mob path's immersion depth (`living.rs` sets it from
/// its own scan); the interaction leaves it 0 — 26.2's tracker keeps the
/// height on itself, not on the current.
#[derive(Clone, Copy, Debug)]
pub struct CurrentAccumulator {
    pub height: f64,
    pub accumulated: Vec3,
    pub count: i32,
}
impl Default for CurrentAccumulator {
    fn default() -> Self {
        Self {
            height: 0.0,
            accumulated: Vec3::ZERO,
            count: 0,
        }
    }
}
pub trait CurrentTarget {
    type Error;
    fn is_player_class(&self) -> bool;
    fn velocity(&mut self) -> Result<Vec3, Self::Error>;
    /// Entity.addDeltaMovement, including finite-addition filtering and a fresh
    /// velocity read. Entity-specific overrides remain the provider's dispatch.
    fn add_delta_movement(&mut self, delta: Vec3) -> Result<(), Self::Error>;
}
impl CurrentAccumulator {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn accumulate(&mut self, current: Vec3) {
        self.accumulated = add(self.accumulated, current);
        self.count = self.count.wrapping_add(1);
    }
    pub fn apply_to<W: CurrentTarget>(&self, world: &mut W, speed: f64) -> Result<(), W::Error> {
        if self.count == 0 || self.accumulated.length_squared() < f64::from(1.0e-5f32) {
            return Ok(());
        }
        let mut current = if world.is_player_class() {
            self.accumulated.scale(1.0 / f64::from(self.count))
        } else {
            self.accumulated.normalize()
        };
        let velocity = world.velocity()?;
        current = current.scale(speed);
        if velocity.x.abs() < 0.003
            && velocity.z.abs() < 0.003
            && current.length() < 0.0045000000000000005
        {
            current = current.normalize().scale(0.0045000000000000005);
        }
        world.add_delta_movement(current)
    }
}
pub struct FluidSections<'a> {
    pub minimum_section_y: i32,
    pub has_fluid: &'a [bool],
}
pub trait FluidInteractionWorld: FluidMembership {
    type Error;
    type Chunk: Copy;
    fn interaction_box(&mut self) -> Result<Option<Aabb>, Self::Error>;
    /// FULL chunk lookup without generating/loading absent chunks.
    fn loaded_chunk(&mut self, x: i32, z: i32) -> Result<Option<Self::Chunk>, Self::Error>;
    fn chunk_sections(&mut self, chunk: Self::Chunk) -> Result<FluidSections<'_>, Self::Error>;
    fn bounding_box_min_y(&self) -> f64;
    fn block_x(&self) -> i32;
    fn eye_y(&mut self) -> Result<f64, Self::Error>;
    fn block_z(&self) -> i32;
    fn fluid_state(&mut self, position: BlockPos) -> Result<FluidState, Self::Error>;
    fn fluid_height(&mut self, state: FluidState, position: BlockPos) -> Result<f32, Self::Error>;
    fn flow(&mut self, state: FluidState, position: BlockPos) -> Result<Vec3, Self::Error>;
}
#[derive(Debug)]
pub enum FluidInteractionError<E> {
    World(E),
    NonFiniteBounds,
    BudgetExceeded,
}
impl<E: fmt::Display> fmt::Display for FluidInteractionError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::World(e) => e.fmt(f),
            Self::NonFiniteBounds => f.write_str("nonfinite immersion bounds"),
            Self::BudgetExceeded => f.write_str("immersion query budget exceeded"),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for FluidInteractionError<E> {}
fn spend<E>(budget: &mut usize) -> Result<(), FluidInteractionError<E>> {
    if *budget == 0 {
        return Err(FluidInteractionError::BudgetExceeded);
    }
    *budget -= 1;
    Ok(())
}
fn add(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}
fn java_max(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        a
    } else if b.is_nan() {
        b
    } else if a == 0.0 && b == 0.0 {
        f64::from_bits(a.to_bits() & b.to_bits())
    } else {
        a.max(b)
    }
}
/// Every required chunk must be loaded, even after a wet section is found.
/// Chunk order is Z then X; section order is ascending Y without early success.
pub fn has_fluid_and_loaded<W: FluidInteractionWorld>(
    world: &mut W,
    min: BlockPos,
    max: BlockPos,
    budget: &mut usize,
) -> Result<bool, FluidInteractionError<W::Error>> {
    let mut present = false;
    for z in (min.z >> 4)..=(max.z >> 4) {
        for x in (min.x >> 4)..=(max.x >> 4) {
            spend(budget)?;
            let Some(chunk) = world
                .loaded_chunk(x, z)
                .map_err(FluidInteractionError::World)?
            else {
                return Ok(false);
            };
            let sections = world
                .chunk_sections(chunk)
                .map_err(FluidInteractionError::World)?;
            for y in (min.y >> 4)..=(max.y >> 4) {
                spend(budget)?;
                let index = y.wrapping_sub(sections.minimum_section_y);
                if index >= 0 && (index as usize) < sections.has_fluid.len() {
                    present |= sections.has_fluid[index as usize];
                }
            }
        }
    }
    Ok(present)
}
#[derive(Clone, Default)]
pub struct EntityFluidInteraction {
    /// One tracker per tag, in construction order — 26.2's `trackerByFluid`.
    trackers: Vec<(FluidTagId, FluidTracker)>,
}
impl EntityFluidInteraction {
    /// Preserve the caller's tag-set iteration order: a fluid's tracker is
    /// the first tag that holds it, even when several tags contain that
    /// fluid. Each mod fluid of the table gets its own tag after the
    /// caller's (NeoForge tracks every `FluidType`), so its current and
    /// immersion are its own.
    pub fn new(tags: impl IntoIterator<Item = FluidTagId>) -> Self {
        let mut state = Self::default();
        for tag in tags {
            state.add_tag(tag);
        }
        state.add_mod_tags();
        state
    }
    fn add_tag(&mut self, tag: FluidTagId) {
        if !self.trackers.iter().any(|(t, _)| *t == tag) {
            self.trackers.push((tag, FluidTracker::default()));
        }
    }
    /// The table's mod fluids' tags, also for an entity made before its
    /// content root loaded them.
    fn add_mod_tags(&mut self) {
        let mods = FluidKind::all()
            .filter(|f| f.source() == *f)
            .filter_map(FluidTagId::of_mod_fluid);
        for tag in mods {
            self.add_tag(tag);
        }
    }
    /// The per-tag trackers, in construction order.
    pub fn trackers(&self) -> &[(FluidTagId, FluidTracker)] {
        &self.trackers
    }
    /// `getFluidHeight`: the tag's tracker, or 0 for a tag with no tracker.
    pub fn fluid_height(&self, tag: FluidTagId) -> f64 {
        self.trackers
            .iter()
            .find(|(t, _)| *t == tag)
            .map_or(0.0, |(_, t)| t.height)
    }
    pub fn is_in_fluid(&self, tag: FluidTagId) -> bool {
        self.fluid_height(tag) > 0.0
    }
    pub fn is_eye_in_fluid(&self, tag: FluidTagId) -> bool {
        self.trackers
            .iter()
            .any(|(t, tracker)| *t == tag && tracker.eyes_inside)
    }
    /// The mod fluids the entity is in, from its last update: their heights and
    /// whether the eyes are in them.
    pub fn mod_fluids(&self) -> impl Iterator<Item = ModFluidImmersion> + '_ {
        self.trackers.iter().filter_map(|(tag, tracker)| {
            let fluid = tag.mod_fluid()?;
            let immersion = ModFluidImmersion {
                fluid,
                height: tracker.height,
                eyes_inside: tracker.eyes_inside,
            };
            (immersion.height > 0.0 || immersion.eyes_inside).then_some(immersion)
        })
    }
    /// Each mod fluid's current, at its `motionScale`, when it `canPushEntity`.
    pub fn apply_mod_currents<W: CurrentTarget>(&self, world: &mut W) -> Result<(), W::Error> {
        for m in self.mod_fluids().filter(|m| m.height > 0.0) {
            let numbers = m.fluid.info().entity;
            if numbers.can_push {
                let tag = FluidTagId::of_mod_fluid(m.fluid).expect("a mod fluid");
                self.apply_current(world, tag, numbers.motion_scale)?;
            }
        }
        Ok(())
    }
    pub fn apply_current<W: CurrentTarget>(
        &self,
        world: &mut W,
        tag: FluidTagId,
        speed: f64,
    ) -> Result<(), W::Error> {
        if let Some((_, tracker)) = self.trackers.iter().find(|(t, _)| *t == tag) {
            tracker.current.apply_to(world, speed)?;
        }
        Ok(())
    }
    /// Reset previous tick state before any query. Provider failures and budget
    /// exhaustion retain all preceding tracker writes. 26.2's `update` is
    /// void: it answers nothing, and `Entity.updateFluidInteraction` reads
    /// `isInFluid` itself (26.3's returns whether any fluid was found).
    pub fn update<W: FluidInteractionWorld>(
        &mut self,
        world: &mut W,
        ignore_current: bool,
        mut budget: usize,
    ) -> Result<(), FluidInteractionError<W::Error>> {
        // `Tracker.reset` is void and the per-tag trackers persist — 26.3's
        // reset returns whether the tracker was already empty and the
        // interaction drops it.
        for (_, t) in &mut self.trackers {
            t.height = 0.0;
            t.eyes_inside = false;
            t.current.reset();
        }
        self.add_mod_tags();
        let Some(bounds) = world
            .interaction_box()
            .map_err(FluidInteractionError::World)?
        else {
            return Ok(());
        };
        let a = bounds.min_position();
        let b = bounds.max_position();
        if [a.x, a.y, a.z, b.x, b.y, b.z]
            .iter()
            .any(|v| !v.is_finite())
        {
            return Err(FluidInteractionError::NonFiniteBounds);
        }
        let low = BlockPos::new(a.x.floor() as i32, a.y.floor() as i32, a.z.floor() as i32);
        let high = BlockPos::new(
            (b.x.ceil() as i32).wrapping_sub(1),
            (b.y.ceil() as i32).wrapping_sub(1),
            (b.z.ceil() as i32).wrapping_sub(1),
        );
        if !has_fluid_and_loaded(
            world,
            BlockPos::new(low.x.wrapping_sub(1), low.y, low.z.wrapping_sub(1)),
            BlockPos::new(high.x.wrapping_add(1), high.y, high.z.wrapping_add(1)),
            &mut budget,
        )? {
            return Ok(());
        }
        let base_y = world.bounding_box_min_y();
        let block_x = world.block_x();
        let eye = world.eye_y().map_err(FluidInteractionError::World)?;
        let block_z = world.block_z();
        // The last fluid kind's tracker index, as `lastFluidType` caches it
        // (none when no tag holds the kind — the block is skipped outright).
        let mut last: Option<(FluidKind, Option<usize>)> = None;
        for x in low.x..=high.x {
            for y in low.y..=high.y {
                for z in low.z..=high.z {
                    spend(&mut budget)?;
                    let p = BlockPos::new(x, y, z);
                    let state = world.fluid_state(p).map_err(FluidInteractionError::World)?;
                    if state.kind() == FluidKind::Empty {
                        continue;
                    }
                    let surface = f64::from(y)
                        + f64::from(
                            world
                                .fluid_height(state, p)
                                .map_err(FluidInteractionError::World)?,
                        );
                    if surface < a.y {
                        continue;
                    }
                    let kind = state.kind();
                    let tracker = if let Some(last) = last.filter(|(k, _)| *k == kind) {
                        last.1
                    } else {
                        let tracker = self
                            .trackers
                            .iter()
                            .position(|(tag, _)| holds(world, kind, *tag));
                        last = Some((kind, tracker));
                        tracker
                    };
                    let Some(index) = tracker else {
                        continue;
                    };
                    // The eye check reads the plain fluid top: 26.3's
                    // `getHeightForCamera` (a source under a sturdy block
                    // counts as full height) does not exist in 26.2.
                    if x == block_x && z == block_z && eye >= f64::from(y) && eye <= surface {
                        self.trackers[index].1.eyes_inside = true;
                    }
                    let tracker = &mut self.trackers[index].1;
                    tracker.height = java_max(surface - base_y, tracker.height);
                    if !ignore_current {
                        let mut flow =
                            world.flow(state, p).map_err(FluidInteractionError::World)?;
                        // The flow is scaled by the tracker's own height (26.3
                        // scales by the current's, the max of the trackers
                        // that fed it).
                        if tracker.height < 0.4 {
                            flow = flow.scale(tracker.height);
                        }
                        tracker.current.accumulate(flow);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A body the accumulator pushes: the player flag and the velocity it
    /// reads are the only things `apply_to` asks for.
    struct Body {
        player: bool,
        velocity: Vec3,
        added: Vec3,
    }
    impl CurrentTarget for Body {
        type Error = std::convert::Infallible;
        fn is_player_class(&self) -> bool {
            self.player
        }
        fn velocity(&mut self) -> Result<Vec3, Self::Error> {
            Ok(self.velocity)
        }
        fn add_delta_movement(&mut self, delta: Vec3) -> Result<(), Self::Error> {
            self.added = self.added.add_vector(delta);
            Ok(())
        }
    }

    /// `EntityFluidInteraction$Tracker.applyCurrentTo`: a body that is not
    /// a player takes the direction of the accumulated current, so two samples
    /// of one block push as hard as one; a player takes their average, so the
    /// push grows with the number of blocks and is halved by two of them.
    #[test]
    fn a_current_pushes_a_player_by_its_average_and_anything_else_by_its_direction() {
        let mut acc = CurrentAccumulator::default();
        acc.accumulate(Vec3::new(1.0, 0.0, 0.0));
        let mut mob = Body {
            player: false,
            velocity: Vec3::ZERO,
            added: Vec3::ZERO,
        };
        acc.apply_to(&mut mob, 0.014).unwrap();
        assert!((mob.added.x - 0.014).abs() < 1e-12, "mob: {}", mob.added.x);
        acc.accumulate(Vec3::new(1.0, 0.0, 0.0));
        let mut mob = Body {
            player: false,
            velocity: Vec3::ZERO,
            added: Vec3::ZERO,
        };
        acc.apply_to(&mut mob, 0.014).unwrap();
        assert!(
            (mob.added.x - 0.014).abs() < 1e-12,
            "a second sample does not double a mob's push: {}",
            mob.added.x
        );
        let mut player = Body {
            player: true,
            velocity: Vec3::ZERO,
            added: Vec3::ZERO,
        };
        acc.apply_to(&mut player, 0.014).unwrap();
        assert!(
            (player.added.x - 0.014).abs() < 1e-12,
            "the average of two (1,0,0) samples is (1,0,0): {}",
            player.added.x
        );
    }

    /// The same body with a single weak sample: the average is (0.5,0,0) for a
    /// player, and a nearly still body gets the 0.0045 lift either way.
    #[test]
    fn a_weak_current_on_a_still_body_is_lifted_to_the_minimum() {
        let mut acc = CurrentAccumulator::default();
        // A tenth of a block a tick: 0.0014 once scaled, under the minimum.
        acc.accumulate(Vec3::new(0.1, 0.0, 0.0));
        let mut player = Body {
            player: true,
            velocity: Vec3::ZERO,
            added: Vec3::ZERO,
        };
        acc.apply_to(&mut player, 0.014).unwrap();
        assert!(
            (player.added.x - 0.0045).abs() < 1e-12,
            "{}",
            player.added.x
        );
        // A body that is already moving on x or z takes the small push as it is.
        let mut moving = Body {
            player: true,
            velocity: Vec3::new(0.05, 0.0, 0.0),
            added: Vec3::ZERO,
        };
        acc.apply_to(&mut moving, 0.014).unwrap();
        assert!(
            (moving.added.x - 0.0014).abs() < 1e-12,
            "{}",
            moving.added.x
        );
    }

    /// `applyCurrentTo` returns before reading the velocity with no samples, and when
    /// the accumulated current is under the 1e-5 of a squared length.
    #[test]
    fn no_current_and_a_current_under_the_threshold_push_nothing() {
        let mut body = Body {
            player: false,
            velocity: Vec3::ZERO,
            added: Vec3::ZERO,
        };
        CurrentAccumulator::default()
            .apply_to(&mut body, 0.014)
            .unwrap();
        assert_eq!(body.added, Vec3::ZERO);
        let mut acc = CurrentAccumulator::default();
        acc.accumulate(Vec3::new(0.001, 0.0, 0.0));
        acc.apply_to(&mut body, 0.014).unwrap();
        assert_eq!(body.added, Vec3::ZERO);
    }
}
