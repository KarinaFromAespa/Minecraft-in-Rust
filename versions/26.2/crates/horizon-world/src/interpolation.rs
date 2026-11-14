//! How a client moves a living entity it doesn't simulate along the
//! positions its server sends (vanilla 26.2 `InterpolationHandler`, which
//! `LivingEntity` gives every living entity: three ticks by default): each
//! packet re-targets it, and each client tick lerps a 1/steps share of the
//! way there, so a move takes three ticks to show. A punched mob's first
//! move, sent at once (`hurtMarked`), thus takes three ticks to show.

/// A position and rotation (`PositionAndRotation`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PosRot {
    pub position: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
}

impl PosRot {
    /// `Objects.equals` on the angles and `Vec3.equals` on the position
    /// (`Float.equals` / `Double.compare`, so -0.0 is not 0.0).
    fn is(&self, position: [f64; 3], yaw: f32, pitch: f32) -> bool {
        self.yaw.to_bits() == yaw.to_bits()
            && self.pitch.to_bits() == pitch.to_bits()
            && same_position(self.position, position)
    }
}

/// `Vec3.equals`.
fn same_position(a: [f64; 3], b: [f64; 3]) -> bool {
    (0..3).all(|i| a[i].to_bits() == b[i].to_bits())
}

/// What the client does with the entity when a packet arrives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Start {
    /// Moves on to it over the coming ticks.
    Interpolating,
    /// Already headed there: nothing changes.
    Unchanged,
    /// Is put there at once (`snapTo`): types with no interpolation steps.
    Snap(PosRot),
}

/// `InterpolationHandler` and its `InterpolationData`.
#[derive(Clone, Debug)]
pub struct InterpolationHandler {
    /// `interpolationSteps`: 3 for living entities
    /// (`DEFAULT_INTERPOLATION_STEPS`), 0 to snap.
    steps: i32,
    /// The data's own position and rotation: where the last packet said.
    target: PosRot,
    /// `InterpolationData.steps`: the ticks left.
    remaining: i32,
    /// `previousTickPosition`: where the entity was last tick.
    previous_position: Option<[f64; 3]>,
    /// `previousTickRot`: (xRot, yRot) last tick — the entity's angles as
    /// they were when the move arrived, then as `setRot` left them after
    /// each tick (yaw wrapped, pitch wrapped and clamped).
    previous_rot: Option<(f32, f32)>,
}

/// `Mth.wrapDegrees(double)`.
fn wrap_degrees(degrees: f64) -> f64 {
    let mut wrapped = degrees % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}

/// `Mth.lerp(double, double, double)`.
fn lerp(a: f64, from: f64, to: f64) -> f64 {
    from + a * (to - from)
}

/// `Mth.rotLerp(double, double, double)`.
fn rot_lerp(a: f64, from: f64, to: f64) -> f64 {
    from + a * wrap_degrees(to - from)
}

/// `Entity.setYRot` / `setXRot`: what the entity's angles are after it is
/// put somewhere (yaw wrapped, pitch wrapped and clamped to [-90, 90]).
fn set_rot(yaw: f32, pitch: f32) -> (f32, f32) {
    (yaw % 360.0, (pitch % 360.0).clamp(-90.0, 90.0))
}

