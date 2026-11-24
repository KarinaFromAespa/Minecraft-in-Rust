//! The launcher's own version: this tree's Minecraft version, provided on first run from files
//! embedded at build time (build.rs assembles them from the tree — the research lock, the
//! exporters' Java sources, the preparer's sources, and a precompiled exporters bundle when one
//! was laid out). No upstream release is involved: the local version plays this tree's game,
//! and an imported build takes its game files from it (`build_import`).
//!
//! The seed is written into `versions/<tag>/` (`.installed` beside it, so it lists as installed)
//! when the folder is missing or its `.seed` marker no longer matches the embedded hash — a
//! newer launcher refreshes the seed files, never the player's prepared resources (`.local/`)
//! or anything else in the folder. The folder holds no game program of its own: Play on it
//! asks for one to import, and the import's copy is what runs.

use crate::backend;
use std::fs;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/seed.rs"));

/// The `.seed` marker: the embedded hash, written last, so a folder with it holds the whole seed.
const MARKER: &str = ".seed";
/// The seed-owned folders, cleared before a (re)materialization so no orphaned file survives.
const OWNED: &[&str] = &[
    "mods/horizon/src/test/java/dev/horizonlab/parity",
    "crates/horizon-prepare",
    "exporters",
];

/// Whether `tag` is this launcher's own version.
pub fn is_local_version(tag: &str) -> bool {
    tag == LOCAL_TAG
}

/// Provides the local version in the launcher's versions folder: true when it wrote it, false
/// when it was already there and current. A failure leaves the folder alone (no partial seed
/// is ever marked complete: the marker goes in last).
pub fn ensure_local_version() -> Result<bool, String> {
    ensure_local_version_in(&backend::versions_dir())
}

fn ensure_local_version_in(versions: &Path) -> Result<bool, String> {
    let dir = versions.join(LOCAL_TAG);
    let fresh = fs::read_to_string(dir.join(MARKER)).is_ok_and(|marker| marker.trim() == SEED_HASH)
        && dir.join(".installed").is_file();
    if fresh {
        return Ok(false);
    }
    for owned in OWNED {
        let owned = owned.split('/').fold(dir.clone(), |d, part| d.join(part));
        if owned.exists() {
            fs::remove_dir_all(&owned).map_err(|e| format!("clearing {}: {e}", owned.display()))?;
        }
    }
    for (name, bytes) in SEED_FILES {
        let dest = name.split('/').fold(dir.clone(), |d, part| d.join(part));
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(&dest, bytes).map_err(|e| format!("{}: {e}", dest.display()))?;
    }
    fs::write(dir.join(".installed"), "").map_err(|e| e.to_string())?;
    fs::write(dir.join(MARKER), SEED_HASH).map_err(|e| e.to_string())?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("horizon-seed-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    /// The tag the seed provides is the version this tree pins.
    #[test]
    fn the_local_version_is_this_trees_version() {
        assert_eq!(LOCAL_TAG, horizon_platform::version::VANILLA_VERSION);
    }

    /// Materializing writes the whole seed, marks it complete, and is a no-op the second time.
    #[test]
    fn ensure_materializes_once_and_only_once() {
        let versions = dir("once");
        assert!(ensure_local_version_in(&versions).unwrap());
        let local = versions.join(LOCAL_TAG);
        assert!(local.join(".installed").is_file());
        assert_eq!(fs::read_to_string(local.join(MARKER)).unwrap(), SEED_HASH);
        for (name, bytes) in SEED_FILES {
            let dest = name.split('/').fold(local.clone(), |d, part| d.join(part));
            assert_eq!(&fs::read(dest).unwrap(), bytes, "{name}");
        }
        assert!(!ensure_local_version_in(&versions).unwrap());
    }

    /// A stale or damaged seed is repaired, and a prepared folder beside it is left alone.
    #[test]
    fn a_stale_seed_is_repaired_without_touching_prepared_resources() {
        let versions = dir("stale");
        let local = versions.join(LOCAL_TAG);
        assert!(ensure_local_version_in(&versions).unwrap());
        fs::create_dir_all(local.join(".local/horizon-client").join(LOCAL_TAG)).unwrap();
        fs::write(
            local
                .join(".local/horizon-client")
                .join(LOCAL_TAG)
                .join("kept.txt"),
            b"prepared",
        )
        .unwrap();
        // An orphaned exporter source and a damaged lock: both gone after the repair.
        let orphan = local.join(OWNED[0]).join("VanillaGoneOracle.java");
        fs::write(&orphan, b"class VanillaGoneOracle {}").unwrap();
        fs::write(local.join("research-lock.json"), b"damaged").unwrap();
        fs::remove_file(local.join(MARKER)).unwrap();
        assert!(ensure_local_version_in(&versions).unwrap());
        assert!(!orphan.exists());
        assert_eq!(
            fs::read(local.join("research-lock.json")).unwrap(),
            SEED_FILES
                .iter()
                .find(|(n, _)| *n == "research-lock.json")
                .unwrap()
                .1
        );
        assert_eq!(
            fs::read_to_string(
                local
                    .join(".local/horizon-client")
                    .join(LOCAL_TAG)
                    .join("kept.txt")
            )
            .unwrap(),
            "prepared"
        );
    }

    /// The materialized preparer sources are the launcher's own: prepare runs in-process.
    #[test]
    fn the_materialized_preparer_is_the_launchers_own() {
        let versions = dir("preparer");
        assert!(ensure_local_version_in(&versions).unwrap());
        let local = versions.join(LOCAL_TAG);
        assert_eq!(
            horizon_platform::prepare::sources_sha256(&local.join("crates/horizon-prepare"))
                .as_deref(),
            Some(horizon_platform::prepare::SOURCES_SHA256)
        );
    }

    /// The materialized lock pins the local version, and the exporter sources are all there.
    #[test]
    fn the_materialized_seed_pins_the_local_version() {
        let versions = dir("pins");
        assert!(ensure_local_version_in(&versions).unwrap());
        let local = versions.join(LOCAL_TAG);
        let lock: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(local.join("research-lock.json")).unwrap())
                .unwrap();
        assert_eq!(lock["minecraft"]["version"].as_str(), Some(LOCAL_TAG));
        assert!(lock["minecraft"]["client_sha1"].as_str().is_some());
        assert!(lock["minecraft"]["server_sha1"].as_str().is_some());
        let parity = local.join(OWNED[0]);
        let sources = fs::read_dir(&parity).unwrap().count();
        assert!(sources > 100, "parity sources: {sources}");
        assert!(parity.join("VanillaGameplayFactsExporter.java").is_file());
    }
}
