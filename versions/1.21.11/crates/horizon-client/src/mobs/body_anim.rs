//! `EntityModel.setupAnim` ports for the animals and monsters whose models
//! have no species file of their own (goat, polar bear, panda, llama, turtle,
//! strider, snow golem, silverfish, endermite, parrot, bee, bat, armadillo,
//! axolotl, frog, rabbit, fox). Each reads the render-state values vanilla's
//! renderer extracts, as plain structs so the differential harness can set
//! them directly; `from_mob` adapters fill them from what a `Mob` tracks.
use super::{
    Mob, Species,
    anim::{Poses, State, quadruped},
    keyframes::{Animation, AnimationState},
};
use horizon_core::geometry::trig::Mth;
use std::{collections::HashMap, f64::consts::PI};

pub type Animations = HashMap<String, Animation>;
const FPI: f32 = PI as f32;
/// `(float) (Math.PI / n)` and friends, evaluated in double like Java.
fn rad(v: f64) -> f32 {
    v as f32
}
fn lerp(a: f32, p: f32, q: f32) -> f32 {
    p + a * (q - p)
}
/// `Mth.rotLerpRad`.
fn rot_lerp_rad(a: f32, from: f32, to: f32) -> f32 {
    let mut diff = to - from;
    while diff < -FPI {
        diff += rad(PI * 2.0);
    }
    while diff >= FPI {
        diff -= rad(PI * 2.0);
    }
    from + a * diff
}
fn head_look(p: &mut Poses, s: &State) {
    p.xrot("head", s.x_rot.to_radians());
    p.yrot("head", s.y_rot.to_radians());
}
/// `KeyframeAnimation.apply(AnimationState, ageInTicks)`.
fn run(animations: &Animations, name: &str, state: &AnimationState, s: &State, p: &mut Poses) {
    if let (Some(ms), Some(a)) = (state.millis(s.age), animations.get(name)) {
        a.apply(ms, 1.0, p);
    }
}
fn walk(animations: &Animations, name: &str, s: &State, factor: f32, scale: f32, p: &mut Poses) {
    if let Some(a) = animations.get(name) {
        a.apply_walk(s.pos, s.speed, factor, scale, p);
    }
}
fn adds(p: &mut Poses, part: &str, i: usize, v: f32) {
    p.get(part)[i] += v;
}

// ---- goat -----------------------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct Goat {
    pub left_horn: bool,
    pub right_horn: bool,
    pub ramming_x_head_rot: f32,
}
impl Goat {
    pub fn from_mob(m: &Mob) -> Self {
        Self {
            left_horn: !m.extra.lost_horns[0],
            right_horn: !m.extra.lost_horns[1],
            ramming_x_head_rot: super::goat::ramming_x_head_rot(m),
        }
    }
}
/// `GoatModel.setupAnim`.
pub fn goat(p: &mut Poses, s: &State, g: &Goat) {
    quadruped(p, s);
    p.visible("left_horn", g.left_horn);
    p.visible("right_horn", g.right_horn);
    if g.ramming_x_head_rot != 0.0 {
        p.xrot("head", g.ramming_x_head_rot);
    }
    // 1.21.11 has no baby branch: the PI/8 head tilt is 26.1's
    // `BabyGoatModel.setupAnim`, and a baby goat's head follows the look.
}

// ---- polar bear -----------------------------------------------------------
/// `PolarBearModel.setupAnim`.
pub fn polar_bear(p: &mut Poses, s: &State, stand_scale: f32) {
    quadruped(p, s);
    let stand = stand_scale * stand_scale;
    let age_scale = if s.baby { 0.5 } else { 1.0 };
    adds(p, "body", 3, -(stand * FPI * 0.35));
    adds(p, "body", 1, stand * age_scale * 2.0);
    adds(p, "right_front_leg", 1, -(stand * age_scale * 20.0));
    adds(p, "right_front_leg", 2, stand * age_scale * 4.0);
    adds(p, "right_front_leg", 3, -(stand * FPI * 0.45));
    let (y, z) = {
        let r = p.get("right_front_leg");
        (r[1], r[2])
    };
    let l = p.get("left_front_leg");
    l[1] = y;
    l[2] = z;
    l[3] -= stand * FPI * 0.45;
    // `PolarBearModel`: the head's stand offsets scale by `1 / BABY_HEAD_SCALE`
    // (4/9) for a cub; the body and legs use `ageScale` instead.
    let head = if s.baby { 0.444_444_45 } else { 1.0 };
    adds(p, "head", 1, -(stand * head * 24.0));
    adds(p, "head", 2, stand * head * 13.0);
    adds(p, "head", 3, stand * FPI * 0.15);
}

