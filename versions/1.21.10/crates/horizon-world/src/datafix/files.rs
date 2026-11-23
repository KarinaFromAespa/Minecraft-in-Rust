//! Opening an older world as 1.21.10 does: level.dat brought to the current
//! version, and the upgrades vanilla runs as it reads each file ([`upgrade_world`]
//! does them all at once, as a converter). 1.21.10 has no file-fixer system
//! (`net.minecraft.util.filefix` arrived with DataVersion 4772 in 26.1), so
//! worlds keep the pre-26.x layout: the overworld at the root, DIM-1 and DIM1
//! for the Nether and the End, `playerdata`, `advancements` and `stats`.
use super::dynamic::{as_int, as_str, remove, set, string};
use super::json::{self, Json};
use super::progress::{UpgradeProgress, UpgradeType};
use super::schema::Ty;
use super::world_upgrader::worker_threads;
use super::{CURRENT, update};
use crate::compression::Compression;
use crate::nbt::{self, Compound, JavaString, Limits, NamedTag, Tag};
use crate::region::{ChunkRecord, RegionFile, RegionWriter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// The name `SharedConstants.getCurrentVersion().name()` writes in level.dat.
pub const VERSION_NAME: &str = horizon_core::version::VANILLA_VERSION;

/// The player pressed Cancel during the world upgrade.
/// (`WorldUpgrader.cancel` in 1.21.10 just stops the worker by flag;
/// the Rust unwinds with an Interrupted error instead. 26.1's filefix
/// system raised `CanceledFileFixException` for this — 1.21.10 has no
/// filefix system.)
fn canceled() -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, "the world upgrade was canceled")
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

/// `NbtIo.readCompressed`: the root compound of a gzip NBT file, if it exists.
pub fn read_dat(path: &Path) -> io::Result<Option<Tag>> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let limits = Limits::default();
    let raw = Compression::Gzip.decode(&bytes, limits.max_bytes)?;
    let (root, _) = nbt::read_named(&raw, limits).map_err(|e| invalid(e.to_string()))?;
    Ok(Some(root.tag))
}

/// `SavedDataStorage.readTagFromDisk`: a file without the gzip header is plain NBT.
pub fn read_saved_dat(path: &Path) -> io::Result<Option<Tag>> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let limits = Limits::default();
    let raw = if bytes.starts_with(&[0x1f, 0x8b]) {
        Compression::Gzip.decode(&bytes, limits.max_bytes)?
    } else {
        bytes
    };
    let (root, _) = nbt::read_named(&raw, limits).map_err(|e| invalid(e.to_string()))?;
    Ok(Some(root.tag))
}

/// `NbtIo.writeCompressed`, creating the parent folders.
pub fn write_dat(path: &Path, tag: &Tag) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let root = NamedTag {
        name: JavaString::default(),
        tag: tag.clone(),
    };
    let raw = nbt::write_named(&root, Limits::default()).map_err(|e| invalid(e.to_string()))?;
    fs::write(path, Compression::Gzip.encode(&raw)?)
}

/// `NbtUtils.getDataVersion` — 1.21.10 has only the two-argument forms
/// (`(CompoundTag, int)` and `(Dynamic, int)`); the one-argument forms
/// are 1.21.11's (`CompoundTag`) and 26.x's (`Dynamic`).
pub fn data_version(tag: &Tag, default: i32) -> i32 {
    match tag.get("DataVersion") {
        Some(Tag::Int(v)) => *v,
        _ => default,
    }
}

/// Whether the world at `level_version` needs file fixes when it is
/// opened. 1.21.10 has no `FileFixerUpper`: there is no
/// `net.minecraft.util.filefix` package, no `FILE_FIXING_REQUIRED` backup
/// status and no layout migration, so a world never needs file fixes — it
/// only needs its data upgraded.
pub fn requires_file_fixing(_level_version: i32) -> bool {
    false
}

// ------------------------------------------------------------------ file operations


