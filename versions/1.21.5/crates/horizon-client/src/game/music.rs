//! `Minecraft.getSituationalMusic` and `getMusicVolume` for the music manager,
//! the local player's ambient sound handlers and the sound engine tick.
use super::Game;
use horizon_client::{
    blocks::Fluid,
    music::{BackgroundMusic, Music, Situation},
    options_ui::MusicGroup,
    random_source,
    sound_engine::{SoundInstance, Sounds, TickContext},
};

impl Game<'_> {
    /// `Minecraft.tick` -> `MusicManager.tick`, once per client tick.
    pub(super) fn tick_music(&mut self) {
        // Splitscreen: one music, P1's.
        if self.options.split_guest {
            return;
        }
        let (mut music, mut volume) = self.situational_music();
        // A native mod's own choice of track and volume, then its layers.
        self.native_music(&mut music, &mut volume);
        let mut layers = Vec::new();
        self.native_music_layers(&mut layers);
        self.music_layers.tick(&layers, &mut self.audio);
        // The Music page's off tracks leave every pool they play in (the
        // extra off: nothing is filtered, vanilla's pools play as they are).
        let off = if self.settings.extras.music_page {
            &self.settings.extras.music_off[..]
        } else {
            &[][..]
        };
        self.music.set_off(off);
        // LevelLoadingScreen: no new track before the player is in the world.
        // A reconfiguration's ServerReconfigScreen is a plain Screen, not a
        // LevelLoadingScreen: its menu music starts as on any other screen.
        let loading = self.title.is_none()
            && self.player.is_none()
            && self.win.is_none()
            && !self
                .remote
                .as_ref()
                .is_some_and(|r| r.reconfiguring.is_some());
        // `Minecraft.tick` (1.21.5): the music manager only ticks while
        // the game runs — a paused game freezes the track timer (1.21.6
        // ticks it always).
        if !self.pause {
            self.music
                .tick(music.as_ref(), volume, loading, &mut self.audio);
        }
    }
    /// The Music page's groups: every track the loaded `sounds.json` can play,
    /// by situation (see `music::track_groups`).
    pub(super) fn music_groups(&self) -> Vec<MusicGroup> {
        horizon_client::music::track_groups(&self.audio.engine.registry)
    }
    /// A Music page row: play this track of `event` now.
    pub(super) fn play_music_track(&mut self, event: &str, track: &str) {
        self.music.play_track(event, track, &mut self.audio);
    }
    /// `Minecraft.tick` -> `SoundManager.tick(pause)`, after the music, with
    /// `pause` (every channel) when the game pauses and `resume` when it
    /// carries on. 1.21.6 pauses all but MUSIC and UI instead.
    pub(super) fn tick_sound_engine(&mut self) {
        update_bound_entities(self.audio.engine.bound_entities(), |id| {
            self.sound_entity_state(id)
        });

        if self.pause != self.sound_paused {
            if self.pause {
                self.audio.pause();
            } else {
                self.audio.resume();
            }
            self.sound_paused = self.pause;
        }
        let (vehicle, carts) = self.start_cart_sounds();
        let mobs = self.start_mob_sounds();
        let ridden = self.start_riding_sound();
        let (player_position, player_speed_sqr, player_fall_flying) =
            self.player.as_ref().map_or(([0.0; 3], 0.0, false), |p| {
                let at = p.position;
                (
                    [at.x, at.y, at.z],
                    p.velocity.length_squared(),
                    p.fall_flying,
                )
            });
        // LocalPlayer.onSyncedDataUpdated: starting to glide starts the
        // elytra's flying sound.
        if player_fall_flying && !self.was_fall_flying {
            let instance = SoundInstance::elytra(player_position, random_source::create());
            self.audio.play_instance(instance, false);
        }
        self.was_fall_flying = player_fall_flying;
        let ctx = TickContext {
            player_underwater: self.ambience.was_underwater(),
            player_removed: self.player.is_none(),
            vehicle,
            carts,
            mobs,
            frozen: !self.clock.runs_normally(),
            player_position,
            player_speed_sqr,
            player_fall_flying,
            ridden,
            camera_position: self.audio.listener,
        };
        self.audio
            .set_listener_reserve(self.settings.extras.ui_sound_reserve);
        self.audio.tick(self.pause, &ctx);
    }
    /// `Minecraft.setLevel` -> `updateLevelInEngines`: `SoundManager.stop()`.
    /// The music manager keeps its stopped track and, on its next tick, waits
    /// the level's music delay before the next one.
    pub(super) fn stop_sounds_for_new_level(&mut self) {
        self.ambience.clear(&mut self.audio);
        self.audio.stop_all();
    }
    /// `getSituationalMusic` and `getMusicVolume`.
    pub(super) fn situational_music(&self) -> (Option<Music>, f32) {
        let empty = BackgroundMusic::default();
        // The title screen has no player (and no level for the camera probe).
        // A server's client has its player from handleLogin on, before the
        // first teleport makes it here: the camera is still at the origin.
        // A reconfiguration's clearClientLevel drops it again (menu music),
        // so the last server's columns are never probed.
        let eye = if self.title.is_some() {
            None
        } else {
            self.eye_position().or_else(|| {
                self.remote
                    .as_ref()
                    .is_some_and(|r| r.has_level())
                    .then_some([0.0; 3])
            })
        };
        let (background, probe_volume) = match eye {
            Some(eye) => {
                let (dimension, biome) = (self.dim.id(), self.biome_at(eye));
                self.res
                    .music()
                    .at(dimension, biome, world_biomes!(self).pack_file(biome))
            }
            None => (empty, 1.0),
        };
        let situation = Situation {
            // WinScreen.getBackgroundMusic.
            screen_music: self
                .win
                .is_some()
                .then(horizon_client::music::Music::credits),
            in_world: eye.is_some(),
            in_end: self.dim == super::Dim::End,
            boss_music: self.boss_music(),
            background: &background,
            probe_volume,
            creative: self.creative,
            underwater: self.ambience.was_underwater(),
        };
        (situation.music(), situation.volume())
    }
    /// `BiomeManager.getNoiseBiomeAtPosition`: the quart cell holding a position.
    pub(super) fn biome_at(&self, p: [f64; 3]) -> &str {
        let (x, y, z) = (
            p[0].floor() as i32,
            p[1].floor() as i32,
            p[2].floor() as i32,
        );
        self.world
            .chunks
            .get(&(x >> 4, z >> 4))
            .map_or("minecraft:plains", |c| {
                world_biomes!(self).name(c.biome(x, y, z))
            })
    }
    /// `isEyeInFluid(WATER)`: the eye inside the water of its block column.
    pub(super) fn eye_in_water(&self, x: f64, eye_y: f64, z: f64) -> bool {
        let (bx, by, bz) = (x.floor() as i32, eye_y.floor() as i32, z.floor() as i32);
        let f = self.res.facts().get(self.world.block(bx, by, bz));
        if f.fluid != Fluid::Water {
            return false;
        }
        let above = self.res.facts().get(self.world.block(bx, by + 1, bz));
        let height = if above.fluid == Fluid::Water {
            1.0
        } else {
            f64::from(f.fluid_level) / 9.0
        };
        eye_y <= f64::from(by) + height
    }
}