impl InterpolationHandler {
    /// The handler of an entity whose interpolation lasts `steps` ticks.
    pub fn new(steps: i32) -> Self {
        Self {
            steps,
            target: PosRot::default(),
            remaining: 0,
            previous_position: None,
            previous_rot: None,
        }
    }
    /// `hasActiveInterpolation`.
    pub fn active(&self) -> bool {
        self.remaining > 0
    }
    /// Where the entity is headed: the last packet's position and rotation
    /// while interpolating, else where it is.
    pub fn heading(&self, entity: PosRot) -> PosRot {
        if self.active() { self.target } else { entity }
    }
    /// `InterpolationHandler.interpolateTo(position, yRot, xRot)` for an
    /// entity at `entity`: a packet without a position or rotation keeps
    /// the one it is headed for (the caller fills those in from
    /// `heading`, as `moveOrInterpolateTo` does).
    pub fn interpolate_to(
        &mut self,
        entity: PosRot,
        position: [f64; 3],
        yaw: f32,
        pitch: f32,
    ) -> Start {
        if self.steps == 0 {
            self.cancel();
            return Start::Snap(PosRot {
                position,
                yaw,
                pitch,
            });
        }
        if self.active() && self.target.is(position, yaw, pitch) {
            return Start::Unchanged;
        }
        self.remaining = self.steps;
        self.target = PosRot {
            position,
            yaw,
            pitch,
        };
        // `previousTickPosition = entity.position()` and
        // `previousTickRot = (entity.getXRot(), entity.getYRot())`: the
        // entity's angles as they are, not wrapped or clamped (only
        // `setRot` does that, after a tick).
        self.previous_position = Some(entity.position);
        self.previous_rot = Some((entity.pitch, entity.yaw));
        Start::Interpolating
    }
    /// `cancel`: no steps left, no previous tick.
    pub fn cancel(&mut self) {
        self.remaining = 0;
        self.previous_position = None;
        self.previous_rot = None;
    }
    /// `InterpolationHandler.interpolate`, once a client tick: where the
    /// entity is put, or None when there is nothing to walk (and the
    /// handler resets). The client moves the entity only this way, so the
    /// previous tick is where it is, and the deltas
    /// `interpolate` would adjust the target by are zero.
    pub fn interpolate(&mut self) -> Option<PosRot> {
        if !self.active() {
            self.cancel();
            return None;
        }
        // `double alpha = 1.0 / steps`: a double share, not a float one,
        // and the angles lerp in double too, cast to float after.
        let alpha = 1.0f64 / self.remaining as f64;
        // The deltas are zero (the client moves the entity only this way),
        // but `addDelta`/`addRotation` still run: adding 0.0 turns a -0.0
        // target component into 0.0, which the Unchanged check can see.
        #[allow(clippy::identity_op)]
        {
            self.target.position = self.target.position.map(|v| v + 0.0);
            self.target.yaw += 0.0;
            self.target.pitch += 0.0;
        }
        let from_position = self.previous_position.unwrap_or([0.0; 3]);
        let from_rot = self.previous_rot.unwrap_or((0.0, 0.0));
        let at = PosRot {
            position: [0, 1, 2]
                .map(|i| lerp(alpha, from_position[i], self.target.position[i])),
            yaw: rot_lerp(alpha, f64::from(from_rot.1), f64::from(self.target.yaw)) as f32,
            pitch: lerp(alpha, f64::from(from_rot.0), f64::from(self.target.pitch)) as f32,
        };
        self.remaining -= 1;
        self.previous_position = Some(at.position);
        let (wrapped_yaw, clamped_pitch) = set_rot(at.yaw, at.pitch);
        self.previous_rot = Some((clamped_pitch, wrapped_yaw));
        Some(at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f64) -> PosRot {
        PosRot {
            position: [x, 0.0, 0.0],
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    fn round3(v: f64) -> f64 {
        (v * 1000.0).round() / 1000.0
    }

    #[test]
    fn a_position_takes_three_ticks_to_reach() {
        let mut h = InterpolationHandler::new(3);
        let mut entity = at(0.0);
        h.interpolate_to(entity, [0.3, 0.0, 0.0], 0.0, 0.0);
        let mut seen = Vec::new();
        while let Some(p) = h.interpolate() {
            entity = p;
            seen.push(round3(p.position[0]));
        }
        assert_eq!(seen, [0.1, 0.2, 0.3]);
        assert!(!h.active());
        assert_eq!(entity.position[0], 0.3);
    }

    #[test]
    fn a_re_target_restarts_the_three_ticks() {
        let mut h = InterpolationHandler::new(3);
        let mut entity = at(0.0);
        h.interpolate_to(entity, [1.0, 0.0, 0.0], 0.0, 0.0);
        entity = h.interpolate().unwrap();
        assert_eq!(round3(entity.position[0]), 0.333);
        h.interpolate_to(entity, [2.0, 0.0, 0.0], 0.0, 0.0);
        let mut seen = Vec::new();
        while let Some(p) = h.interpolate() {
            entity = p;
            seen.push(round3(p.position[0]));
        }
        assert_eq!(seen, [0.889, 1.444, 2.0]);
        assert_eq!(entity.position[0], 2.0);
    }

    #[test]
    fn a_repeat_packet_changes_nothing() {
        let mut h = InterpolationHandler::new(3);
        let entity = at(0.0);
        assert_eq!(
            h.interpolate_to(entity, [1.0, 0.0, 0.0], 0.0, 0.0),
            Start::Interpolating
        );
        assert_eq!(
            h.interpolate_to(entity, [1.0, 0.0, 0.0], 0.0, 0.0),
            Start::Unchanged
        );
        assert!(h.active());
    }

    #[test]
    fn a_negative_zero_yaw_is_not_the_same_after_a_tick() {
        let mut h = InterpolationHandler::new(3);
        let entity = at(0.0);
        h.interpolate_to(entity, [1.0, 0.0, 0.0], -0.0, 0.0);
        // The tick's addRotation(0, 0) turns the target's -0.0 into 0.0,
        // so the same packet again is a new target, not a repeat.
        h.interpolate().unwrap();
        assert_eq!(
            h.interpolate_to(entity, [1.0, 0.0, 0.0], -0.0, 0.0),
            Start::Interpolating
        );
    }

    #[test]
    fn rotation_turns_the_short_way() {
        let mut h = InterpolationHandler::new(3);
        let entity = PosRot {
            position: [0.0; 3],
            yaw: 350.0,
            pitch: 0.0,
        };
        h.interpolate_to(entity, [0.0; 3], 10.0, 0.0);
        let mut seen = Vec::new();
        while let Some(p) = h.interpolate() {
            seen.push(round3(f64::from(p.yaw)));
        }
        // The yaw wraps after each tick (setRot), so the last tick lerps
        // from 3.333 and lands exactly on 10.
        assert_eq!(seen, [356.667, 363.333, 10.0]);
    }

    #[test]
    fn no_interpolation_steps_snaps() {
        let mut h = InterpolationHandler::new(0);
        let start = h.interpolate_to(at(0.0), [2.0, 0.0, 0.0], 90.0, 0.0);
        assert_eq!(
            start,
            Start::Snap(PosRot {
                position: [2.0, 0.0, 0.0],
                yaw: 90.0,
                pitch: 0.0
            })
        );
    }
}