pub(super) fn subdirectories(path: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(path)
        .map(|r| {
            r.filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}


/// The version stamp level.dat carries once a world is brought to the
/// current version: DataVersion, LastPlayed and the Version compound.
fn add_versions(data: &mut Tag) {
    set(data, "DataVersion", Tag::Int(CURRENT));
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64);
    set(data, "LastPlayed", Tag::Long(now));
    set(
        data,
        "Version",
        Tag::compound([
            ("Name", string(VERSION_NAME)),
            ("Id", Tag::Int(CURRENT)),
            ("Snapshot", Tag::Byte(0)),
            ("Series", string("main")),
        ]),
    );
}

/// Bring a world saved at an older version to the current one: its
/// level.dat at the current version. Returns the DataVersion the world had.
pub fn fix_world_files(base: &Path) -> io::Result<i32> {
    let (loaded, pending) = fix_world_files_deferred(base, &UpgradeProgress::new())?;
    if let Some(data) = pending {
        write_dat(&base.join("level.dat"), &Tag::compound([("Data", data)]))?;
    }
    Ok(loaded)
}

/// [`fix_world_files`] without the final level.dat write: returns the loaded
/// DataVersion and the level data at the current version still to be written
/// (`None` when the world is already current). A caller that converts the
/// world's data afterwards writes it last, so a failed upgrade keeps the old
/// DataVersion and is retried on the next open.
fn fix_world_files_deferred(
    base: &Path,
    _progress: &UpgradeProgress,
) -> io::Result<(i32, Option<Tag>)> {
    let root = read_dat(&base.join("level.dat"))?.ok_or_else(|| invalid("missing level.dat"))?;
    let mut data = root
        .get("Data")
        .cloned()
        .unwrap_or_else(|| Tag::Compound(Compound::new()));
    let loaded = data_version(&data, -1);
    if loaded >= CURRENT {
        return Ok((loaded, None));
    }
    let from = loaded;
    update(Ty::Level, &mut data, from, CURRENT);
    add_versions(&mut data);
    Ok((loaded, Some(data)))
}

/// The world's generator settings as 1.21.10 reads them: `WorldGenSettings`
/// in level.dat's `Data`, brought to the current version. (26.x moved the
/// settings out of level.dat into `world_gen_settings.dat` with the 4772
/// file fix; 1.21.10 never did.)
pub fn current_world_gen_settings(base: &Path) -> io::Result<Option<Tag>> {
    let Some(root) = read_dat(&base.join("level.dat"))? else {
        return Ok(None);
    };
    let Some(data) = root.get("Data") else {
        return Ok(None);
    };
    let Some(mut settings) = data.get("WorldGenSettings").cloned() else {
        return Ok(None);
    };
    let from = data_version(data, -1);
    update(Ty::WorldGenSettings, &mut settings, from, CURRENT);
    Ok(Some(settings))
}

// ------------------------------------------------------------------ the read-time upgrades

/// Chunk generator types per dimension from level.dat's `WorldGenSettings`,
/// the `generator` of the context the chunk fixes read.
pub fn generators(base: &Path) -> io::Result<Vec<(String, String)>> {
    let Some(root) = read_dat(&base.join("level.dat"))? else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    if let Some(Tag::Compound(dims)) = root
        .get("Data")
        .and_then(|d| d.get("WorldGenSettings"))
        .and_then(|s| s.get("dimensions"))
    {
        for (id, dim) in dims {
            if let Some(ty) = dim.get("generator").and_then(|g| as_str(g.get("type"))) {
                out.push((id.to_string_lossy(), ty));
            }
        }
    }
    Ok(out)
}

fn files_with_suffix(folder: &Path, suffix: &str) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(folder)
        .map(|r| {
            r.filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.to_string_lossy().ends_with(suffix))
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

/// Rewrite every chunk of the region files of a folder with `fix`, the files
/// spread over [`worker_threads`] workers. Each file is read, fixed and
/// written whole by one worker, so its bytes do not depend on the thread count.
fn upgrade_regions(
    folder: &Path,
    progress: &Progress,
    fix: impl Fn(&mut Tag) + Sync,
) -> io::Result<()> {
    upgrade_regions_with_threads(folder, progress, fix, worker_threads())
}

/// [`upgrade_regions`] on `threads` workers. A cancel or an error stops every
/// worker before its next file; the error returned is the first file's in
/// the folder's order.
fn upgrade_regions_with_threads(
    folder: &Path,
    progress: &Progress,
    fix: impl Fn(&mut Tag) + Sync,
    threads: usize,
) -> io::Result<()> {
    let files = files_with_suffix(folder, ".mca");
    if files.is_empty() {
        return Ok(());
    }
    let staging = folder.join(".datafix");
    fs::create_dir_all(&staging)?;
    let next = AtomicUsize::new(0);
    let stop = AtomicBool::new(false);
    let failed: Mutex<Option<(usize, io::Error)>> = Mutex::new(None);
    let work = || {
        while !stop.load(Ordering::Relaxed) {
            let index = next.fetch_add(1, Ordering::Relaxed);
            let Some(path) = files.get(index) else {
                break;
            };
            match progress
                .check()
                .and_then(|()| upgrade_region_file(path, &staging, &fix))
            {
                Ok(()) => progress.progress.increment_finished_operations(),
                Err(e) => {
                    stop.store(true, Ordering::Relaxed);
                    let mut failed = failed.lock().unwrap_or_else(|e| e.into_inner());
                    if failed.as_ref().is_none_or(|(first, _)| index < *first) {
                        *failed = Some((index, e));
                    }
                }
            }
        }
    };
    std::thread::scope(|scope| {
        // This thread is worker 0; a helper that fails to start leaves its
        // share to the others.
        for i in 1..threads.clamp(1, files.len()) {
            let _ = std::thread::Builder::new()
                .name(format!("World Upgrader #{i}"))
                .spawn_scoped(scope, work);
        }
        work();
    });
    if let Some((_, e)) = failed.into_inner().unwrap_or_else(|e| e.into_inner()) {
        let _ = fs::remove_dir(&staging);
        return Err(e);
    }
    fs::remove_dir(&staging)
}

/// One file of [`upgrade_regions`]: every chunk fixed, then the file written
/// anew in `staging` and moved over the old one. A file without chunks (most
/// of a large world's are empty) has nothing to fix and is left as it is.
fn upgrade_region_file(path: &Path, staging: &Path, fix: &impl Fn(&mut Tag)) -> io::Result<()> {
    let limits = Limits::default();
    let mut region = RegionFile::open(path)?;
    let positions: Vec<_> = region.chunks().map(|(p, _)| p).collect();
    if positions.is_empty() {
        return Ok(());
    }
    let mut records = Vec::with_capacity(positions.len());
    for pos in positions {
        let Some(record) = region.read(pos)? else {
            continue;
        };
        let mut root = record.nbt(limits)?;
        fix(&mut root.tag);
        records.push((
            pos,
            ChunkRecord::from_nbt(&root, record.compression, record.timestamp, limits)?,
        ));
    }
    drop(region);
    let temporary = staging.join(path.file_name().unwrap_or_default());
    let _ = fs::remove_file(&temporary);
    let mut writer = RegionWriter::create(&temporary)?;
    for (pos, record) in &records {
        writer.write(*pos, record)?;
    }
    writer.finish()?;
    fs::rename(&temporary, path)
}

/// The progress a data upgrade reports to and whether it may stop early: a
/// cancel is honoured only while the world's files are still unmoved, since
/// the DataVersion is stamped last and a retry would run the file upgrades again.
struct Progress<'a> {
    progress: &'a UpgradeProgress,
    cancelable: bool,
}
impl Progress<'static> {
    /// A converter run with nobody watching.
    fn none() -> Self {
        static NONE: std::sync::LazyLock<UpgradeProgress> =
            std::sync::LazyLock::new(UpgradeProgress::new);
        Progress {
            progress: &NONE,
            cancelable: false,
        }
    }
}
impl Progress<'_> {
    fn check(&self) -> io::Result<()> {
        if self.cancelable && self.progress.is_canceled() {
            Err(canceled())
        } else {
            Ok(())
        }
    }
}