impl Game<'_> {
    /// The local player's ambient sound handlers (LocalPlayer.tick).
    pub(super) fn tick_ambience(&mut self) {
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let feet = [p.position.x, p.position.y, p.position.z];
        let eye_y = feet[1] + f64::from(p.dimensions.eye_height);
        let biome = self.biome_at(feet);
        let sounds =
            self.res
                .ambient()
                .at(self.dim.id(), biome, world_biomes!(self).pack_file(biome));
        let underwater = self.eye_in_water(feet[0], eye_y, feet[2]);
        let world = &self.world;
        let light = |b: [i32; 3]| {
            let l = world.light(b[0], b[1], b[2]);
            (l >> 4, l & 15)
        };
        let (half, height) = (
            f64::from(p.dimensions.width / 2.0),
            f64::from(p.dimensions.height),
        );
        let registry = &self.pack.assets.registry;
        let facts = self.res.facts();
        let bubble_column = horizon_client::ambience::bubble_column_in(
            [
                feet[0] - half,
                feet[1],
                feet[2] - half,
                feet[0] + half,
                feet[1] + height,
                feet[2] + half,
            ],
            |b| {
                let state = world.block(b[0], b[1], b[2]);
                (&*facts.get(state).name == "minecraft:bubble_column").then(|| {
                    registry
                        .state(state)
                        .and_then(|s| s.properties.get("drag"))
                        .is_some_and(|d| d == "true")
                })
            },
        );
        let ctx = horizon_client::ambience::AmbientContext {
            sounds: &sounds,
            position: feet,
            eye_y,
            underwater,
            spectator: self.spectator(),
            light: &light,
            bubble_column,
        };
        self.ambience.tick(&ctx, &mut self.audio);
    }
}