// ---- panda ----------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default)]
pub struct Panda {
    pub unhappy: bool,
    pub sneezing: bool,
    pub sneeze_time: i32,
    pub eating: bool,
    pub scared: bool,
    pub sit_amount: f32,
    pub lie_on_back_amount: f32,
    pub roll_amount: f32,
}
/// `PandaModel` / `BabyPandaModel.setupAnim`.
pub fn panda(p: &mut Poses, s: &State, st: &Panda) {
    quadruped(p, s);
    let age = s.age;
    if st.unhappy {
        p.yrot("head", 0.35 * (0.6 * age).msin());
        p.zrot("head", 0.35 * (0.6 * age).msin());
        p.xrot("right_front_leg", -0.75 * (0.3 * age).msin());
        p.xrot("left_front_leg", 0.75 * (0.3 * age).msin());
    } else {
        p.zrot("head", 0.0);
    }
    if st.sneezing {
        if st.sneeze_time < 15 {
            p.xrot("head", rad(-PI / 4.0) * st.sneeze_time as f32 / 14.0);
        } else if st.sneeze_time < 20 {
            let pos = ((st.sneeze_time - 15) / 5) as f32;
            p.xrot("head", rad(-PI / 4.0) + rad(PI / 4.0) * pos);
        }
    }
    if st.sit_amount > 0.0 {
        let a = st.sit_amount;
        if s.baby {
            let x = rot_lerp_rad(a, p.get("body")[3], rad(PI / 18.0));
            p.xrot("body", x);
            let v = lerp(a, p.get("body")[2], -1.5);
            p.get("body")[2] = v;
            let v = lerp(a, p.get("head")[2], -11.5);
            p.get("head")[2] = v;
            let v = lerp(a, p.get("head")[1], 17.5);
            p.get("head")[1] = v;
            for (part, z) in [
                ("right_front_leg", -5.0),
                ("left_front_leg", -5.0),
                ("right_hind_leg", 3.0),
                ("left_hind_leg", 3.0),
            ] {
                let v = lerp(a, p.get(part)[2], z);
                p.get(part)[2] = v;
            }
        } else {
            let x = rot_lerp_rad(a, p.get("body")[3], 1.740_796_3);
            p.xrot("body", x);
            let x = rot_lerp_rad(a, p.get("head")[3], rad(PI / 2.0));
            p.xrot("head", x);
        }
        p.zrot("right_front_leg", -0.270_796_42);
        p.zrot("left_front_leg", 0.270_796_42);
        p.zrot("right_hind_leg", 0.570_796_4);
        p.zrot("left_hind_leg", -0.570_796_4);
        if st.eating {
            p.xrot("head", rad(PI / 2.0) + 0.2 * (age * 0.6).msin());
            p.xrot("right_front_leg", -0.4 - 0.2 * (age * 0.6).msin());
            p.xrot("left_front_leg", -0.4 - 0.2 * (age * 0.6).msin());
        }
        if st.scared {
            p.xrot("head", 2.170_796_4);
            p.xrot("right_front_leg", -0.9);
            p.xrot("left_front_leg", -0.9);
        }
    } else {
        for leg in [
            "right_hind_leg",
            "left_hind_leg",
            "right_front_leg",
            "left_front_leg",
        ] {
            p.zrot(leg, 0.0);
        }
    }
    if st.lie_on_back_amount > 0.0 {
        p.xrot("right_hind_leg", -0.6 * (age * 0.15).msin());
        p.xrot("left_hind_leg", 0.6 * (age * 0.15).msin());
        p.xrot("right_front_leg", 0.3 * (age * 0.25).msin());
        p.xrot("left_front_leg", -0.3 * (age * 0.25).msin());
        let x = rot_lerp_rad(st.lie_on_back_amount, p.get("head")[3], rad(PI / 2.0));
        p.xrot("head", x);
    }
    if st.roll_amount > 0.0 {
        let x = rot_lerp_rad(st.roll_amount, p.get("head")[3], 2.056_194_5);
        p.xrot("head", x);
        p.xrot("right_hind_leg", -0.5 * (age * 0.5).msin());
        p.xrot("left_hind_leg", 0.5 * (age * 0.5).msin());
        p.xrot("right_front_leg", 0.5 * (age * 0.5).msin());
        p.xrot("left_front_leg", -0.5 * (age * 0.5).msin());
    }
}

// ---- llama ----------------------------------------------------------------
/// `LlamaModel.setupAnim`.
pub fn llama(p: &mut Poses, s: &State, has_chest: bool) {
    quadruped(p, s);
    p.visible("right_chest", has_chest);
    p.visible("left_chest", has_chest);
}

// ---- turtle ---------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default)]
pub struct Turtle {
    pub on_land: bool,
    pub laying_egg: bool,
    pub has_egg: bool,
}
/// `TurtleModel` / `AdultTurtleModel.setupAnim`.
pub fn turtle(p: &mut Poses, s: &State, t: &Turtle) {
    quadruped(p, s);
    let (pos, speed) = (s.pos, s.speed);
    if t.on_land {
        let lay = if t.laying_egg { 4.0 } else { 1.0 };
        let amp = if t.laying_egg { 2.0 } else { 1.0 };
        let swing = pos * 5.0;
        let front = (lay * swing).mcos();
        let hind = swing.mcos();
        p.yrot("right_front_leg", -front * 8.0 * speed * amp);
        p.yrot("left_front_leg", front * 8.0 * speed * amp);
        p.yrot("right_hind_leg", -hind * 3.0 * speed);
        p.yrot("left_hind_leg", hind * 3.0 * speed);
    } else {
        let scale = 0.5 * speed;
        let swing = (pos * 0.6662 * 0.6).mcos() * scale;
        p.xrot("right_hind_leg", swing);
        p.xrot("left_hind_leg", -swing);
        p.zrot("right_front_leg", -swing);
        p.zrot("left_front_leg", swing);
    }
    if !s.baby {
        p.visible("egg_belly", t.has_egg);
        if t.has_egg {
            adds(p, "root", 1, -1.0);
        }
    }
}