/// `SavedDataStorage.readTagFromDisk` over every known file of a data folder.
fn saved_data_files(data: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !data.is_dir() {
        return Ok(files);
    }
    let mut stack = vec![data.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)?.filter_map(Result::ok) {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.to_string_lossy().ends_with(".dat") {
                files.push(p);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn upgrade_saved_data(data: &Path, progress: &Progress) -> io::Result<()> {
    for file in saved_data_files(data)? {
        progress.check()?;
        let relative = file
            .strip_prefix(data)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        if let Some(ty) = super::saved_data_type(&relative)
            && let Some(mut tag) = read_saved_dat(&file)?
        {
            super::update_saved_data(ty, &mut tag);
            set(&mut tag, "DataVersion", Tag::Int(CURRENT));
            write_dat(&file, &tag)?;
        }
        progress.progress.increment_finished_operations();
    }
    Ok(())
}

/// A JSON file of the player folders as NBT for the fixes (`JsonOps` and
/// `NbtOps` see the same structure), booleans kept apart.
fn json_to_tag(j: &Json) -> Tag {
    match j {
        Json::Bool(b) => Tag::compound([("\u{0}bool", Tag::Byte(i8::from(*b)))]),
        Json::Arr(items) => Tag::List(items.iter().map(json_to_tag).collect()),
        Json::Obj(entries) => Tag::Compound(
            entries
                .iter()
                .map(|(k, v)| (JavaString(k.clone()), json_to_tag(v)))
                .collect(),
        ),
        other => json::to_nbt(other).unwrap_or(Tag::End),
    }
}
fn tag_to_json(t: &Tag) -> Json {
    match t {
        Tag::Compound(c) if c.len() == 1 && c.contains_key(&JavaString::from("\u{0}bool")) => {
            Json::Bool(as_int(t.get("\u{0}bool"), 0) != 0)
        }
        Tag::List(items) => Json::Arr(items.iter().map(tag_to_json).collect()),
        Tag::Compound(c) => Json::Obj(
            c.iter()
                .map(|(k, v)| (k.0.clone(), tag_to_json(v)))
                .collect(),
        ),
        other => json::from_nbt(other),
    }
}

fn upgrade_json(path: &Path, ty: Ty) -> io::Result<()> {
    upgrade_json_to(path, ty, CURRENT)
}

fn upgrade_json_to(path: &Path, ty: Ty, to: i32) -> io::Result<()> {
    let text = fs::read_to_string(path)?;
    let units: Vec<u16> = text.encode_utf16().collect();
    let parsed = json::parse_lenient(&units)
        .ok_or_else(|| invalid(format!("bad JSON in {}", path.display())))?;
    let mut tag = json_to_tag(&parsed);
    let from = super::dynamic::as_int(tag.get("DataVersion"), 1343);
    if ty == Ty::Advancements {
        // `PlayerAdvancements`: the version field is not part of the data.
        remove(&mut tag, "DataVersion");
    }
    update(ty, &mut tag, from, to);
    set(&mut tag, "DataVersion", Tag::Int(to));
    let out = json::to_stable_string(&tag_to_json(&tag));
    fs::write(path, String::from_utf16_lossy(&out))
}

/// Everything vanilla upgrades as it reads a world, done at once: every chunk,
/// entity chunk and POI chunk, saved data, player, advancements and
/// statistics file. Run after [`fix_world_files`].
pub fn upgrade_world_data(base: &Path) -> io::Result<()> {
    upgrade_world_data_with(base, &UpgradeProgress::new(), false)
}

/// [`upgrade_world_data`] reporting to `progress` (type `Regions`, one
/// operation per file) and stopping on its cancel when `cancelable`.
pub fn upgrade_world_data_with(
    base: &Path,
    progress: &UpgradeProgress,
    cancelable: bool,
) -> io::Result<()> {
    let generators = generators(base)?;
    // 1.21.10 worlds keep the pre-26.x layout: the overworld at the root, the
    // Nether under DIM-1 and the End under DIM1, plus custom dimensions under
    // dimensions/<namespace>/<name>.
    let mut dimensions: Vec<(String, PathBuf)> = vec![
        ("minecraft:overworld".to_string(), base.to_path_buf()),
        ("minecraft:the_nether".to_string(), base.join("DIM-1")),
        ("minecraft:the_end".to_string(), base.join("DIM1")),
    ];
    dimensions.extend(
        subdirectories(&base.join("dimensions"))
            .into_iter()
            .flat_map(|ns| {
                subdirectories(&ns).into_iter().map(move |dim| {
                    let id = format!(
                        "{}:{}",
                        ns.file_name().unwrap_or_default().to_string_lossy(),
                        dim.file_name().unwrap_or_default().to_string_lossy()
                    );
                    (id, dim)
                })
            }),
    );
    let players = files_with_suffix(&base.join("playerdata"), ".dat");
    let advancements = files_with_suffix(&base.join("advancements"), ".json");
    let stats = files_with_suffix(&base.join("stats"), ".json");
    // Counting the files first, so the progress screen knows its total.
    progress.set_type(UpgradeType::Regions);
    let mut total = players.len() + advancements.len() + stats.len();
    for (_, dim) in &dimensions {
        for folder in ["region", "entities", "poi"] {
            total += files_with_suffix(&dim.join(folder), ".mca").len();
        }
        total += saved_data_files(&dim.join("data"))?.len();
    }
    progress.add_total_file_fix_operations(total as i32);
    let run = Progress {
        progress,
        cancelable,
    };
    for (id, dim) in &dimensions {
        let generator = generators
            .iter()
            .find(|(d, _)| d == id)
            .map(|(_, g)| g.clone());
        upgrade_regions(&dim.join("region"), &run, |tag| {
            super::update_chunk(tag, id, generator.as_deref())
        })?;
        upgrade_regions(&dim.join("entities"), &run, |tag| {
            super::update_region_tag(Ty::EntityChunk, tag)
        })?;
        upgrade_regions(&dim.join("poi"), &run, |tag| {
            super::update_region_tag(Ty::PoiChunk, tag)
        })?;
        upgrade_saved_data(&dim.join("data"), &run)?;
    }
    for file in players {
        run.check()?;
        if let Some(mut tag) = read_dat(&file)? {
            super::update_player(&mut tag);
            set(&mut tag, "DataVersion", Tag::Int(CURRENT));
            write_dat(&file, &tag)?;
        }
        progress.increment_finished_operations();
    }
    for file in advancements {
        run.check()?;
        upgrade_json(&file, Ty::Advancements)?;
        progress.increment_finished_operations();
    }
    for file in stats {
        run.check()?;
        upgrade_json(&file, Ty::Stats)?;
        progress.increment_finished_operations();
    }
    Ok(())
}

/// A world in the layout of before 1.16 (dimension folders in the world folder,
/// `playerdata`, `stats`, `advancements`) brought to DataVersion `target` and
/// no further, data fixes only: the harness checks the fixes of a range of
/// versions on their own (the vanilla oracle does the same given a target).
/// 1.21.10 has no file fixes, so no file is moved or renamed here.
pub fn upgrade_band(base: &Path, target: i32) -> io::Result<()> {
    let level = base.join("level.dat");
    if let Some(mut root) = read_dat(&level)? {
        let mut data = root
            .get("Data")
            .cloned()
            .unwrap_or_else(|| Tag::Compound(Compound::new()));
        let from = data_version(&data, -1);
        update(Ty::Level, &mut data, from, target);
        set(&mut data, "DataVersion", Tag::Int(target));
        set(&mut root, "Data", data);
        write_dat(&level, &root)?;
    }
    for (folder, id) in [
        ("", "minecraft:overworld"),
        ("DIM-1", "minecraft:the_nether"),
        ("DIM1", "minecraft:the_end"),
    ] {
        let dim = base.join(folder);
        upgrade_regions(&dim.join("region"), &Progress::none(), |tag| {
            super::update_chunk_to(tag, id, None, target)
        })?;
        upgrade_regions(&dim.join("entities"), &Progress::none(), |tag| {
            super::update_region_tag_to(Ty::EntityChunk, tag, target)
        })?;
        upgrade_regions(&dim.join("poi"), &Progress::none(), |tag| {
            super::update_region_tag_to(Ty::PoiChunk, tag, target)
        })?;
    }
    for file in files_with_suffix(&base.join("playerdata"), ".dat") {
        if let Some(mut tag) = read_dat(&file)? {
            super::update_player_to(&mut tag, target);
            write_dat(&file, &tag)?;
        }
    }
    for (folder, ty) in [("advancements", Ty::Advancements), ("stats", Ty::Stats)] {
        for file in files_with_suffix(&base.join(folder), ".json") {
            upgrade_json_to(&file, ty, target)?;
        }
    }
    // The saved data of the old layout.
    for file in files_with_suffix(&base.join("data"), ".dat") {
        let name = file
            .file_stem()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let ty = match name.as_str() {
            "scoreboard" => Ty::SavedScoreboard,
            n if n
                .strip_prefix("map_")
                .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit())) =>
            {
                Ty::SavedMapData
            }
            _ => continue,
        };
        if let Some(mut tag) = read_dat(&file)? {
            let from = data_version(&tag, 1343);
            update(ty, &mut tag, from, target);
            set(&mut tag, "DataVersion", Tag::Int(target));
            write_dat(&file, &tag)?;
        }
    }
    Ok(())
}

/// Bring a whole world saved by an older version to the 1.21.10 format in place.
pub fn upgrade_world(base: &Path) -> io::Result<i32> {
    upgrade_world_with(base, &UpgradeProgress::new())
}

/// [`upgrade_world`] reporting to `progress`, which also carries the player's
/// cancel (the upgrade screen's button).
pub fn upgrade_world_with(base: &Path, progress: &UpgradeProgress) -> io::Result<i32> {
    let (version, pending) = fix_world_files_deferred(base, progress)?;
    upgrade_world_data_with(base, progress, !progress.is_file_fixing())?;
    // Stamp the current DataVersion only once everything converted, so a failed
    // upgrade is retried on the next open.
    if let Some(data) = pending {
        write_dat(&base.join("level.dat"), &Tag::compound([("Data", data)]))?;
    }
    Ok(version)
}

#[cfg(test)]
mod upgrade_tests {
    use super::*;

    fn level_version(base: &Path) -> i32 {
        let root = read_dat(&base.join("level.dat")).unwrap().unwrap();
        data_version(root.get("Data").unwrap(), -1)
    }

    /// The upgrade screen reads the upgrade's counts: the data stage is
    /// type `Regions` with one operation per file.
    #[test]
    fn upgrade_reports_one_operation_per_file() {
        let base = std::env::temp_dir().join(format!("horizon-progress-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let level = Tag::compound([("Data", Tag::compound([("DataVersion", Tag::Int(4200))]))]);
        write_dat(&base.join("level.dat"), &level).unwrap();
        let player = Tag::compound([("DataVersion", Tag::Int(4200))]);
        write_dat(&base.join("playerdata/a.dat"), &player).unwrap();
        write_dat(&base.join("playerdata/b.dat"), &player).unwrap();

        let progress = UpgradeProgress::new();
        upgrade_world_with(&base, &progress).unwrap();
        assert!(!progress.is_file_fixing());
        assert_eq!(progress.upgrade_type(), Some(UpgradeType::Regions));
        assert_eq!(progress.total_file_fix_stats().total_operations(), 2);
        assert_eq!(progress.total_file_fix_stats().finished_operations(), 2);
        assert_eq!(progress.total_file_fix_stats().progress(), 1.0);
        let _ = fs::remove_dir_all(&base);
    }

    /// Cancel stops the data stage before the next file, and the DataVersion
    /// stays old so the next open retries.
    #[test]
    fn cancel_stops_the_data_upgrade_and_keeps_the_version() {
        let base = std::env::temp_dir().join(format!("horizon-cancel-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let level = Tag::compound([("Data", Tag::compound([("DataVersion", Tag::Int(4200))]))]);
        write_dat(&base.join("level.dat"), &level).unwrap();
        write_dat(
            &base.join("playerdata/a.dat"),
            &Tag::compound([("DataVersion", Tag::Int(4200))]),
        )
        .unwrap();

        let progress = UpgradeProgress::new();
        progress.set_canceled();
        let error = upgrade_world_with(&base, &progress).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        assert_eq!(progress.total_file_fix_stats().finished_operations(), 0);
        assert_eq!(level_version(&base), 4200);
        let _ = fs::remove_dir_all(&base);
    }

    /// A failed data upgrade must keep the old DataVersion so the next open
    /// retries, and a later successful run stamps the current one.
    #[test]
    fn failed_upgrade_keeps_old_data_version() {
        let base = std::env::temp_dir().join(format!("horizon-upgrade-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let old = 4200;
        let level = Tag::compound([("Data", Tag::compound([("DataVersion", Tag::Int(old))]))]);
        write_dat(&base.join("level.dat"), &level).unwrap();
        let region = base.join("region");
        fs::create_dir_all(&region).unwrap();
        // Not a valid region file name, so opening it fails mid-upgrade.
        fs::write(region.join("broken.mca"), b"x").unwrap();

        assert!(upgrade_world(&base).is_err());
        assert_eq!(level_version(&base), old);

        fs::remove_file(region.join("broken.mca")).unwrap();
        assert_eq!(upgrade_world(&base).unwrap(), old);
        assert_eq!(level_version(&base), CURRENT);
        let _ = fs::remove_dir_all(&base);
    }

    fn old_chunk(x: i32, z: i32) -> Tag {
        Tag::compound([
            ("DataVersion", Tag::Int(4000)),
            ("xPos", Tag::Int(x)),
            ("zPos", Tag::Int(z)),
            ("Status", Tag::String("minecraft:full".into())),
        ])
    }

    /// `files` region files of `chunks` old chunks each.
    fn old_region_folder(dir: &Path, files: i32, chunks: i32) {
        fs::create_dir_all(dir).unwrap();
        for rx in 0..files {
            let mut region =
                RegionFile::open_writable(dir.join(format!("r.{rx}.0.mca")), true).unwrap();
            for i in 0..chunks {
                let pos = horizon_core::position::ChunkPos::new(rx * 32 + i, i % 3);
                let root = NamedTag {
                    name: "".into(),
                    tag: old_chunk(pos.x, pos.z),
                };
                let record =
                    ChunkRecord::from_nbt(&root, Compression::Zlib, 7, Limits::default()).unwrap();
                region.write(pos, &record).unwrap();
            }
        }
    }

    /// Every region file's chunk versions, in file then chunk order.
    fn chunk_versions(dir: &Path) -> Vec<Vec<i32>> {
        files_with_suffix(dir, ".mca")
            .iter()
            .map(|file| {
                let mut region = RegionFile::open(file).unwrap();
                let positions: Vec<_> = region.chunks().map(|(p, _)| p).collect();
                positions
                    .into_iter()
                    .map(|pos| {
                        let record = region.read(pos).unwrap().unwrap();
                        data_version(&record.nbt(Limits::default()).unwrap().tag, -1)
                    })
                    .collect()
            })
            .collect()
    }

    fn fix_overworld_chunk(tag: &mut Tag) {
        crate::datafix::update_chunk(tag, "minecraft:overworld", Some("minecraft:noise"));
    }

    /// Region files upgraded on 4 workers come out byte for byte as on one,
    /// every chunk at the current version.
    #[test]
    fn parallel_region_upgrade_matches_one_thread() {
        let base = std::env::temp_dir().join(format!("horizon-par-regions-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let mut outputs = Vec::new();
        for threads in [1, 4] {
            let dir = base.join(format!("threads-{threads}"));
            old_region_folder(&dir, 9, 20);
            // An empty file, as the game leaves for regions it never saved into.
            fs::write(dir.join("r.-1.5.mca"), []).unwrap();
            let progress = UpgradeProgress::new();
            let run = Progress {
                progress: &progress,
                cancelable: true,
            };
            upgrade_regions_with_threads(&dir, &run, fix_overworld_chunk, threads).unwrap();
            assert_eq!(progress.total_file_fix_stats().finished_operations(), 10);
            assert_eq!(
                fs::read(dir.join("r.-1.5.mca")).unwrap(),
                b"",
                "the empty file is left alone"
            );
            fs::remove_file(dir.join("r.-1.5.mca")).unwrap();
            assert_eq!(chunk_versions(&dir), vec![vec![CURRENT; 20]; 9]);
            assert!(!dir.join(".datafix").exists());
            let bytes: Vec<Vec<u8>> = files_with_suffix(&dir, ".mca")
                .iter()
                .map(|f| fs::read(f).unwrap())
                .collect();
            outputs.push(bytes);
        }
        assert!(outputs[0] == outputs[1], "the files differ");
        let _ = fs::remove_dir_all(&base);
    }

    /// A cancel during the region stage stops every worker after the file in
    /// hand: at most one file each is converted, the rest stay old.
    #[test]
    fn cancel_stops_every_region_worker_after_its_file() {
        let dir = std::env::temp_dir().join(format!("horizon-par-cancel-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        old_region_folder(&dir, 9, 20);
        let progress = UpgradeProgress::new();
        let run = Progress {
            progress: &progress,
            cancelable: true,
        };
        let error = upgrade_regions_with_threads(
            &dir,
            &run,
            |tag: &mut Tag| {
                progress.set_canceled();
                fix_overworld_chunk(tag);
            },
            4,
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        let versions = chunk_versions(&dir);
        let converted = versions.iter().filter(|f| **f == [CURRENT; 20]).count();
        let untouched = versions.iter().filter(|f| **f == [4000; 20]).count();
        assert!((1..=4).contains(&converted), "{converted} files converted");
        assert_eq!(converted + untouched, 9);
        assert_eq!(
            progress.total_file_fix_stats().finished_operations() as usize,
            converted
        );
        assert!(!dir.join(".datafix").exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
