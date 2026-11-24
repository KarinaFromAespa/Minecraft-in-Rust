//! Embeds Horizon's icon (`packaging/icons/horizon.ico`) in the Windows executables, as icon
//! resource 1: Explorer and the task bar show it, and the window takes it as its icon.
//!
//! And assembles the version seed (`seed.rs`, read by `src/seed.rs`): the files of this tree the
//! launcher provides as its own local version on first run — the research lock, the exporters'
//! Java sources, the preparer's sources, jsr305 (the exporter compile's classpath needs
//! javax.annotation.Nullable), and a precompiled exporters bundle when `seed-exporters/` holds
//! one. The seed is this tree's own files only: no game jars, no Java at build time, so the
//! launcher build stays pure Rust.

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // Resolve imports before main from System32, never a nearby injector DLL (#13108).
        println!("cargo:rustc-link-arg-bins=/DEPENDENTLOADFLAG:0x800");
    }
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let ico = format!("{dir}/../packaging/icons/horizon.ico");
    println!("cargo:rerun-if-changed={ico}");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon(&ico);
        if let Err(e) = resource.compile() {
            panic!("embedding {ico}: {e}");
        }
    }
    seed(Path::new(&dir));
}

/// The version seed: every file the local version folder is made of, as `(relative path, bytes)`,
/// copied to `OUT_DIR/seed/` and listed in the generated `OUT_DIR/seed.rs` (with the tag from the
/// lock and an FNV-1a staleness hash over name, length and bytes — a cache key, not a digest).
fn seed(manifest_dir: &Path) {
    let tree = manifest_dir.join("..");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let seed_dir = out.join("seed");
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    let mut add = |rel: &str, path: &Path| {
        println!("cargo:rerun-if-changed={}", path.display());
        let bytes =
            fs::read(path).unwrap_or_else(|e| panic!("reading the seed's {}: {e}", path.display()));
        files.push((rel.to_string(), bytes));
    };
    // The pins: the local version's Minecraft version and jar checksums.
    add("research-lock.json", &tree.join("research-lock.json"));
    // The exporters' Java sources: `horizon-prepare` compiles them (or checks a bundle against
    // them) from the version folder, and refuses a folder without them.
    let parity = "mods/horizon/src/test/java/dev/horizonlab/parity";
    println!("cargo:rerun-if-changed={}", tree.join(parity).display());
    for path in walk(&tree.join(parity)) {
        if path.extension().is_some_and(|e| e == "java") {
            let rel = path
                .strip_prefix(&tree)
                .unwrap()
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            add(&rel, &path);
        }
    }
    // The preparer's sources: the launcher prepares in-process when the version folder's copy
    // hashes to its own `SOURCES_SHA256` (the same files `horizon-prepare`'s build.rs hashes).
    let preparer = "crates/horizon-prepare";
    println!("cargo:rerun-if-changed={}", tree.join(preparer).display());
    for name in ["Cargo.toml", "build.rs"] {
        add(
            &format!("{preparer}/{name}"),
            &tree.join(preparer).join(name),
        );
    }
    let mut sources: Vec<PathBuf> = fs::read_dir(tree.join(preparer).join("src"))
        .unwrap_or_else(|e| panic!("reading the seed's {preparer}/src: {e}"))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    sources.sort();
    for path in sources {
        let rel = path
            .strip_prefix(&tree)
            .unwrap()
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        add(&rel, &path);
    }
    // jsr305: the exporter compile's classpath needs javax.annotation.Nullable
    // (horizon-prepare's with_jsr305 looks for it under tools/), and the seed's
    // version folder has no tools/ otherwise — without it javac crashes
    // completing symbols from the game jars. A public annotation library, not
    // Mojang-derived data.
    add("tools/jsr305-3.0.2.jar", &tree.join("tools/jsr305-3.0.2.jar"));
    // A precompiled exporters bundle, when one was laid out beside the launcher: with it a
    // player needs only a Java runtime to prepare, without it a JDK. The bundle is built, not
    // committed (compiled classes stay out of Git), so its absence is normal. The rerun line
    // stands even when the folder is missing, so laying one out rebuilds the launcher.
    let bundle = manifest_dir.join("seed-exporters");
    println!("cargo:rerun-if-changed={}", bundle.display());
    if bundle.is_dir() {
        for path in walk(&bundle) {
            let rel = format!(
                "exporters/{}",
                path.strip_prefix(&bundle)
                    .unwrap()
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/")
            );
            add(&rel, &path);
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let lock = files
        .iter()
        .find(|(name, _)| name == "research-lock.json")
        .map(|(_, bytes)| String::from_utf8_lossy(bytes).into_owned())
        .unwrap();
    let tag = local_tag(&lock);
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut mix = |bytes: &[u8]| {
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    };
    for (name, bytes) in &files {
        mix(name.as_bytes());
        mix(b"\0");
        mix(&(bytes.len() as u64).to_le_bytes());
        mix(b"\0");
        mix(bytes);
    }
    let _ = fs::remove_dir_all(&seed_dir);
    for (name, bytes) in &files {
        let dest = name
            .split('/')
            .fold(seed_dir.clone(), |d, part| d.join(part));
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::write(&dest, bytes).unwrap();
    }
    let mut generated = String::from(
        "// Generated by build.rs: the version seed (see `seed.rs` for what it is for).\n",
    );
    generated.push_str(&format!("pub const LOCAL_TAG: &str = \"{tag}\";\n"));
    generated.push_str(&format!("pub const SEED_HASH: &str = \"{hash:016x}\";\n"));
    generated.push_str("pub const SEED_FILES: &[(&str, &[u8])] = &[\n");
    for (name, _) in &files {
        generated.push_str(&format!(
            "    (\"{name}\", include_bytes!(\"seed/{name}\")),\n"
        ));
    }
    generated.push_str("];\n");
    fs::write(out.join("seed.rs"), generated).unwrap();
}

/// The `minecraft.version` of the research lock: the local version's tag.
fn local_tag(lock: &str) -> String {
    let after = |key: &str, from: usize| -> usize {
        lock[from..]
            .find(key)
            .unwrap_or_else(|| panic!("research-lock.json has no {key}"))
            + from
            + key.len()
    };
    let mc = after("\"minecraft\"", 0);
    let version = after("\"version\"", mc);
    let quoted = lock[version..]
        .find('"')
        .unwrap_or_else(|| panic!("research-lock.json has no quoted version"));
    let start = version + quoted + 1;
    let end = lock[start..]
        .find('"')
        .unwrap_or_else(|| panic!("research-lock.json has an unterminated version"));
    lock[start..start + end].to_string()
}

/// Every file under `dir`, recursively, sorted.
fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("reading the seed's {}: {e}", dir.display()));
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}
