//! Test transport for comparing 26.2 noise with the downloaded vanilla artifacts.
use horizon_core::{
    noise::synth::{
        BlendedNoise, ImprovedNoise, NoiseParameters, NoiseUtils, NormalNoise, PerlinNoise,
        PerlinSimplexNoise, SimplexNoise, wrap,
    },
    random::{LegacyRandom, RandomSource, XoroshiroRandom},
};
use std::io::{self, BufRead, BufWriter, Write};

fn read<T: std::str::FromStr>(
    args: &mut std::str::SplitWhitespace<'_>,
) -> Result<T, Box<dyn std::error::Error>> {
    args.next()
        .ok_or("missing argument")?
        .parse()
        .map_err(|_| "invalid argument".into())
}
fn double(args: &mut std::str::SplitWhitespace<'_>) -> Result<f64, Box<dyn std::error::Error>> {
    Ok(f64::from_bits(u64::from_str_radix(
        args.next().ok_or("missing double")?,
        16,
    )?))
}
fn dbits(v: f64) -> u64 {
    if v.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        v.to_bits()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut out = BufWriter::new(io::stdout().lock());
    let mut improved: Vec<ImprovedNoise> = Vec::new();
    let mut perlin: Vec<PerlinNoise> = Vec::new();
    let mut simplex: Vec<SimplexNoise> = Vec::new();
    let mut psimplex: Vec<PerlinSimplexNoise> = Vec::new();
    let mut normal: Vec<NormalNoise> = Vec::new();
    let mut blended: Vec<BlendedNoise> = Vec::new();
    for line in stdin.lock().lines() {
        let line = line?;
        let mut args = line.split_whitespace();
        match args.next().ok_or("missing operation")? {
            "wrap" => writeln!(out, "wrap {}", dbits(wrap(double(&mut args)?)))?,
            "bias" => {
                let noise = double(&mut args)?;
                let factor = double(&mut args)?;
                writeln!(
                    out,
                    "bias {}",
                    dbits(NoiseUtils::bias_towards_extreme(noise, factor))
                )?;
            }
            "create-improved" => {
                let kind = args.next().ok_or("missing RNG kind")?;
                let seed: i64 = read(&mut args)?;
                let noise = match kind {
                    "legacy" => {
                        let mut random = LegacyRandom::new(seed);
                        let noise = ImprovedNoise::new(&mut random);
                        writeln!(
                            out,
                            "improved {} {} {} {}",
                            dbits(noise.xo),
                            dbits(noise.yo),
                            dbits(noise.zo),
                            random.next_long()
                        )?;
                        noise
                    }
                    "xoroshiro" => {
                        let mut random = XoroshiroRandom::new(seed);
                        let noise = ImprovedNoise::new(&mut random);
                        writeln!(
                            out,
                            "improved {} {} {} {}",
                            dbits(noise.xo),
                            dbits(noise.yo),
                            dbits(noise.zo),
                            random.next_long()
                        )?;
                        noise
                    }
                    _ => return Err("unknown stream".into()),
                };
                write!(out, "permutations")?;
                for v in noise.permutations() {
                    write!(out, " {v}")?;
                }
                writeln!(out)?;
                writeln!(out, "config {}", noise.parity_config_string())?;
                improved.push(noise);
            }
            operation @ ("query-improved" | "local-improved") => {
                let id: usize = read(&mut args)?;
                let mut point = [double(&mut args)?, double(&mut args)?, double(&mut args)?];
                let noise = &improved[id];
                if operation == "local-improved" {
                    point[0] -= noise.xo;
                    point[1] -= noise.yo;
                    point[2] -= noise.zo;
                }
                let y_scale = double(&mut args)?;
                let y_fudge = double(&mut args)?;
                let mut derivative =
                    [double(&mut args)?, double(&mut args)?, double(&mut args)?];
                let [x, y, z] = point;
                let value = noise.noise(x, y, z);
                let smeared = noise.noise_smeared(x, y, z, y_scale, y_fudge);
                let with_derivative = noise.noise_with_derivative(x, y, z, &mut derivative);
                writeln!(
                    out,
                    "improved-query {} {} {} {} {} {}",
                    dbits(value),
                    dbits(smeared),
                    dbits(with_derivative),
                    dbits(derivative[0]),
                    dbits(derivative[1]),
                    dbits(derivative[2])
                )?;
            }
            "create-perlin" => {
                let kind = args.next().ok_or("missing RNG kind")?;
                let seed: i64 = read(&mut args)?;
                let init: i32 = read(&mut args)?;
                let first_octave: i32 = read(&mut args)?;
                let count: usize = read(&mut args)?;
                let amplitudes = (0..count)
                    .map(|_| double(&mut args))
                    .collect::<Result<Vec<_>, _>>()?;
                macro_rules! build {
                    ($random:expr) => {{
                        let mut random = $random;
                        let built = if init == 0 {
                            PerlinNoise::create(&mut random, first_octave, &amplitudes)
                        } else {
                            PerlinNoise::create_legacy_for_legacy_nether_biome(
                                &mut random,
                                first_octave,
                                &amplitudes,
                            )
                        };
                        match built {
                            Err(_) => {
                                writeln!(out, "perlin error {}", random.next_long())?;
                            }
                            Ok(noise) => {
                                writeln!(
                                    out,
                                    "perlin {} {}",
                                    first_octave,
                                    dbits(noise.max_value())
                                )?;
                                writeln!(out, "perlin-config {}", noise.parity_config_string())?;
                                writeln!(out, "perlin-parent {}", random.next_long())?;
                                perlin.push(noise);
                            }
                        }
                    }};
                }
                match kind {
                    "legacy" => build!(LegacyRandom::new(seed)),
                    "xoroshiro" => build!(XoroshiroRandom::new(seed)),
                    _ => return Err("unknown stream".into()),
                }
            }
            "create-perlin-octaves" => {
                let kind = args.next().ok_or("missing RNG kind")?;
                let seed: i64 = read(&mut args)?;
                let init: i32 = read(&mut args)?;
                let count: usize = read(&mut args)?;
                let octaves = (0..count)
                    .map(|_| read::<i32>(&mut args))
                    .collect::<Result<Vec<_>, _>>()?;
                macro_rules! build {
                    ($random:expr) => {{
                        let mut random = $random;
                        let built = if init == 0 {
                            PerlinNoise::create_from_octaves(&mut random, &octaves)
                        } else {
                            PerlinNoise::create_legacy_for_blended_noise(&mut random, &octaves)
                        };
                        match built {
                            Err(_) => {
                                writeln!(out, "perlin error {}", random.next_long())?;
                            }
                            Ok(noise) => {
                                writeln!(
                                    out,
                                    "perlin {} {}",
                                    noise.first_octave(),
                                    dbits(noise.max_value())
                                )?;
                                writeln!(out, "perlin-config {}", noise.parity_config_string())?;
                                writeln!(out, "perlin-parent {}", random.next_long())?;
                                perlin.push(noise);
                            }
                        }
                    }};
                }
                match kind {
                    "legacy" => build!(LegacyRandom::new(seed)),
                    "xoroshiro" => build!(XoroshiroRandom::new(seed)),
                    _ => return Err("unknown stream".into()),
                }
            }
            "query-perlin" => {
                let id: usize = read(&mut args)?;
                let x = double(&mut args)?;
                let y = double(&mut args)?;
                let z = double(&mut args)?;
                let y_scale = double(&mut args)?;
                let y_fudge = double(&mut args)?;
                let noise = &perlin[id];
                writeln!(
                    out,
                    "perlin-query {} {}",
                    dbits(noise.get_value(x, y, z)),
                    dbits(noise.get_value_smeared(x, y, z, y_scale, y_fudge))
                )?;
            }
            "perlin-maxbroken" => {
                let id: usize = read(&mut args)?;
                let y_scale = double(&mut args)?;
                writeln!(
                    out,
                    "perlin-maxbroken {}",
                    dbits(perlin[id].max_broken_value(y_scale))
                )?;
            }
            "create-simplex" => {
                let kind = args.next().ok_or("missing RNG kind")?;
                let seed: i64 = read(&mut args)?;
                let noise = match kind {
                    "legacy" => {
                        let mut random = LegacyRandom::new(seed);
                        let noise = SimplexNoise::new(&mut random);
                        writeln!(
                            out,
                            "simplex {} {} {} {}",
                            dbits(noise.xo),
                            dbits(noise.yo),
                            dbits(noise.zo),
                            random.next_long()
                        )?;
                        noise
                    }
                    "xoroshiro" => {
                        let mut random = XoroshiroRandom::new(seed);
                        let noise = SimplexNoise::new(&mut random);
                        writeln!(
                            out,
                            "simplex {} {} {} {}",
                            dbits(noise.xo),
                            dbits(noise.yo),
                            dbits(noise.zo),
                            random.next_long()
                        )?;
                        noise
                    }
                    _ => return Err("unknown stream".into()),
                };
                write!(out, "permutations")?;
                for v in noise.permutations() {
                    write!(out, " {v}")?;
                }
                writeln!(out)?;
                writeln!(out, "config {}", noise.parity_config_string())?;
                simplex.push(noise);
            }
            "query-simplex" => {
                let id: usize = read(&mut args)?;
                let x = double(&mut args)?;
                let y = double(&mut args)?;
                let z = double(&mut args)?;
                let noise = &simplex[id];
                writeln!(
                    out,
                    "simplex-query {} {}",
                    dbits(noise.get_value(x, y)),
                    dbits(noise.get_value_3d(x, y, z))
                )?;
            }
            "create-perlin-simplex" => {
                let kind = args.next().ok_or("missing RNG kind")?;
                let seed: i64 = read(&mut args)?;
                let count: usize = read(&mut args)?;
                let octaves = (0..count)
                    .map(|_| read::<i32>(&mut args))
                    .collect::<Result<Vec<_>, _>>()?;
                macro_rules! build {
                    ($random:expr) => {{
                        let mut random = $random;
                        match PerlinSimplexNoise::new(&mut random, &octaves) {
                            Err(_) => {
                                writeln!(out, "perlin-simplex error {}", random.next_long())?;
                            }
                            Ok(noise) => {
                                writeln!(
                                    out,
                                    "perlin-simplex {} {}",
                                    dbits(noise.highest_freq_input_factor()),
                                    dbits(noise.highest_freq_value_factor())
                                )?;
                                for (i, level) in noise.levels().iter().enumerate() {
                                    match level {
                                        None => writeln!(out, "psimplex-level {i} absent")?,
                                        Some(level) => writeln!(
                                            out,
                                            "psimplex-level {i} {} {} {}",
                                            dbits(level.xo),
                                            dbits(level.yo),
                                            dbits(level.zo)
                                        )?,
                                    }
                                }
                                writeln!(out, "perlin-simplex-parent {}", random.next_long())?;
                                psimplex.push(noise);
                            }
                        }
                    }};
                }
                match kind {
                    "legacy" => build!(LegacyRandom::new(seed)),
                    "xoroshiro" => build!(XoroshiroRandom::new(seed)),
                    _ => return Err("unknown stream".into()),
                }
            }
            "query-perlin-simplex" => {
                let id: usize = read(&mut args)?;
                let x = double(&mut args)?;
                let y = double(&mut args)?;
                let use_start: i32 = read(&mut args)?;
                writeln!(
                    out,
                    "psimplex-query {}",
                    dbits(psimplex[id].get_value(x, y, use_start != 0))
                )?;
            }
            "normal" => {
                let kind = args.next().ok_or("missing RNG kind")?;
                let seed: i64 = read(&mut args)?;
                let legacy: i32 = read(&mut args)?;
                let first_octave: i32 = read(&mut args)?;
                let count: usize = read(&mut args)?;
                let amplitudes = (0..count)
                    .map(|_| double(&mut args))
                    .collect::<Result<Vec<_>, _>>()?;
                let point_count: usize = read(&mut args)?;
                let points = (0..point_count)
                    .map(|_| {
                        Ok([
                            double(&mut args)?,
                            double(&mut args)?,
                            double(&mut args)?,
                        ])
                    })
                    .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
                macro_rules! build {
                    ($random:expr) => {{
                        let mut random = $random;
                        let built = if legacy != 0 {
                            NormalNoise::create_legacy_nether_biome(
                                &mut random,
                                NoiseParameters {
                                    first_octave,
                                    amplitudes: amplitudes.clone(),
                                },
                            )
                        } else {
                            NormalNoise::create(&mut random, first_octave, &amplitudes)
                        };
                        match built {
                            Err(_) => {
                                writeln!(out, "normal error {}", random.next_long())?;
                            }
                            Ok(noise) => {
                                writeln!(
                                    out,
                                    "normal {} {}",
                                    dbits(noise.value_factor()),
                                    dbits(noise.max_value())
                                )?;
                                writeln!(out, "normal-config {}", noise.parity_config_string())?;
                                for &[x, y, z] in &points {
                                    writeln!(out, "normal-query {}", dbits(noise.get_value(x, y, z)))?;
                                }
                                writeln!(out, "normal-parent {}", random.next_long())?;
                                normal.push(noise);
                            }
                        }
                    }};
                }
                match kind {
                    "legacy" => build!(LegacyRandom::new(seed)),
                    "xoroshiro" => build!(XoroshiroRandom::new(seed)),
                    _ => return Err("unknown stream".into()),
                }
            }
            operation @ ("blended" | "blended-unseeded") => {
                let seeded = operation == "blended";
                let (kind, seed) = if seeded {
                    (
                        args.next().ok_or("missing RNG kind")?.to_owned(),
                        read::<i64>(&mut args)?,
                    )
                } else {
                    (String::new(), 0)
                };
                let scales = [
                    double(&mut args)?,
                    double(&mut args)?,
                    double(&mut args)?,
                    double(&mut args)?,
                    double(&mut args)?,
                ];
                let point_count: usize = read(&mut args)?;
                let points = (0..point_count)
                    .map(|_| {
                        Ok([
                            read::<i32>(&mut args)?,
                            read::<i32>(&mut args)?,
                            read::<i32>(&mut args)?,
                        ])
                    })
                    .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
                let [xz_scale, y_scale, xz_factor, y_factor, smear] = scales;
                if seeded {
                    macro_rules! build {
                        ($random:expr) => {{
                            let mut random = $random;
                            match BlendedNoise::new(
                                &mut random, xz_scale, y_scale, xz_factor, y_factor, smear,
                            ) {
                                Err(_) => {
                                    writeln!(out, "blended error {}", random.next_long())?;
                                }
                                Ok(noise) => {
                                    writeln!(
                                        out,
                                        "blended {} {}",
                                        dbits(noise.max_value()),
                                        dbits(noise.min_value())
                                    )?;
                                    writeln!(out, "blended-config {}", noise.parity_config_string())?;
                                    for &[x, y, z] in &points {
                                        writeln!(out, "blended-query {}", dbits(noise.compute(x, y, z)))?;
                                    }
                                    writeln!(out, "blended-parent {}", random.next_long())?;
                                    blended.push(noise);
                                }
                            }
                        }};
                    }
                    match kind.as_str() {
                        "legacy" => build!(LegacyRandom::new(seed)),
                        "xoroshiro" => build!(XoroshiroRandom::new(seed)),
                        _ => return Err("unknown stream".into()),
                    }
                } else {
                    match BlendedNoise::create_unseeded(
                        xz_scale, y_scale, xz_factor, y_factor, smear,
                    ) {
                        Err(_) => writeln!(out, "blended error")?,
                        Ok(noise) => {
                            writeln!(
                                out,
                                "blended {} {}",
                                dbits(noise.max_value()),
                                dbits(noise.min_value())
                            )?;
                            writeln!(out, "blended-config {}", noise.parity_config_string())?;
                            for &[x, y, z] in &points {
                                writeln!(out, "blended-query {}", dbits(noise.compute(x, y, z)))?;
                            }
                            blended.push(noise);
                        }
                    }
                }
            }
            _ => return Err("unknown operation".into()),
        }
        if args.next().is_some() {
            return Err("extra argument".into());
        }
    }
    out.flush()?;
    Ok(())
}