/// Resolve each live entity once per sound tick, even when several sounds name it.
fn update_bound_entities(
    entities: Vec<horizon_client::sound_engine::SoundEntity>,
    mut resolve: impl FnMut(u32) -> Option<horizon_client::sound_engine::SoundEntityState>,
) {
    use horizon_client::sound_engine::SoundEntityState;
    let mut states = std::collections::HashMap::new();
    for entity in entities {
        let previous = entity.state();
        if previous.removed {
            continue;
        }
        let state = *states
            .entry(entity.id)
            .or_insert_with(|| resolve(entity.id));
        entity.update(state.unwrap_or(SoundEntityState {
            removed: true,
            ..previous
        }));
    }
}

#[cfg(test)]
mod bound_lookup_tests {
    use super::update_bound_entities;
    use horizon_client::sound_engine::{SoundEntity, SoundEntityState};

    #[test]
    fn simultaneous_sounds_share_a_lookup_but_follow_each_new_tick() {
        let a = SoundEntity::new(7, [0.0; 3], false);
        let b = SoundEntity::new(7, [1.0; 3], false);
        let mut calls = Vec::new();
        for position in [[24.0, 80.0, -16.0], [25.5, 81.0, -18.0]] {
            update_bound_entities(vec![a.clone(), b.clone()], |id| {
                calls.push(id);
                Some(SoundEntityState {
                    position,
                    silent: true,
                    removed: false,
                })
            });
            assert_eq!(a.state().position, position);
            assert_eq!(b.state().position, position);
            assert!(a.state().silent && b.state().silent);
        }
        assert_eq!(calls, vec![7, 7]);
    }

    #[test]
    fn missing_entities_stop_all_their_sounds_without_reviving_reused_ids() {
        let a = SoundEntity::new(7, [1.0; 3], false);
        let b = SoundEntity::new(7, [2.0; 3], false);
        let mut calls = 0;
        update_bound_entities(vec![a.clone(), b.clone()], |_| {
            calls += 1;
            None
        });
        assert_eq!(calls, 1);
        assert!(a.state().removed && b.state().removed);
        let replacement = SoundEntity::new(7, [3.0; 3], false);
        update_bound_entities(vec![a.clone(), replacement.clone(), b.clone()], |_| {
            calls += 1;
            Some(SoundEntityState {
                position: [4.0; 3],
                silent: false,
                removed: false,
            })
        });
        assert_eq!(calls, 2);
        assert!(a.state().removed && b.state().removed);
        assert_eq!(a.state().position, [1.0; 3]);
        assert_eq!(replacement.state().position, [4.0; 3]);
        assert!(!replacement.state().removed);
    }
}