// ---- strider --------------------------------------------------------------
/// `StriderModel.animateBristle` with the rotation axis the model adds to.
fn bristles(p: &mut Poses, age: f32, flow: f32, parts: [&str; 3], axis: usize) {
    adds(p, parts[0], axis, flow * 0.6);
    adds(p, parts[1], axis, flow * 1.2);
    adds(p, parts[2], axis, flow * 1.3);
    adds(p, parts[0], axis, 0.1 * (age * 0.4).msin());
    adds(p, parts[1], axis, 0.1 * (age * 0.2).msin());
    adds(p, parts[2], axis, 0.05 * (age * -0.4).msin());
}
/// `StriderModel.setupAnim`.
pub fn strider(p: &mut Poses, s: &State, ridden: bool) {
    let pos = s.pos;
    let speed = s.speed.min(0.25);
    if !ridden {
        p.xrot("body", s.x_rot.to_radians());
        p.yrot("body", s.y_rot.to_radians());
    } else {
        p.xrot("body", 0.0);
        p.yrot("body", 0.0);
    }
    p.zrot("body", 0.1 * (pos * 1.5).msin() * 4.0 * speed);
    p.xrot("left_leg", (pos * 1.5 * 0.5).msin() * 2.0 * speed);
    p.xrot("right_leg", (pos * 1.5 * 0.5 + FPI).msin() * 2.0 * speed);
    p.zrot(
        "left_leg",
        rad(PI / 18.0) * (pos * 1.5 * 0.5).mcos() * speed,
    );
    p.zrot(
        "right_leg",
        rad(PI / 18.0) * (pos * 1.5 * 0.5 + FPI).mcos() * speed,
    );
    let flow = (pos * 1.5 + FPI).mcos() * speed;
    // One `StriderModel.setupAnim` serves both ages in 1.21.11: the baby layer
    // is the adult mesh scaled 0.5, with the same six named bristles. The
    // 17.25/20.0 body and leg heights and the `bristle0/1/2` parts are 26.1's
    // `BabyStriderModel`.
    p.zrot("right_bottom_bristle", -1.221_730_5);
    p.zrot("right_middle_bristle", -1.134_464);
    p.zrot("right_top_bristle", -0.872_664_63);
    p.zrot("left_top_bristle", 0.872_664_63);
    p.zrot("left_middle_bristle", 1.134_464);
    p.zrot("left_bottom_bristle", 1.221_730_5);
    bristles(
        p,
        s.age,
        flow,
        [
            "right_top_bristle",
            "right_middle_bristle",
            "right_bottom_bristle",
        ],
        5,
    );
    bristles(
        p,
        s.age,
        flow,
        [
            "left_top_bristle",
            "left_middle_bristle",
            "left_bottom_bristle",
        ],
        5,
    );
    p.get("body")[1] = 2.0;
    p.get("body")[1] -= 2.0 * (pos * 1.5).mcos() * 2.0 * speed;
    p.get("left_leg")[1] = 8.0 + 2.0 * (pos * 1.5 * 0.5 + FPI).msin() * 2.0 * speed;
    p.get("right_leg")[1] = 8.0 + 2.0 * (pos * 1.5 * 0.5).msin() * 2.0 * speed;
}

// ---- snow golem -----------------------------------------------------------
/// `SnowGolemModel.setupAnim`.
pub fn snow_golem(p: &mut Poses, s: &State) {
    p.yrot("head", s.y_rot * rad(PI / 180.0));
    p.xrot("head", s.x_rot * rad(PI / 180.0));
    let y = s.y_rot * rad(PI / 180.0) * 0.25;
    p.yrot("upper_body", y);
    let (sin, cos) = (y.msin(), y.mcos());
    p.yrot("left_arm", y);
    p.yrot("right_arm", y + FPI);
    p.get("left_arm")[0] = cos * 5.0;
    p.get("left_arm")[2] = -sin * 5.0;
    p.get("right_arm")[0] = -cos * 5.0;
    p.get("right_arm")[2] = sin * 5.0;
}

// ---- silverfish and endermite ---------------------------------------------
/// `SilverfishModel.setupAnim`.
pub fn silverfish(p: &mut Poses, s: &State) {
    let mut ys = [0.0f32; 7];
    let mut xs = [0.0f32; 7];
    for i in 0..7usize {
        let d = (i as i32 - 2).abs();
        let phase = s.age * 0.9 + i as f32 * 0.15 * FPI;
        ys[i] = phase.mcos() * FPI * 0.05 * (1 + d) as f32;
        xs[i] = phase.msin() * FPI * 0.2 * d as f32;
        let name = format!("segment{i}");
        p.yrot(&name, ys[i]);
        p.get(&name)[0] = xs[i];
    }
    p.yrot("layer0", ys[2]);
    p.yrot("layer1", ys[4]);
    p.get("layer1")[0] = xs[4];
    p.yrot("layer2", ys[1]);
    p.get("layer2")[0] = xs[1];
}
/// `EndermiteModel.setupAnim`.
pub fn endermite(p: &mut Poses, s: &State) {
    for i in 0..4usize {
        let d = (i as i32 - 2).abs();
        let phase = s.age * 0.9 + i as f32 * 0.15 * FPI;
        let name = format!("segment{i}");
        p.yrot(&name, phase.mcos() * FPI * 0.01 * (1 + d) as f32);
        p.get(&name)[0] = phase.msin() * FPI * 0.1 * d as f32;
    }
}

// ---- parrot ---------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ParrotPose {
    #[default]
    Standing,
    Flying,
    OnShoulder,
    Sitting,
    Party,
}
/// `ParrotModel.setupAnim`.
pub fn parrot(p: &mut Poses, s: &State, pose: ParrotPose, flap_angle: f32) {
    use ParrotPose as P;
    match pose {
        P::Flying => {
            adds(p, "left_leg", 3, rad(PI * 2.0 / 9.0));
            adds(p, "right_leg", 3, rad(PI * 2.0 / 9.0));
        }
        P::Sitting => {
            adds(p, "head", 1, 1.9);
            adds(p, "tail", 3, rad(PI / 6.0));
            adds(p, "tail", 1, 1.9);
            adds(p, "body", 1, 1.9);
            p.zrot("left_wing", -0.0873);
            adds(p, "left_wing", 1, 1.9);
            p.zrot("right_wing", 0.0873);
            adds(p, "right_wing", 1, 1.9);
            adds(p, "left_leg", 1, 1.9);
            adds(p, "right_leg", 1, 1.9);
            adds(p, "left_leg", 3, rad(PI / 2.0));
            adds(p, "right_leg", 3, rad(PI / 2.0));
        }
        P::Party => {
            p.zrot("left_leg", rad(-PI / 9.0));
            p.zrot("right_leg", rad(PI / 9.0));
        }
        _ => {}
    }
    head_look(p, s);
    let flutter = |p: &mut Poses| {
        let bob = flap_angle * 0.3;
        adds(p, "head", 1, bob);
        adds(p, "tail", 3, (s.pos * 0.6662).mcos() * 0.3 * s.speed);
        adds(p, "tail", 1, bob);
        adds(p, "body", 1, bob);
        p.zrot("left_wing", -0.0873 - flap_angle);
        adds(p, "left_wing", 1, bob);
        p.zrot("right_wing", 0.0873 + flap_angle);
        adds(p, "right_wing", 1, bob);
        adds(p, "left_leg", 1, bob);
        adds(p, "right_leg", 1, bob);
    };
    match pose {
        P::Standing => {
            adds(p, "left_leg", 3, (s.pos * 0.6662).mcos() * 1.4 * s.speed);
            adds(
                p,
                "right_leg",
                3,
                (s.pos * 0.6662 + FPI).mcos() * 1.4 * s.speed,
            );
            flutter(p);
        }
        P::Flying | P::OnShoulder => flutter(p),
        P::Sitting => {}
        P::Party => {
            let (x, y) = (s.age.mcos(), s.age.msin());
            adds(p, "head", 0, x);
            adds(p, "head", 1, y);
            p.xrot("head", 0.0);
            p.yrot("head", 0.0);
            p.zrot("head", s.age.msin() * 0.4);
            adds(p, "body", 0, x);
            adds(p, "body", 1, y);
            p.zrot("left_wing", -0.0873 - flap_angle);
            adds(p, "left_wing", 0, x);
            adds(p, "left_wing", 1, y);
            p.zrot("right_wing", 0.0873 + flap_angle);
            adds(p, "right_wing", 0, x);
            adds(p, "right_wing", 1, y);
            adds(p, "tail", 0, x);
            adds(p, "tail", 1, y);
        }
    }
}

