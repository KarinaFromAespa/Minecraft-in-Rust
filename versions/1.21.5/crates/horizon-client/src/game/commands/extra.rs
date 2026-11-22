//! Commands that act on the player and the hosted game rather than the world:
//! `/spectate` (SpectateCommand), `/posteffect` (PostEffectCommand), `/kick`,
//! `/fetchprofile`, `/publish`, `/unpublish`, `/debug` and `/jfr`, with
//! vanilla's feedback and failures.
//! No `/waypoint` (WaypointCommand), `/dialog` (DialogCommand) or `/version`
//! (VersionCommand): all three arrived in 1.21.6.
use super::{Arg, Context, Game, R, Target, err};
use horizon_client::actions::Action;
use horizon_client::{
    command::CommandError,
    skins::{NameOrId, Resolve, ResolvedProfile},
    snbt,
    text::{self, ClickEvent, InlineObject, Style, Text},
};
use horizon_world::nbt::Tag;
use std::{collections::HashMap, time::Instant};

/// A `/fetchprofile` lookup in flight: what it resolves, the message ids it
/// answers with (`commands.fetchprofile.<kind>.success|failure`), the
/// argument they show, and the source to answer.
pub(in crate::game) struct ProfileFetch {
    key: NameOrId,
    kind: &'static str,
    argument: Text,
    source: Option<super::execute::Source>,
}

/// `ResolvableProfile.CODEC` of a resolved profile as NBT (`id` as an int
/// array, `name`, and `properties` only when there are some, each with its
/// `signature` only when signed), and the same as the JSON a head object
/// draws from.
fn encode_profile(profile: &ResolvedProfile) -> (Tag, serde_json::Value) {
    let ints = [
        (profile.id >> 96) as i32,
        (profile.id >> 64) as i32,
        (profile.id >> 32) as i32,
        profile.id as i32,
    ];
    let mut fields = vec![
        ("id", Tag::IntArray(ints.to_vec())),
        ("name", Tag::String(profile.name.as_str().into())),
    ];
    let mut json = serde_json::json!({ "id": ints, "name": profile.name });
    if !profile.properties.is_empty() {
        let mut list = Vec::new();
        let mut json_list = Vec::new();
        for (name, value, signature) in &profile.properties {
            let mut entry = vec![
                ("name", Tag::String(name.as_str().into())),
                ("value", Tag::String(value.as_str().into())),
            ];
            let mut j = serde_json::json!({ "name": name, "value": value });
            if let Some(signature) = signature {
                entry.push(("signature", Tag::String(signature.as_str().into())));
                j["signature"] = signature.clone().into();
            }
            list.push(Tag::compound(entry));
            json_list.push(j);
        }
        fields.push(("properties", Tag::List(list)));
        json["properties"] = json_list.into();
    }
    (Tag::compound(fields), json)
}
#[cfg(test)]
mod fetchprofile_tests {
    use super::*;
    #[test]
    fn unsigned_profile_without_properties_omits_them() {
        let profile = ResolvedProfile {
            id: 0x0000_0001_0000_0002_0000_0003_0000_0004,
            name: "Steve".into(),
            properties: Vec::new(),
        };
        let (tag, json) = encode_profile(&profile);
        assert_eq!(snbt::to_snbt(&tag), "{id:[I;1,2,3,4],name:\"Steve\"}");
        assert!(json.get("properties").is_none());
    }
    #[test]
    fn properties_keep_the_signature_only_when_signed() {
        let profile = ResolvedProfile {
            id: 1,
            name: "Alex".into(),
            properties: vec![
                ("textures".into(), "v".into(), Some("s".into())),
                ("other".into(), "w".into(), None),
            ],
        };
        let (tag, _) = encode_profile(&profile);
        assert_eq!(
            snbt::to_snbt(&tag),
            "{id:[I;0,0,0,1],name:\"Alex\",properties:[{name:\"textures\",signature:\"s\",value:\"v\"},{name:\"other\",value:\"w\"}]}"
        );
    }
}
/// Server-side player and server state these commands keep.
#[derive(Default)]
pub(in crate::game) struct CommandExtras {
    /// ServerPlayer.camera: the entity `/spectate` attached the view to.
    pub camera: Option<Target>,
    /// The cameras of the guests of a server, by connection
    /// (game/lan_camera.rs).
    pub guest_cameras: HashMap<u32, super::super::lan_camera::GuestCamera>,
    /// `/spectate` for a guest: connection and the camera's network id
    /// (None to stop), taken in the guests' tick.
    pub camera_requests: Vec<(u32, Option<i32>)>,
    /// ServerPlayer.postEffects, in insertion order.
    pub post_effects: Vec<String>,
    /// MinecraftServer's tick profiler (`/debug start`): when it started and
    /// the tick count then.
    pub profiler: Option<(Instant, u64)>,
    /// `debugCommandProfilerDelayStart`: the profiler starts with the next tick.
    pub profiler_delay: bool,
    /// `/fetchprofile name|id` lookups still running.
    pub profile_fetches: Vec<ProfileFetch>,
    /// The `/jfr start` recording: when it started and the tick count then.
    pub jfr: Option<(Instant, u64, String)>,
    /// `/datapack disable vanilla` took the built-in pack out of the selection.
    pub vanilla_disabled: bool,
    /// Attribute maps of living entities whose attributes Horizon does not
    /// model (an armor stand's, a mob's beyond its modelled three).
    pub attributes: HashMap<Target, horizon_world::attributes::entity::EntityAttributes>,
    /// The `CommandResultCallback` results of the outermost source, when a
    /// harness records them.
    pub results: Option<Vec<(bool, i32)>>,
    /// The running `/pregen` task.
    pub pregen: Option<super::pregen::PregenTask>,
}


