//! LocalPlayer sprint decisions around KeyboardInput.tick. Conditions are the
//! current player queries; setting sprint dispatches to the live attribute state.
use crate::input;

#[derive(Clone, Copy, Debug, Default)]
pub struct Keys {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shift: bool,
    pub sprint: bool,
    /// A controller stick's move vector (`[left, forward]`, Horizon's own),
    /// used instead of the keys' direction while it is pushed.
    pub analog: Option<[f32; 2]>,
}
impl Keys {
    pub fn direction(self) -> [f32; 2] {
        match self.analog {
            Some(v) => v,
            None => input::keyboard_direction(self.forward, self.backward, self.left, self.right),
        }
    }
    /// Add a stick's move vector: the direction flags code reading keys
    /// (sprint, vehicles, the server's input packet) see are set past a
    /// third of the travel.
    pub fn with_stick(mut self, stick: Option<[f32; 2]>) -> Self {
        if let Some([left, forward]) = stick {
            self.forward |= forward > 0.3;
            self.backward |= forward < -0.3;
            self.left |= left > 0.3;
            self.right |= left < -0.3;
            self.analog = Some([left, forward]);
        }
        self
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct SprintState {
    pub trigger_time: i32,
    pub previous_forward: bool,
    pub previous_shift: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Conditions {
    pub sprinting: bool,
    pub mobility_restricted: bool,
    pub passenger: bool,
    pub vehicle_can_sprint: bool,
    pub vehicle_local: bool,
    pub enough_food: bool,
    pub may_fly: bool,
    pub flying: bool,
    pub shallow_water: bool,
    pub slow_item: bool,
    pub fall_flying: bool,
    pub under_water: bool,
    /// The ridden vehicle's on-ground state: 1.21.4's sprint-start
    /// block reads the vehicle's `onGround` for a passenger.
    pub vehicle_on_ground: bool,
    pub moving_slowly: bool,
    pub swimming: bool,
    pub in_water: bool,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    pub minor_horizontal_collision: bool,
}
impl Conditions {
    fn possible(self, allow_shallow_water: bool) -> bool {
        !self.mobility_restricted
            && if self.passenger {
                self.vehicle_can_sprint && self.vehicle_local
            } else {
                self.enough_food || self.may_fly
            }
            && (allow_shallow_water || !self.shallow_water)
    }
    /// 1.21.4's `LocalPlayer.canStartSprinting`: not already sprinting,
    /// enough impulse, possible, not slowed by an item, never while
    /// fall-flying, and not moving slowly unless underwater. (1.21.5
    /// allows a sprint start while fall-flying underwater.)
    fn can_start(self, forward: bool) -> bool {
        !self.sprinting
            && forward
            && self.possible(self.flying)
            && !self.slow_item
            && !self.fall_flying
            && (!self.moving_slowly || self.under_water)
    }
    /// 1.21.4's `LocalPlayer.aiStep` stop block, on the current
    /// `hasForwardImpulse` (any forward impulse at all): the run stop
    /// also fires in water at the surface, which the shared form below
    /// does not model (a gap in both trees, not a port difference).
    fn should_stop(self, forward: bool, shift: bool) -> bool {
        if self.swimming {
            !self.possible(true) || !self.in_water || (!forward && !self.on_ground && !shift)
        } else {
            !self.possible(self.flying)
                || !forward
                || (self.horizontal_collision && !self.minor_horizontal_collision)
        }
    }
}
impl SprintState {
    /// Runs the sprint phase. The default vanilla sprint window is seven ticks;
    /// it is supplied so the options binding can also select 0..10. The setter
    /// is called for every native request, including repeated true requests.
    pub fn tick<E>(
        &mut self,
        keys: Keys,
        conditions: Conditions,
        window: i32,
        mut set_sprinting: impl FnMut(bool) -> Result<(), E>,
    ) -> Result<(), E> {
        if self.trigger_time > 0 {
            self.trigger_time -= 1;
        }
        let old_forward = self.previous_forward;
        let old_shift = self.previous_shift;
        // 1.21.4's `LocalPlayer.hasEnoughImpulseToStartSprinting`, on the
        // raw forward impulse: 0.8 on land, any forward impulse at all
        // underwater. 1.21.5 checks `hasForwardImpulse` (any impulse) on
        // the normalized move vector instead, so a half-pushed stick
        // starts a sprint there and not here. `previous_forward` tracks
        // this same threshold: aiStep's fresh-press edge is the previous
        // tick's `hasEnoughImpulseToStartSprinting`.
        let impulse = keys.direction()[1];
        let forward = if conditions.under_water {
            impulse > 1.0e-5
        } else {
            f64::from(impulse) >= 0.8
        };
        // 1.21.4's `ClientInput.hasForwardImpulse`, for the stop block.
        let has_forward = impulse > 1.0e-5;
        self.previous_forward = forward;
        self.previous_shift = keys.shift;
        // 1.21.4 resets the trigger on shift (last tick's) and on item
        // use, but not on backward — the backward reset is 1.21.5's.
        if old_shift || (conditions.slow_item && !conditions.passenger) {
            self.trigger_time = 0;
        }
        let mut sprinting = conditions.sprinting;
        // 1.21.4's sprint-start block needs the ground (or the vehicle
        // under it, or being underwater) and a fresh press — no
        // mid-air sprint start. 1.21.5 drops the ground gate.
        let grounded =
            conditions.on_ground || conditions.vehicle_on_ground || conditions.under_water;
        if conditions.can_start(forward) && grounded && !old_shift {
            if !old_forward {
                if self.trigger_time > 0 {
                    set_sprinting(true)?;
                    sprinting = true;
                } else {
                    self.trigger_time = window;
                }
            }
            if keys.sprint {
                set_sprinting(true)?;
                sprinting = true;
            }
        }
        // 1.21.4's water block: the sprint key also starts a sprint
        // swimming at the surface (in water, not under it), with no
        // ground or fresh-press gate.
        if conditions.can_start(forward)
            && conditions.in_water
            && !conditions.under_water
            && keys.sprint
        {
            set_sprinting(true)?;
            sprinting = true;
        }
        if sprinting && conditions.should_stop(has_forward, keys.shift) {
            set_sprinting(false)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standing() -> Conditions {
        Conditions {
            enough_food: true,
            on_ground: true,
            ..Conditions::default()
        }
    }

    fn step(state: &mut SprintState, keys: Keys, sprinting: &mut bool) {
        step_with(state, keys, sprinting, standing())
    }

    fn step_with(state: &mut SprintState, keys: Keys, sprinting: &mut bool, base: Conditions) {
        let current = *sprinting;
        let mut next = current;
        state
            .tick(
                keys,
                Conditions {
                    sprinting: current,
                    ..base
                },
                7,
                |value| {
                    next = value;
                    Ok::<(), ()>(())
                },
            )
            .unwrap();
        *sprinting = next;
    }

    /// 1.21.4's `hasEnoughImpulseToStartSprinting`: 0.8 on land, any
    /// forward impulse underwater. A half-pushed stick (0.5) starts no
    /// sprint on land but does under water.
    #[test]
    fn a_half_pushed_stick_starts_no_sprint_on_land_but_does_under_water() {
        let mut state = SprintState::default();
        let mut sprinting = false;
        let keys = Keys {
            analog: Some([0.0, 0.5]),
            sprint: true,
            ..Keys::default()
        };
        step(&mut state, keys, &mut sprinting);
        assert!(!sprinting, "0.5 forward is below the 0.8 land threshold");
        step_with(
            &mut state,
            keys,
            &mut sprinting,
            Conditions {
                under_water: true,
                in_water: true,
                ..standing()
            },
        );
        assert!(sprinting, "any forward impulse starts a sprint under water");
    }

    /// 1.21.4's sprint-start block needs the ground (or water): the
    /// sprint key mid-air starts nothing.
    #[test]
    fn no_sprint_start_mid_air() {
        let mut state = SprintState::default();
        let mut sprinting = false;
        let keys = Keys {
            forward: true,
            sprint: true,
            ..Keys::default()
        };
        step_with(
            &mut state,
            keys,
            &mut sprinting,
            Conditions {
                on_ground: false,
                ..standing()
            },
        );
        assert!(!sprinting, "no ground, no water, no sprint start");
        step(&mut state, keys, &mut sprinting);
        assert!(sprinting, "the same keys start a sprint on the ground");
    }

    /// 1.21.4 resets the double-tap window on shift and item use, not
    /// on backward (the backward reset is 1.21.5's): a backward press
    /// inside the window leaves the second forward tap a sprint.
    #[test]
    fn backward_does_not_reset_the_double_tap_window() {
        let mut state = SprintState::default();
        let mut sprinting = false;
        step(
            &mut state,
            Keys {
                forward: true,
                ..Keys::default()
            },
            &mut sprinting,
        );
        assert!(!sprinting, "the first tap only arms the window");
        step(
            &mut state,
            Keys {
                backward: true,
                ..Keys::default()
            },
            &mut sprinting,
        );
        step(
            &mut state,
            Keys {
                forward: true,
                ..Keys::default()
            },
            &mut sprinting,
        );
        assert!(
            sprinting,
            "backward inside the window must not cancel the double-tap"
        );
    }

    /// Vanilla sprint is a state. Releasing the sprint key, including on the
    /// jump tick, keeps it while forward is held; releasing forward stops it.
    #[test]
    fn sprint_survives_releasing_the_key_through_a_jump() {
        let mut state = SprintState::default();
        let mut sprinting = false;
        step(
            &mut state,
            Keys {
                forward: true,
                sprint: true,
                ..Keys::default()
            },
            &mut sprinting,
        );
        assert!(sprinting);
        step(
            &mut state,
            Keys {
                forward: true,
                jump: true,
                ..Keys::default()
            },
            &mut sprinting,
        );
        assert!(
            sprinting,
            "a jump after the sprint key comes up must stay a sprint"
        );
        step(
            &mut state,
            Keys {
                jump: true,
                ..Keys::default()
            },
            &mut sprinting,
        );
        assert!(!sprinting);
    }
}