// ---- bee ------------------------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct Bee {
    pub has_stinger: bool,
    pub on_ground: bool,
    pub angry: bool,
    pub roll_amount: f32,
}
/// `BeeModel.setupAnim`.
pub fn bee(p: &mut Poses, s: &State, b: &Bee) {
    p.visible("stinger", b.has_stinger);
    let age = s.age;
    if !b.on_ground {
        let speed = age * 120.32113 * rad(PI / 180.0);
        p.yrot("right_wing", 0.0);
        p.zrot("right_wing", speed.mcos() * FPI * 0.15);
        let (x, y, z) = {
            let r = p.get("right_wing");
            (r[3], r[4], r[5])
        };
        p.xrot("left_wing", x);
        p.yrot("left_wing", y);
        p.zrot("left_wing", -z);
        p.xrot("front_legs", rad(PI / 4.0));
        p.xrot("middle_legs", rad(PI / 4.0));
        p.xrot("back_legs", rad(PI / 4.0));
    }
    if !b.angry && !b.on_ground {
        let speed = (age * 0.18).mcos();
        p.xrot("bone", 0.1 + speed * FPI * 0.025);
        adds(p, "bone", 1, -((age * 0.18).mcos() * 0.9));
        p.xrot("front_legs", -speed * FPI * 0.1 + rad(PI / 8.0));
        p.xrot("back_legs", -speed * FPI * 0.05 + rad(PI / 4.0));
        // The antennae animate at every age: 1.21.11's one `BeeModel` carries
        // them, and the baby mesh is the adult mesh scaled 0.5. Only 26.1's
        // base `BeeModel` lacks them (they live on `AdultBeeModel`).
        p.xrot("left_antenna", speed * FPI * 0.03);
        p.xrot("right_antenna", speed * FPI * 0.03);
    }
    if b.roll_amount > 0.0 {
        let x = rot_lerp_rad(b.roll_amount, p.get("bone")[3], 3.091_592_8);
        p.xrot("bone", x);
    }
}

// ---- bat ------------------------------------------------------------------
/// `BatModel.setupAnim`.
pub fn bat(
    p: &mut Poses,
    s: &State,
    resting: bool,
    fly: &AnimationState,
    rest: &AnimationState,
    a: &Animations,
) {
    if resting {
        p.yrot("head", s.y_rot * rad(PI / 180.0));
    }
    run(a, "BatAnimation.BAT_FLYING", fly, s, p);
    run(a, "BatAnimation.BAT_RESTING", rest, s, p);
}

// ---- armadillo ------------------------------------------------------------
#[derive(Clone, Debug, Default)]
pub struct Armadillo {
    pub hiding_in_shell: bool,
    pub roll_out: AnimationState,
    pub roll_up: AnimationState,
    pub peek: AnimationState,
}
/// `ArmadilloModel.setupAnim`.
pub fn armadillo(p: &mut Poses, s: &State, st: &Armadillo, a: &Animations) {
    if st.hiding_in_shell {
        p.skip_draw("body");
        p.visible("left_hind_leg", false);
        p.visible("right_hind_leg", false);
        p.visible("tail", false);
    } else {
        p.visible("cube", false);
        p.xrot("head", s.x_rot.clamp(-22.5, 25.0) * rad(PI / 180.0));
        p.yrot("head", s.y_rot.clamp(-32.5, 32.5) * rad(PI / 180.0));
    }
    // 1.21.11 applies the walk even while hiding (26.1 gates it on
    // `!isHidingInShell`), and one `ArmadilloAnimation` serves both ages:
    // `BabyArmadilloAnimation` is 26.1-only, so the baby prefix resolved to
    // nothing and baby armadillos never animated.
    walk(a, "ArmadilloAnimation.ARMADILLO_WALK", s, 16.5, 2.5, p);
    run(
        a,
        "ArmadilloAnimation.ARMADILLO_ROLL_OUT",
        &st.roll_out,
        s,
        p,
    );
    run(a, "ArmadilloAnimation.ARMADILLO_ROLL_UP", &st.roll_up, s, p);
    run(a, "ArmadilloAnimation.ARMADILLO_PEEK", &st.peek, s, p);
}