fn num(n: impl std::fmt::Display) -> Text {
    Text::literal(n.to_string())
}
fn tr_error(key: &str, args: &[Text]) -> CommandError {
    CommandError::new(Text::translate(key, args))
}
/// `String.format(Locale.ROOT, "%.2f", v)`.
fn two_places(v: f64) -> String {
    super::percent_f(v, 2)
}
/// `Util.getFilenameFormattedDateTime` (`yyyy-MM-dd_HH.mm.ss`, local time).
fn filename_date_time() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let (y, mo, d, h, mi, s) = civil(secs);
    format!("{y:04}-{mo:02}-{d:02}_{h:02}.{mi:02}.{s:02}")
}
/// UTC civil date and time of a Unix timestamp.
fn civil(secs: i64) -> (i64, i64, i64, i64, i64, i64) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d, rem / 3600, rem % 3600 / 60, rem % 60)
}

impl Game<'_> {
    /// CommandSourceStack.getPlayerOrException.
    pub(super) fn source_player(&self) -> R<Target> {
        match self.command_source.map(|s| s.entity) {
            None | Some(Some(Target::Player)) => Ok(Target::Player),
            _ => Err(err("permissions.requires.player", &[])),
        }
    }
    /// EntityArgument.getEntity / getPlayer.
    fn one_entity(&mut self, ctx: &Context, name: &str, player: bool) -> R<Target> {
        Ok(self.entities(ctx, name, player)?[0])
    }

    // ---- /spectate ------------------------------------------------------

    /// `SpectateCommand.spectate`.
    pub(super) fn cmd_spectate(&mut self, ctx: &Context) -> R<i32> {
        let target = if ctx.get("target").is_some() {
            Some(self.one_entity(ctx, "target", false)?)
        } else {
            None
        };
        let player = if ctx.get("player").is_some() {
            self.one_entity(ctx, "player", true)?
        } else {
            self.source_player()?
        };
        if target == Some(player) {
            return Err(err("commands.spectate.self", &[]));
        }
        if !self.spectator() {
            return Err(tr_error(
                "commands.spectate.not_spectator",
                &[self.target_name(player)],
            ));
        }
        // EntityType.clientTrackingRange: only a marker's is zero.
        if let Some(t) = target
            && self.target_type(t) == "minecraft:marker"
        {
            return Err(tr_error(
                "commands.spectate.cannot_spectate",
                &[self.target_name(t)],
            ));
        }
        // A guest's camera is the hub's: the view moves in the guests' tick.
        let guest = match player {
            Target::Player => self.lan_acting,
            Target::Guest(conn) => Some(conn),
            _ => None,
        };
        match guest {
            Some(conn) => {
                let camera = target.and_then(|t| self.network_entity_id(t));
                self.cmds.camera_requests.push((conn, camera));
            }
            None => self.set_camera(target),
        }
        match target {
            Some(t) => {
                let name = self.target_name(t);
                self.feedback(Text::translate(
                    "commands.spectate.success.started",
                    &[name],
                ));
            }
            None => self.feedback(Text::translate("commands.spectate.success.stopped", &[])),
        }
        Ok(1)
    }
    /// `ServerPlayer.setCamera`: a new camera moves the player to it.
    pub(in crate::game) fn set_camera(&mut self, target: Option<Target>) {
        let target = target.filter(|t| *t != Target::Player);
        if self.cmds.camera == target {
            return;
        }
        self.cmds.camera = target;
        if let Some(t) = target
            && let Some(p) = self.target_position(t)
            && let Some(player) = self.player.as_mut()
        {
            player.position = horizon_core::geometry::Vec3::new(p[0], p[1], p[2]);
            self.previous_position = player.position;
        }
    }
    /// Per-tick server work of these commands: the delayed tick profiler
    /// start, queued messages and the spectator camera.
    pub(in crate::game) fn tick_command_extras(&mut self) {
        if self.cmds.profiler_delay {
            self.cmds.profiler_delay = false;
            self.cmds.profiler = Some((Instant::now(), self.ticks));
        }
        self.tick_profile_fetches();
        self.tick_camera();
    }
    /// The camera part of `ServerPlayer.tick`: follow a living camera, drop a
    /// removed one, and stop when the player sneaks or leaves spectator mode.
    fn tick_camera(&mut self) {
        let Some(t) = self.cmds.camera else {
            return;
        };
        if !self.spectator() {
            self.set_camera(None);
            return;
        }
        let sneaking = self.keys.held(Action::Sneak) && self.screen == super::Screen::Playing;
        match self.target_position(t) {
            Some(p) if !sneaking => {
                let rot = self.target_rotation(t);
                if let Some(player) = self.player.as_mut() {
                    player.position = horizon_core::geometry::Vec3::new(p[0], p[1], p[2]);
                }
                self.yaw = rot[0];
                self.pitch = rot[1];
            }
            _ => self.set_camera(None),
        }
    }

    // ---- /posteffect ----------------------------------------------------

    /// `ServerPlayer.postEffects` of a player target: a connected player's
    /// (told to their client at the end of their tick), or the host's own.
    fn post_effects_of(&self, t: Target) -> Vec<String> {
        let conn = match t {
            Target::Guest(conn) => Some(conn),
            Target::Player => self.lan_acting,
            _ => return Vec::new(),
        };
        match conn {
            Some(conn) => self
                .command_guest(conn)
                .map(|g| g.post_effects.clone())
                .unwrap_or_default(),
            None => self.cmds.post_effects.clone(),
        }
    }

    /// Changes a player target's `postEffects`; whether `change` did, which
    /// sets `postEffectsDirty` for the packet.
    fn change_post_effects(
        &mut self,
        t: Target,
        change: impl FnOnce(&mut Vec<String>) -> bool,
    ) -> bool {
        let conn = match t {
            Target::Guest(conn) => Some(conn),
            Target::Player => self.lan_acting,
            _ => return false,
        };
        match conn {
            Some(conn) => {
                let Some(g) = self.lan.as_mut().and_then(|h| h.hub.guest_mut(conn)) else {
                    return false;
                };
                let changed = change(&mut g.post_effects);
                g.post_effects_dirty |= changed;
                changed
            }
            None => change(&mut self.cmds.post_effects),
        }
    }

    pub(super) fn cmd_posteffect(&mut self, id: &str, ctx: &Context) -> R<i32> {
        if id == "posteffect.list" {
            let t = self.one_entity(ctx, "target", true)?;
            let name = self.target_name(t);
            let effects = self.post_effects_of(t);
            if effects.is_empty() {
                self.feedback(Text::translate("commands.posteffect.list.empty", &[name]));
            } else {
                self.feedback(Text::translate(
                    "commands.posteffect.list.success",
                    &[name, num(effects.len()), Text::literal(effects.join(", "))],
                ));
            }
            return Ok(effects.len() as i32);
        }
        let targets = self.entities(ctx, "targets", true)?;
        let effect = match ctx.get("posteffect") {
            Some(Arg::Resource(s)) => s.clone(),
            _ => String::new(),
        };
        let mut changed: Vec<Target> = Vec::new();
        for t in &targets {
            let done = self.change_post_effects(*t, |list| match id {
                // ServerPlayer.addPostEffect.
                "posteffect.add" => {
                    if list.contains(&effect) {
                        false
                    } else {
                        list.push(effect.clone());
                        true
                    }
                }
                // ServerPlayer.removePostEffect.
                "posteffect.remove" => match list.iter().position(|e| *e == effect) {
                    Some(i) => {
                        list.remove(i);
                        true
                    }
                    None => false,
                },
                // ServerPlayer.clearPostEffects.
                _ => !std::mem::take(list).is_empty(),
            });
            if done {
                changed.push(*t);
            }
        }
        let key = id.trim_start_matches("posteffect.");
        if changed.is_empty() {
            return Err(err(&format!("commands.posteffect.{key}.failed"), &[]));
        }
        let n = changed.len() as i32;
        let effect_text = Text::literal(effect);
        let message = match (key, n) {
            ("clear", 1) => Text::translate(
                "commands.posteffect.clear.success.single",
                &[self.target_name(changed[0])],
            ),
            ("clear", _) => {
                Text::translate("commands.posteffect.clear.success.multiple", &[num(n)])
            }
            (_, 1) => Text::translate(
                &format!("commands.posteffect.{key}.success.single"),
                &[effect_text, self.target_name(changed[0])],
            ),
            _ => Text::translate(
                &format!("commands.posteffect.{key}.success.multiple"),
                &[effect_text, num(n)],
            ),
        };
        self.feedback(message);
        Ok(n)
    }

    // ---- /test ----------------------------------------------------------

    /// `TestCommand`: running, resetting and clearing tests is
    /// `test_runs`; `locate`, `pos` and `create` are `test_locate`.
    pub(super) fn cmd_test(&mut self, id: &str, ctx: &Context) -> R<i32> {
        match id {
            "test.run" | "test.run.found" | "test.clear" | "test.stop" => {
                self.cmd_test_runs(id, ctx)
            }
            "test.verify" => self.test_verify(ctx),
            "test.create" => self.test_create(ctx),
            "test.locate" => self.test_locate(ctx),
            _ => self.test_show_pos(ctx),
        }
    }
    /// `Entity.pick(10, 1, false)`: the first non-air block on the view ray,
    /// else the block at the ray's end.
    pub(super) fn test_pick_block(&self) -> [i32; 3] {
        use horizon_core::{
            geometry::{Vec3, traverse_blocks},
            position::BlockPos,
        };
        let Some(p) = self.player.as_ref() else {
            return [0; 3];
        };
        let eye = Vec3::new(
            p.position.x,
            p.position.y + f64::from(p.dimensions.eye_height),
            p.position.z,
        );
        let (yaw, pitch) = (
            f64::from(self.yaw).to_radians(),
            f64::from(self.pitch).to_radians(),
        );
        let dir = [
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        ];
        let end = Vec3::new(
            eye.x + dir[0] * 10.0,
            eye.y + dir[1] * 10.0,
            eye.z + dir[2] * 10.0,
        );
        let hit = traverse_blocks(eye, end, 256, |b: BlockPos| {
            let state = self.world.block(b.x, b.y, b.z);
            (!self.res.facts().is_air(state)).then_some([b.x, b.y, b.z])
        });
        hit.ok().flatten().unwrap_or([
            end.x.floor() as i32,
            end.y.floor() as i32,
            end.z.floor() as i32,
        ])
    }

    // ---- /kick ----------------------------------------------------------

    /// `KickCommand.kickPlayers`: only guests of a published world can be
    /// kicked; the owner never.
    pub(super) fn cmd_kick(&mut self, ctx: &Context) -> R<i32> {
        let selector = match ctx.get("targets") {
            Some(Arg::Entity(s)) => Some(s.clone()),
            _ => None,
        };
        // EntityArgument.getPlayers runs before the published check.
        let host = self.entities(ctx, "targets", true).unwrap_or_default();
        let guests: Vec<(u32, String)> = match (&self.lan, &selector) {
            (Some(lan), Some(s)) => lan
                .hub
                .guests
                .iter()
                .filter(|g| lan.hub.singleplayer_owner != Some(g.uuid))
                .filter(|g| match &s.player_name {
                    Some(name) => g.name.eq_ignore_ascii_case(name),
                    None => s.kind == 'a' || s.kind == 'e',
                })
                .map(|g| (g.conn, g.name.clone()))
                .collect(),
            _ => Vec::new(),
        };
        if host.is_empty() && guests.is_empty() {
            return Err(err("argument.entity.notfound.player", &[]));
        }
        if self.lan.is_none() || (self.serving() && self.published_port().is_none()) {
            return Err(err("commands.kick.singleplayer.failed", &[]));
        }
        let reason_raw = match ctx.get("reason") {
            Some(Arg::Str(s)) => Some(s.clone()),
            _ => None,
        };
        let reason = match &reason_raw {
            Some(raw) => self.expand_selectors(raw),
            None => Text::translate("multiplayer.disconnect.kicked", &[]),
        };
        let tag = match &reason_raw {
            Some(_) => horizon_world::nbt::Tag::String(reason.plain().as_str().into()),
            None => horizon_world::nbt::Tag::compound([(
                "translate",
                horizon_world::nbt::Tag::String("multiplayer.disconnect.kicked".into()),
            )]),
        };
        let mut count = 0;
        for (conn, name) in guests {
            if let Some(lan) = self.lan.as_mut() {
                lan.hub.kick(conn, tag.clone());
            }
            self.feedback(Text::translate(
                "commands.kick.success",
                &[Text::literal(name), reason.clone()],
            ));
            count += 1;
        }
        if count == 0 {
            return Err(err("commands.kick.owner.failed", &[]));
        }
        Ok(count)
    }

    // ---- /fetchprofile --------------------------------------------------

    /// `FetchProfileCommand.resolveName` and `resolveId`: the lookup runs on
    /// background threads (`Util.nonCriticalIoPool`, here the cached profile
    /// resolver's), and the answer is reported by a later tick;
    /// `printForEntity` reports the player's own profile at once.
    pub(super) fn cmd_fetchprofile(&mut self, ctx: &Context) -> R<i32> {
        if ctx.literals.iter().any(|l| l == "name") {
            let name = match ctx.get("name") {
                Some(Arg::Str(s)) => s.clone(),
                _ => String::new(),
            };
            self.cmds.profile_fetches.push(ProfileFetch {
                key: NameOrId::Name(name.clone()),
                kind: "name",
                argument: Text::literal(name),
                source: self.command_source,
            });
            return Ok(1);
        }
        if ctx.literals.iter().any(|l| l == "id") {
            let id = match ctx.get("id") {
                Some(Arg::Str(s)) => horizon_client::command::selector::java_uuid(s),
                _ => None,
            }
            .unwrap_or_default();
            // `Component.translationArg(id)`: the UUID's string form.
            self.cmds.profile_fetches.push(ProfileFetch {
                key: NameOrId::Id(id),
                kind: "id",
                argument: Text::literal(horizon_protocol::crypto::uuid_string(id)),
                source: self.command_source,
            });
            return Ok(1);
        }
        let t = self.one_entity(ctx, "entity", false)?;
        if t != Target::Player {
            return Err(tr_error(
                "commands.fetchprofile.no_profile",
                &[self.target_name(t)],
            ));
        }
        // The player in place: a guest's own profile while their command runs.
        let id = horizon_client::lan::playerdata::ints_uuid(self.acting_uuid());
        let name = self.acting_player_name().to_owned();
        self.report_resolved_profile(
            "commands.fetchprofile.entity.success",
            Text::literal(&name),
            &ResolvedProfile {
                id,
                name,
                properties: Vec::new(),
            },
        );
        Ok(1)
    }
    /// The answers of the profile lookups that have come in
    /// (`server.execute(() -> result.ifPresentOrElse(...))`).
    /// Whether a `/fetchprofile` lookup is still in flight (the command
    /// harness waits for them, as vanilla's oracle resolves at once).
    pub(in crate::game) fn profile_fetches_pending(&mut self) -> bool {
        let mut resolver = self.skin_profiles.borrow_mut();
        self.cmds
            .profile_fetches
            .iter()
            .any(|f| matches!(resolver.state(&f.key), Resolve::Pending))
    }
    fn tick_profile_fetches(&mut self) {
        let fetches = std::mem::take(&mut self.cmds.profile_fetches);
        for fetch in fetches {
            let state = self.skin_profiles.borrow_mut().state(&fetch.key);
            match state {
                Resolve::Pending => self.cmds.profile_fetches.push(fetch),
                Resolve::Done(result) => {
                    let outer = std::mem::replace(&mut self.command_source, fetch.source);
                    match result {
                        Some(profile) => self.report_resolved_profile(
                            &format!("commands.fetchprofile.{}.success", fetch.kind),
                            fetch.argument,
                            &profile,
                        ),
                        None => self.command_failure(Text::translate(
                            &format!("commands.fetchprofile.{}.failure", fetch.kind),
                            &[fetch.argument],
                        )),
                    }
                    self.command_source = outer;
                }
            }
        }
    }
    /// `FetchProfileCommand.reportResolvedProfile`: the profile as
    /// `ResolvableProfile.CODEC` encodes it (`id`, `name`, and `properties`
    /// only when there are some), four clickable actions and the head object,
    /// sent with `sendSuccess(..., false)`.
    fn report_resolved_profile(
        &mut self,
        message_id: &str,
        argument: Text,
        profile: &ResolvedProfile,
    ) {
        let (profile_tag, json) = encode_profile(profile);
        let encoded_profile = snbt::to_snbt(&profile_tag);
        // `Component.object(new PlayerSprite(profile, true))`, encoded: `hat`
        // is left out at its default.
        let encoded_component = snbt::to_snbt(&Tag::compound([("player", profile_tag)]));
        let mut head = Text::literal("\u{FFFC}");
        head.spans[0].style.inline_object = Some(InlineObject::Player {
            profile: json,
            hat: true,
            fallback: format!("[{} head]", profile.name),
        });
        let item = |key: &str, args: &[Text], click: ClickEvent| {
            let inner = Text::translate(key, args)
                .with_color(text::GREEN)
                .with_style(&Style {
                    click: Some(click),
                    ..Style::default()
                });
            Text::translate("chat.square_brackets", &[inner])
        };
        let mut clickable = item(
            "commands.fetchprofile.copy_component",
            &[],
            ClickEvent::CopyToClipboard(encoded_profile.clone()),
        );
        clickable.push(" ");
        clickable.push(item(
            "commands.fetchprofile.give_item",
            &[],
            ClickEvent::RunCommand(format!(
                "give @s minecraft:player_head[profile={encoded_profile}]"
            )),
        ));
        clickable.push(" ");
        clickable.push(item(
            "commands.fetchprofile.summon_mannequin",
            &[],
            ClickEvent::RunCommand(format!(
                "summon minecraft:mannequin ~ ~ ~ {{profile:{encoded_profile}}}"
            )),
        ));
        clickable.push(" ");
        clickable.push(item(
            "commands.fetchprofile.copy_text",
            &[head.with_color(text::WHITE)],
            ClickEvent::CopyToClipboard(encoded_component),
        ));
        self.feedback_unbroadcast(Text::translate(message_id, &[argument, clickable]));
    }

    // ---- /publish, /unpublish -------------------------------------------

    /// `PublishCommand.publish` with `IntegratedServer.publishServer`.
    pub(super) fn cmd_publish(&mut self, ctx: &Context) -> R<i32> {
        let allow = matches!(ctx.get("allowCommands"), Some(Arg::Bool(true)));
        let port = match ctx.get("port") {
            Some(Arg::Int(p)) => *p,
            // HttpUtil.getAvailablePort.
            _ => std::net::TcpListener::bind((
                if std::env::var_os("HORIZON_TEST_LOOPBACK").is_some_and(|v| v == "1") {
                    "127.0.0.1"
                } else {
                    "0.0.0.0"
                },
                0,
            ))
            .and_then(|l| l.local_addr())
            .map_or(25564, |a| i32::from(a.port())),
        };
        let published = if self.serving() {
            self.published_port()
        } else {
            self.lan.as_ref().map(|l| l.settings.port)
        };
        if let Some(port) = published {
            return Err(err("commands.publish.alreadyPublished.lan", &[&port]));
        }
        let settings = super::super::lan::LanSettings {
            port: port as u16,
            guest_commands: allow,
            force_game_mode: false,
        };
        // A dedicated server is open from the start (`published` above).
        let started = self.serving() && self.publish(settings).is_ok();
        if !started {
            return Err(err("commands.publish.failed", &[]));
        }
        let port_text = port.to_string();
        let link = Text::literal(port_text.clone()).with_style(&Style {
            color: Some(text::GREEN),
            hover: Some(Box::new(Text::translate("chat.copy.click", &[]))),
            ..Style::default()
        });
        self.feedback(Text::translate(
            "commands.publish.started.lan",
            &[Text::translate("chat.square_brackets", &[link])],
        ));
        Ok(port)
    }
    pub(super) fn cmd_unpublish(&mut self) -> R<i32> {
        if self.serving() {
            if !self.unpublish() {
                return Err(err("commands.unpublish.notPublished", &[]));
            }
            self.feedback(Text::translate("commands.unpublish.success", &[]));
            return Ok(1);
        }
        if self.lan.is_none() {
            return Err(err("commands.unpublish.notPublished", &[]));
        }
        self.lan_close(false);
        self.feedback(Text::translate("commands.unpublish.success", &[]));
        Ok(1)
    }

    // ---- /debug, /jfr, /version -----------------------------------------

    pub(super) fn cmd_debug(&mut self, ctx: &Context) -> R<i32> {
        if ctx.literals.iter().any(|l| l == "function") {
            return self.cmd_debug_function(ctx);
        }
        // MinecraftServer.isTimeProfilerRunning / startTimeProfiler /
        // stopTimeProfiler: a stop before the delayed start took effect
        // reports empty results and leaves the start pending.
        let running = self.cmds.profiler_delay || self.cmds.profiler.is_some();
        if ctx.literals.iter().any(|l| l == "start") {
            if running {
                return Err(err("commands.debug.alreadyRunning", &[]));
            }
            self.cmds.profiler_delay = true;
            self.feedback(Text::translate("commands.debug.started", &[]));
            return Ok(0);
        }
        if !running {
            return Err(err("commands.debug.notRunning", &[]));
        }
        let (seconds, ticks) = match self.cmds.profiler.take() {
            Some((started, ticks)) => (
                started.elapsed().as_nanos() as f64 / 1.0e9,
                (self.ticks - ticks) as f64,
            ),
            None => (0.0, 0.0),
        };
        let tps = ticks / seconds;
        self.feedback(Text::translate(
            "commands.debug.stopped",
            &[
                Text::literal(two_places(seconds)),
                num(ticks as i64),
                Text::literal(two_places(tps)),
            ],
        ));
        Ok(tps as i32)
    }
    /// `DebugCommand.TraceCustomExecutor`: run the functions with a tracer
    /// writing to `debug/debug-trace-<time>.txt`.
    fn cmd_debug_function(&mut self, ctx: &Context) -> R<i32> {
        if self.exec.function_depth > 0 {
            return Err(err("commands.debug.function.noRecursion", &[]));
        }
        let Some(Arg::Resource(id)) = ctx.get("name").cloned() else {
            return Ok(0);
        };
        let functions = self.functions_named(&id)?;
        let output = format!("debug-trace-{}.txt", filename_date_time());
        let dir = std::env::current_dir().unwrap_or_default().join("debug");
        let mut trace = String::new();
        let mut total = 0;
        let mut traced: Vec<String> = Vec::new();
        for f in &functions {
            let lines = match f.instantiate(None) {
                Ok(lines) => lines,
                Err((key, args)) => {
                    let args: Vec<&dyn std::fmt::Display> =
                        args.iter().map(|s| s as &dyn std::fmt::Display).collect();
                    let reason = Text::tr(&key, &args);
                    self.command_failure(Text::translate(
                        "commands.function.instantiationFailure",
                        &[Text::literal(f.id.clone()), reason],
                    ));
                    continue;
                }
            };
            total += lines.len() as i32;
            traced.push(f.id.clone());
            trace.push_str(&f.id);
            trace.push('\n');
            self.trace_function(&f.id, &lines, 0, &mut trace);
        }
        let written = std::fs::create_dir_all(&dir)
            .and_then(|()| std::fs::write(dir.join(&output), trace))
            .is_ok();
        if !written {
            self.command_failure(Text::translate("commands.debug.function.traceFailed", &[]));
        }
        let message = if traced.len() == 1 {
            Text::translate(
                "commands.debug.function.success.single",
                &[
                    num(total),
                    Text::literal(traced[0].clone()),
                    Text::literal(output),
                ],
            )
        } else {
            Text::translate(
                "commands.debug.function.success.multiple",
                &[num(total), num(traced.len()), Text::literal(output)],
            )
        };
        self.feedback(message);
        Ok(total)
    }
    /// The tracer's `[F]`/`[C]` lines for one function, running its commands.
    fn trace_function(&mut self, id: &str, lines: &[String], depth: usize, out: &mut String) {
        let indent = |d: usize| "    ".repeat(d + 1);
        out.push_str(&format!("{}[F] {id} size={}\n", indent(depth), lines.len()));
        let source = super::execute::Source {
            silent: true,
            ..self.player_source()
        };
        for line in lines {
            out.push_str(&format!("{}[C] {line}", indent(depth + 1)));
            match self.execute_line(line, source) {
                Ok(v) => out.push_str(&format!(" -> {v}\n")),
                Err(e) => {
                    out.push('\n');
                    out.push_str(&format!("{}[E] {}", indent(depth + 2), e.message.plain()));
                }
            }
        }
    }

    /// `/jfr start|stop` (JfrCommand). Horizon has no JVM flight recorder:
    /// the recording is its own tick timing, dumped as JSON.
    pub(super) fn cmd_jfr(&mut self, ctx: &Context) -> R<i32> {
        if ctx.literals.iter().any(|l| l == "start") {
            if self.cmds.jfr.is_some() {
                return Err(err("commands.jfr.start.failed", &[]));
            }
            let stamp = filename_date_time();
            self.cmds.jfr = Some((Instant::now(), self.ticks, stamp));
            self.feedback(Text::translate("commands.jfr.started", &[]));
            return Ok(1);
        }
        let Some((started, ticks, stamp)) = self.cmds.jfr.take() else {
            return Err(tr_error(
                "commands.jfr.dump.failed",
                &[Text::literal("Not currently profiling")],
            ));
        };
        let relative = format!("debug/client-{stamp}.json");
        let path = std::env::current_dir().unwrap_or_default().join(&relative);
        let report = serde_json::json!({
            "environment": "client",
            "seconds": started.elapsed().as_secs_f64(),
            "ticks": self.ticks - ticks,
        });
        let written = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&path, report.to_string()));
        if let Err(e) = written {
            return Err(tr_error(
                "commands.jfr.dump.failed",
                &[Text::literal(e.to_string())],
            ));
        }
        let file = Text::literal(relative).underlined().with_style(&Style {
            hover: Some(Box::new(Text::translate("chat.copy.click", &[]))),
            ..Style::default()
        });
        self.feedback(Text::translate("commands.jfr.stopped", &[file]));
        Ok(1)
    }
}

// `VersionCommand` does not exist in 1.21.5 (added in 1.21.6).
// Neither does `DialogCommand`: `/dialog` arrived in 1.21.6 with it.
impl Game<'_> {}
