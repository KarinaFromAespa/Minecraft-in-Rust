//! The Rust side of tools/test_interpolation_parity.py: replays the
//! harness's events (VanillaInterpolationOracle's input format) on
//! `InterpolationHandler` and writes the entity after each one.
use horizon_world::interpolation::{InterpolationHandler, PosRot, Start};
use std::fmt::Write as _;

/// `Entity.setRot`: yaw and pitch wrapped, pitch clamped (`setXRot`).
fn set_rot(entity: &mut PosRot, yaw: f32, pitch: f32) {
    entity.yaw = yaw % 360.0;
    entity.pitch = (pitch % 360.0 % 360.0).clamp(-90.0, 90.0);
}

pub fn main() {
    let args: Vec<String> = crate::args().collect();
    let input = std::fs::read_to_string(&args[1]).expect("input");
    let mut out = String::new();
    let mut handler = InterpolationHandler::new(0);
    let mut entity = PosRot::default();
    for line in input.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize| f[i].parse::<f64>().expect("number");
        let what = match f[0] {
            "case" => {
                handler = InterpolationHandler::new(f[1].parse().expect("steps"));
                entity = PosRot {
                    position: [num(2), num(3), num(4)],
                    yaw: f[5].parse().expect("yaw"),
                    pitch: f[6].parse().expect("pitch"),
                };
                "c"
            }
            "move" => {
                let mut i = 2;
                let position = match f[1] {
                    "L" => {
                        i += 3;
                        Some([num(2), num(3), num(4)])
                    }
                    _ => None,
                };
                let rotation = (f[i] == "R").then(|| {
                    (
                        f[i + 1].parse().expect("yaw"),
                        f[i + 2].parse().expect("pitch"),
                    )
                });
                // `moveOrInterpolateTo`: a packet without a position or
                // rotation keeps the one the entity is headed for.
                let current = handler.heading(entity);
                let position = position.unwrap_or(current.position);
                let (yaw, pitch) = rotation.unwrap_or((current.yaw, current.pitch));
                match handler.interpolate_to(entity, position, yaw, pitch) {
                    Start::Snap(at) => {
                        entity.position = at.position;
                        set_rot(&mut entity, at.yaw, at.pitch);
                        "s"
                    }
                    Start::Unchanged => "k",
                    Start::Interpolating => "i",
                }
            }
            "tick" => match handler.interpolate() {
                Some(at) => {
                    entity.position = at.position;
                    set_rot(&mut entity, at.yaw, at.pitch);
                    "t"
                }
                None => "n",
            },
            other => panic!("unknown event {other}"),
        };
        let p = entity.position;
        writeln!(
            out,
            "{what} {:x} {:x} {:x} {:x} {:x}",
            p[0].to_bits(),
            p[1].to_bits(),
            p[2].to_bits(),
            entity.yaw.to_bits(),
            entity.pitch.to_bits()
        )
        .expect("write");
    }
    std::fs::write(&args[2], out).expect("output");
}