// ---- axolotl --------------------------------------------------------------
#[derive(Clone, Debug, Default)]
pub struct Axolotl {
    pub playing_dead: f32,
    pub in_water: f32,
    pub on_ground: f32,
    pub moving: f32,
    pub swim: AnimationState,
    pub walk: AnimationState,
    pub idle_on_ground: AnimationState,
    pub idle_underwater_on_ground: AnimationState,
    pub idle_underwater: AnimationState,
    pub play_dead: AnimationState,
}
/// `AdultAxolotlModel` / `BabyAxolotlModel.setupAnim`.
pub fn axolotl(p: &mut Poses, s: &State, x: &Axolotl, a: &Animations) {
    if s.baby {
        if x.walk.started.is_some() {
            walk(
                a,
                "BabyAxolotlAnimation.AXOLOTL_WALK_FLOOR",
                s,
                15.0,
                30.0,
                p,
            );
        }
        run(a, "BabyAxolotlAnimation.BABY_AXOLOTL_SWIM", &x.swim, s, p);
        run(
            a,
            "BabyAxolotlAnimation.WALK_FLOOR_UNDERWATER",
            // The model reads `walkAnimationState` again here, not the
            // underwater state the renderer also copies.
            &x.walk,
            s,
            p,
        );
        run(
            a,
            "BabyAxolotlAnimation.BABY_AXOLOTL_IDLE_FLOOR",
            &x.idle_on_ground,
            s,
            p,
        );
        run(
            a,
            "BabyAxolotlAnimation.IDLE_UNDERWATER",
            &x.idle_underwater,
            s,
            p,
        );
        run(
            a,
            "BabyAxolotlAnimation.IDLE_FLOOR_UNDERWATER",
            &x.idle_underwater_on_ground,
            s,
            p,
        );
        run(
            a,
            "BabyAxolotlAnimation.BABY_AXOLOTL_PLAY_DEAD",
            &x.play_dead,
            s,
            p,
        );
        return;
    }
    let not_moving = 1.0 - x.moving;
    let mirrored = 1.0 - x.on_ground.min(x.moving);
    adds(p, "body", 4, s.y_rot * rad(PI / 180.0));
    axolotl_swim(p, s.age, s.x_rot, x.moving.min(x.in_water));
    axolotl_hover(p, s.age, not_moving.min(x.in_water));
    axolotl_crawl(p, s.age, x.moving.min(x.on_ground));
    axolotl_lay(p, s.age, not_moving.min(x.on_ground));
    axolotl_dead(p, x.playing_dead);
    if mirrored > 1.0e-5 {
        let l_hind = [
            p.get("left_hind_leg")[3],
            p.get("left_hind_leg")[4],
            p.get("left_hind_leg")[5],
        ];
        let l_front = [
            p.get("left_front_leg")[3],
            p.get("left_front_leg")[4],
            p.get("left_front_leg")[5],
        ];
        for (part, l) in [("right_hind_leg", l_hind), ("right_front_leg", l_front)] {
            let r = p.get(part);
            r[3] += l[0] * mirrored;
            r[4] += -l[1] * mirrored;
            r[5] += -l[2] * mirrored;
        }
    }
}
fn axolotl_lay(p: &mut Poses, age: f32, factor: f32) {
    if factor <= 1.0e-5 {
        return;
    }
    let sway = age * 0.09;
    let sine = sway.msin();
    let cosine = sway.mcos();
    let movement = sine * sine - 2.0 * sine;
    let movement2 = cosine * cosine - 3.0 * sine;
    adds(p, "head", 3, -0.09 * movement * factor);
    adds(p, "head", 5, -0.2 * factor);
    adds(p, "tail", 4, (-0.1 + 0.1 * movement) * factor);
    let gill = (0.6 + 0.05 * movement2) * factor;
    adds(p, "top_gills", 3, gill);
    adds(p, "left_gills", 4, -gill);
    adds(p, "right_gills", 4, gill);
    adds(p, "left_hind_leg", 3, 1.1 * factor);
    adds(p, "left_hind_leg", 4, 1.0 * factor);
    adds(p, "left_front_leg", 3, 0.8 * factor);
    adds(p, "left_front_leg", 4, 2.3 * factor);
    adds(p, "left_front_leg", 5, -0.5 * factor);
}
fn axolotl_crawl(p: &mut Poses, age: f32, factor: f32) {
    if factor <= 1.0e-5 {
        return;
    }
    let sway = age * 0.11;
    let cosine = sway.mcos();
    let hind_y = (cosine * cosine - 2.0 * cosine) / 5.0;
    let front_y = 0.7 * cosine;
    let head_tail = 0.09 * cosine * factor;
    adds(p, "head", 4, head_tail);
    adds(p, "tail", 4, head_tail);
    let gill = (0.6 - 0.08 * (cosine * cosine + 2.0 * sway.msin())) * factor;
    adds(p, "top_gills", 3, gill);
    adds(p, "left_gills", 4, -gill);
    adds(p, "right_gills", 4, gill);
    let hind_x = 0.942_477_9 * factor;
    let front_x = 1.099_557_4 * factor;
    adds(p, "left_hind_leg", 3, hind_x);
    adds(p, "left_hind_leg", 4, (1.5 - hind_y) * factor);
    adds(p, "left_hind_leg", 5, -0.1 * factor);
    adds(p, "left_front_leg", 3, front_x);
    adds(p, "left_front_leg", 4, (rad(PI / 2.0) - front_y) * factor);
    adds(p, "right_hind_leg", 3, hind_x);
    adds(p, "right_hind_leg", 4, (-1.0 - hind_y) * factor);
    adds(p, "right_front_leg", 3, front_x);
    adds(p, "right_front_leg", 4, (rad(-PI / 2.0) - front_y) * factor);
}
fn axolotl_hover(p: &mut Poses, age: f32, factor: f32) {
    if factor <= 1.0e-5 {
        return;
    }
    let sway = age * 0.075;
    let cosine = sway.mcos();
    let sine = sway.msin() * 0.15;
    let body_x = (-0.15 + 0.075 * cosine) * factor;
    adds(p, "body", 3, body_x);
    adds(p, "body", 1, -(sine * factor));
    adds(p, "head", 3, -body_x);
    adds(p, "top_gills", 3, 0.2 * cosine * factor);
    let gill_y = (-0.3 * cosine - 0.19) * factor;
    adds(p, "left_gills", 4, gill_y);
    adds(p, "right_gills", 4, -gill_y);
    adds(
        p,
        "left_hind_leg",
        3,
        (rad(PI * 3.0 / 4.0) - cosine * 0.11) * factor,
    );
    adds(p, "left_hind_leg", 4, 0.471_238_94 * factor);
    adds(p, "left_hind_leg", 5, 1.727_876_1 * factor);
    adds(
        p,
        "left_front_leg",
        3,
        (rad(PI / 4.0) - cosine * 0.2) * factor,
    );
    adds(p, "left_front_leg", 4, 2.042_035 * factor);
    adds(p, "tail", 4, 0.5 * cosine * factor);
}
fn axolotl_swim(p: &mut Poses, age: f32, x_rot: f32, factor: f32) {
    if factor <= 1.0e-5 {
        return;
    }
    let sway = age * 0.33;
    let sine = sway.msin();
    let cosine = sway.mcos();
    let body_sway = 0.13 * sine;
    adds(p, "body", 3, (x_rot * rad(PI / 180.0) + body_sway) * factor);
    adds(p, "head", 3, -(body_sway * 1.8 * factor));
    adds(p, "body", 1, -(0.45 * cosine * factor));
    adds(p, "top_gills", 3, (-0.5 * sine - 0.8) * factor);
    let gill_y = (0.3 * sine + 0.9) * factor;
    adds(p, "left_gills", 4, gill_y);
    adds(p, "right_gills", 4, -gill_y);
    let tail = p.get("tail")[4] + 0.3 * (sway * 0.9).mcos() * factor;
    p.yrot("tail", tail);
    adds(p, "left_hind_leg", 3, 1.884_955_8 * factor);
    adds(p, "left_hind_leg", 4, -0.4 * sine * factor);
    adds(p, "left_hind_leg", 5, rad(PI / 2.0) * factor);
    adds(p, "left_front_leg", 3, 1.884_955_8 * factor);
    adds(p, "left_front_leg", 4, (-0.2 * cosine - 0.1) * factor);
    adds(p, "left_front_leg", 5, rad(PI / 2.0) * factor);
}
fn axolotl_dead(p: &mut Poses, factor: f32) {
    if factor <= 1.0e-5 {
        return;
    }
    adds(p, "left_hind_leg", 3, 1.413_716_7 * factor);
    adds(p, "left_hind_leg", 4, 1.099_557_4 * factor);
    adds(p, "left_hind_leg", 5, rad(PI / 4.0) * factor);
    adds(p, "left_front_leg", 3, rad(PI / 4.0) * factor);
    adds(p, "left_front_leg", 4, 2.042_035 * factor);
    adds(p, "body", 3, -0.15 * factor);
    adds(p, "body", 5, 0.35 * factor);
}

// ---- frog -----------------------------------------------------------------
#[derive(Clone, Debug, Default)]
pub struct Frog {
    pub swimming: bool,
    pub jump: AnimationState,
    pub croak: AnimationState,
    pub tongue: AnimationState,
    pub swim_idle: AnimationState,
}
/// `FrogModel.setupAnim`.
pub fn frog(p: &mut Poses, s: &State, f: &Frog, a: &Animations) {
    run(a, "FrogAnimation.FROG_JUMP", &f.jump, s, p);
    run(a, "FrogAnimation.FROG_CROAK", &f.croak, s, p);
    run(a, "FrogAnimation.FROG_TONGUE", &f.tongue, s, p);
    if f.swimming {
        walk(a, "FrogAnimation.FROG_SWIM", s, 1.0, 2.5, p);
    } else {
        walk(a, "FrogAnimation.FROG_WALK", s, 1.5, 2.5, p);
    }
    run(a, "FrogAnimation.FROG_IDLE_WATER", &f.swim_idle, s, p);
    p.visible("croaking_body", f.croak.started.is_some());
}

// ---- rabbit ---------------------------------------------------------------
/// `RabbitModel.setupAnim`: the classic procedural hop. 1.21.11 has no
/// keyframe animations for rabbits — the hop/idle-tilt keyframes and the
/// entity's `hopAnimationState`/`idleHeadTiltAnimationState` are 26.1-only —
/// so the head always follows the look and the legs ride `jumpCompletion`.
pub fn rabbit(p: &mut Poses, s: &State, jump_completion: f32) {
    p.yrot("head", s.y_rot * rad(PI / 180.0));
    p.xrot("head", s.x_rot * rad(PI / 180.0));
    let hop = (jump_completion * FPI).msin();
    adds(p, "left_haunch", 3, hop * rad(50.0 * PI / 180.0));
    adds(p, "right_haunch", 3, hop * rad(50.0 * PI / 180.0));
    adds(p, "left_front_leg", 3, hop * rad(-40.0 * PI / 180.0));
    adds(p, "right_front_leg", 3, hop * rad(-40.0 * PI / 180.0));
}

// ---- fox ------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default)]
pub struct Fox {
    pub crouching: bool,
    pub sleeping: bool,
    pub sitting: bool,
    pub faceplanted: bool,
    pub head_roll_angle: f32,
    pub crouch_amount: f32,
    /// `FoxModel.legMotionPos` after this frame's `+= 0.67`.
    pub leg_motion_pos: f32,
}
/// `FoxModel.setupAnim`: one form for both ages, scaled by `ageScale`
/// (0.5 for a kit). The `FoxBabyAnimation` walk, the bespoke baby sleep/sit
/// poses and the pouncing pose are 26.1's `AdultFoxModel`/`BabyFoxModel`;
/// 1.21.11's single model has no pouncing pose at all.
pub fn fox(p: &mut Poses, s: &State, f: &Fox) {
    let baby = s.baby;
    let age_scale = if baby { 0.5 } else { 1.0 };
    let (pos, speed) = (s.pos, s.speed);
    p.xrot("right_hind_leg", (pos * 0.6662).mcos() * 1.4 * speed);
    p.xrot("left_hind_leg", (pos * 0.6662 + FPI).mcos() * 1.4 * speed);
    p.xrot("right_front_leg", (pos * 0.6662 + FPI).mcos() * 1.4 * speed);
    p.xrot("left_front_leg", (pos * 0.6662).mcos() * 1.4 * speed);
    p.zrot("head", f.head_roll_angle);
    if f.crouching {
        adds(p, "body", 3, 0.104_719_76);
        adds(p, "body", 1, f.crouch_amount * age_scale);
        adds(p, "head", 1, f.crouch_amount * age_scale);
    } else if f.sleeping {
        for leg in [
            "right_hind_leg",
            "left_hind_leg",
            "right_front_leg",
            "left_front_leg",
        ] {
            p.visible(leg, false);
        }
        p.zrot("body", rad(-PI / 2.0));
        adds(p, "body", 1, 5.0 * age_scale);
        if baby {
            // A sleeping kit curls its tail further and shifts the body.
            p.xrot("tail", -2.181_661_6);
            adds(p, "body", 2, 2.0);
        } else {
            p.xrot("tail", rad(-PI * 5.0 / 6.0));
        }
        adds(p, "head", 0, 2.0 * age_scale);
        adds(p, "head", 1, 2.99 * age_scale);
        p.yrot("head", rad(-PI * 2.0 / 3.0));
        p.zrot("head", 0.0);
    } else if f.sitting {
        p.xrot("body", rad(PI / 6.0));
        adds(p, "body", 1, -7.0 * age_scale);
        adds(p, "body", 2, 3.0 * age_scale);
        p.xrot("tail", rad(PI / 4.0));
        adds(p, "tail", 2, -1.0 * age_scale);
        p.xrot("head", 0.0);
        p.yrot("head", 0.0);
        if baby {
            adds(p, "head", 1, -1.75);
            adds(p, "head", 2, -0.375);
        } else {
            adds(p, "head", 1, -6.5);
            adds(p, "head", 2, 2.75);
        }
        p.xrot("right_hind_leg", rad(-PI * 5.0 / 12.0));
        adds(p, "right_hind_leg", 1, 4.0 * age_scale);
        adds(p, "right_hind_leg", 2, -0.25 * age_scale);
        p.xrot("left_hind_leg", rad(-PI * 5.0 / 12.0));
        adds(p, "left_hind_leg", 1, 4.0 * age_scale);
        adds(p, "left_hind_leg", 2, -0.25 * age_scale);
        p.xrot("right_front_leg", rad(-PI / 12.0));
        p.xrot("left_front_leg", rad(-PI / 12.0));
    }
    if !f.sleeping && !f.faceplanted && !f.crouching {
        p.xrot("head", s.x_rot * rad(PI / 180.0));
        p.yrot("head", s.y_rot * rad(PI / 180.0));
    }
    if f.sleeping {
        p.xrot("head", 0.0);
        p.yrot("head", rad(-PI * 2.0 / 3.0));
        p.zrot("head", (s.age * 0.027).mcos() / 22.0);
    }
    if f.crouching {
        // 1.21.11's wiggle is `cos(ageInTicks) * 0.01`; 26.1's base uses 0.05.
        let wiggle = s.age.mcos() * 0.01;
        p.yrot("body", wiggle);
        p.zrot("right_hind_leg", wiggle);
        p.zrot("left_hind_leg", wiggle);
        p.zrot("right_front_leg", wiggle / 2.0);
        p.zrot("left_front_leg", wiggle / 2.0);
    }
    if f.faceplanted {
        let m = f.leg_motion_pos;
        p.xrot("right_hind_leg", (m * 0.4662).mcos() * 0.1);
        p.xrot("left_hind_leg", (m * 0.4662 + FPI).mcos() * 0.1);
        p.xrot("right_front_leg", (m * 0.4662 + FPI).mcos() * 0.1);
        p.xrot("left_front_leg", (m * 0.4662).mcos() * 0.1);
    }
}

// ---- adapters ---------------------------------------------------------------
/// Dispatch for the species ported here; `false` when `m` is not one of them.
pub fn anim(m: &Mob, s: &State, p: &mut Poses, a: &Animations) -> bool {
    let st = &m.anim.states;
    match m.species() {
        Species::Goat => goat(p, s, &Goat::from_mob(m)),
        Species::PolarBear => polar_bear(p, s, super::polar_bear::standing_scale(m, s.partial)),
        Species::Panda => panda(p, s, &super::panda::pose(m, s.partial)),
        Species::Llama | Species::TraderLlama => {
            llama(p, s, m.extra.chested && !s.baby);
        }
        Species::Turtle => turtle(
            p,
            s,
            &Turtle {
                on_land: !m.fluids.in_water && m.on_ground,
                laying_egg: m.extra.flag2,
                has_egg: m.extra.flag,
            },
        ),
        Species::Strider => strider(p, s, m.rider.is_some()),
        Species::SnowGolem => snow_golem(p, s),
        Species::Silverfish => silverfish(p, s),
        Species::Endermite => endermite(p, s),
        Species::Parrot => {
            let flap = m.anim.flap_o + (m.anim.flap - m.anim.flap_o) * s.partial;
            let speed = m.anim.flap_speed_o + (m.anim.flap_speed - m.anim.flap_speed_o) * s.partial;
            // `Parrot.isFlying` is `!onGround`, in water too.
            let pose = if m.extra.party_jukebox.is_some() {
                ParrotPose::Party
            } else if m.extra.sitting {
                ParrotPose::Sitting
            } else if !m.on_ground {
                ParrotPose::Flying
            } else {
                ParrotPose::Standing
            };
            parrot(p, s, pose, (flap.msin() + 1.0) * speed);
        }
        Species::Bee => bee(
            p,
            s,
            &Bee {
                has_stinger: !super::bee::has_stung(m),
                on_ground: m.on_ground && m.velocity.length_squared() < 1.0e-7,
                // The anger end time is cleared the tick after it passes.
                angry: super::bee::get(m).is_some_and(|b| b.anger_end > 0),
                // Bee.getRollAmount(partialTick).
                roll_amount: super::bee::get(m)
                    .map_or(0.0, |b| b.roll_o + (b.roll - b.roll_o) * s.partial),
            },
        ),
        Species::Bat => bat(p, s, m.extra.flag, &st[0], &st[1], a),
        Species::Armadillo => armadillo(
            p,
            s,
            &Armadillo {
                hiding_in_shell: super::armadillo::hiding_in_shell(m),
                roll_out: st[0],
                roll_up: st[1],
                peek: st[2],
            },
            a,
        ),
        Species::Axolotl => axolotl(p, s, &Axolotl::from_mob(m, s.partial), a),
        Species::Frog => frog(
            p,
            s,
            &Frog {
                swimming: m.fluids.in_water,
                jump: st[0],
                croak: st[1],
                tongue: st[2],
                swim_idle: st[3],
            },
            a,
        ),
        Species::Rabbit => rabbit(p, s, super::rabbit::jump_completion(m, s.partial)),
        Species::Fox => fox(p, s, &super::fox::pose(m, s.partial)),
        _ => return false,
    }
    true
}
impl Axolotl {
    pub fn from_mob(m: &Mob, partial: f32) -> Self {
        let x = &m.extra.axolotl;
        Self {
            playing_dead: x.factor(0, partial),
            in_water: x.factor(1, partial),
            on_ground: x.factor(2, partial),
            moving: x.factor(3, partial),
            // Native baby model deliberately reads walkAnimationState, not its underwater counterpart.
            walk: x.states[0],
            swim: x.states[1],
            idle_on_ground: x.states[3],
            idle_underwater: x.states[4],
            idle_underwater_on_ground: x.states[5],
            play_dead: x.states[6],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobs::Part;
    use serde_json::json;
    use std::sync::Arc;

    fn model(children: &[&str]) -> Arc<Part> {
        let mut map = serde_json::Map::new();
        for c in children {
            map.insert((*c).to_string(), json!({}));
        }
        Arc::new(Part::parse("root", &json!({ "children": map })))
    }

    fn state(baby: bool) -> State {
        State {
            pos: 0.0,
            speed: 0.0,
            age: 0.0,
            y_rot: 0.0,
            x_rot: 0.0,
            swing: 0.0,
            baby,
            partial: 0.0,
        }
    }

    fn near(a: f32, b: f32) {
        assert!((a - b).abs() < 1.0e-4, "{a} != {b}");
    }

    #[test]
    fn a_baby_polar_bears_head_stands_by_four_ninths() {
        let m = model(&[
            "head",
            "body",
            "right_front_leg",
            "left_front_leg",
            "right_hind_leg",
            "left_hind_leg",
        ]);
        let mut p = Poses::new(&m);
        polar_bear(&mut p, &state(true), 1.0);
        // `PolarBearModel`: the head offsets scale by `1 / BABY_HEAD_SCALE`.
        near(p.get("head")[1], -0.444_444_45 * 24.0);
        near(p.get("head")[2], 0.444_444_45 * 13.0);
        let mut p = Poses::new(&m);
        polar_bear(&mut p, &state(false), 1.0);
        near(p.get("head")[1], -24.0);
        near(p.get("head")[2], 13.0);
    }

    #[test]
    fn a_baby_strider_runs_the_adult_animation() {
        let m = model(&[
            "body",
            "left_leg",
            "right_leg",
            "right_bottom_bristle",
            "right_middle_bristle",
            "right_top_bristle",
            "left_top_bristle",
            "left_middle_bristle",
            "left_bottom_bristle",
        ]);
        let mut p = Poses::new(&m);
        strider(&mut p, &state(true), false);
        // 26.1's `BabyStriderModel` used 17.25 / 20.0 and `bristle0/1/2`.
        near(p.get("body")[1], 2.0);
        near(p.get("left_leg")[1], 8.0);
        near(p.get("right_leg")[1], 8.0);
        near(p.get("right_bottom_bristle")[5], -1.221_730_5);
        near(p.get("left_top_bristle")[5], 0.872_664_63);
    }

    #[test]
    fn a_baby_fox_sleeps_and_sits_at_half_scale() {
        let m = model(&[
            "head",
            "body",
            "tail",
            "right_hind_leg",
            "left_hind_leg",
            "right_front_leg",
            "left_front_leg",
        ]);
        let mut p = Poses::new(&m);
        fox(
            &mut p,
            &state(true),
            &Fox {
                sleeping: true,
                ..Fox::default()
            },
        );
        near(p.get("body")[1], 2.5);
        near(p.get("body")[2], 2.0);
        near(p.get("tail")[3], -2.181_661_6);
        near(p.get("head")[0], 1.0);
        near(p.get("head")[1], 1.495);
        let mut p = Poses::new(&m);
        fox(
            &mut p,
            &state(true),
            &Fox {
                sitting: true,
                ..Fox::default()
            },
        );
        near(p.get("body")[1], -3.5);
        near(p.get("head")[1], -1.75);
        near(p.get("head")[2], -0.375);
    }

    #[test]
    fn a_crouching_fox_wiggles_by_one_hundredth() {
        let m = model(&[
            "head",
            "body",
            "tail",
            "right_hind_leg",
            "left_hind_leg",
            "right_front_leg",
            "left_front_leg",
        ]);
        let mut p = Poses::new(&m);
        fox(
            &mut p,
            &state(false),
            &Fox {
                crouching: true,
                crouch_amount: 1.0,
                ..Fox::default()
            },
        );
        // 1.21.11's wiggle is `cos(ageInTicks) * 0.01`; 26.1's base uses 0.05.
        near(p.get("body")[4], 0.01);
        near(p.get("right_hind_leg")[5], 0.01);
        near(p.get("right_front_leg")[5], 0.005);
    }

    #[test]
    fn a_baby_bees_antennae_bob() {
        let m = model(&[
            "bone",
            "stinger",
            "front_legs",
            "middle_legs",
            "back_legs",
            "left_antenna",
            "right_antenna",
            "left_wing",
            "right_wing",
        ]);
        let mut p = Poses::new(&m);
        bee(
            &mut p,
            &state(true),
            &Bee {
                has_stinger: true,
                on_ground: false,
                angry: false,
                roll_amount: 0.0,
            },
        );
        near(p.get("left_antenna")[3], FPI * 0.03);
        near(p.get("right_antenna")[3], FPI * 0.03);
    }

    #[test]
    fn a_baby_goat_has_no_head_tilt() {
        let m = model(&[
            "head",
            "body",
            "left_horn",
            "right_horn",
            "right_hind_leg",
            "left_hind_leg",
            "right_front_leg",
            "left_front_leg",
        ]);
        let mut p = Poses::new(&m);
        goat(
            &mut p,
            &state(true),
            &Goat {
                left_horn: true,
                right_horn: true,
                ramming_x_head_rot: 0.0,
            },
        );
        // 26.1's `BabyGoatModel` tilted the head by PI/8 when not ramming.
        near(p.get("head")[3], 0.0);
    }
}
