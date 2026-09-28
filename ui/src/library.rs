//! Library scanning: finds ROMs in the user's folders and describes them for the grid.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};

use crate::rom::{self, System};

/// Cover colour pairs (background, foreground) used until real box art exists.
const COVER_COLORS: [(&str, &str); 9] = [
    ("#9bbc0f", "#0f380f"),
    ("#f2a65a", "#3b1f0e"),
    ("#2f3e75", "#f4e9c1"),
    ("#d8d4c8", "#2b2a27"),
    ("#c4513f", "#fff1e0"),
    ("#306230", "#d9e8a2"),
    ("#e3b552", "#2e2208"),
    ("#5c7a6b", "#f0efe6"),
    ("#1f1f1d", "#b5cc3a"),
];

/// One ROM found by a scan.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Game {
    /// Absolute path of the ROM file.
    pub path: String,
    /// File name including extension.
    pub file: String,
    /// Display title: the file name without extension, or the header title if that is empty.
    pub title: String,
    /// Target console from the header.
    pub system: System,
    /// File size in bytes.
    pub bytes: u64,
}

/// Why a scan stopped early.
#[derive(Debug, PartialEq, Eq)]
pub struct Cancelled;

/// Scan `folders` for ROMs. `progress` is called with the number of files looked at so far.
/// Returns the games sorted by title, or `Cancelled` if `cancel` was set.
pub fn scan(
    folders: &[String],
    recursive: bool,
    cancel: &AtomicBool,
    mut progress: impl FnMut(usize),
) -> Result<Vec<Game>, Cancelled> {
    let mut games = Vec::new();
    let mut seen = 0usize;
    let mut stack: Vec<(PathBuf, bool)> =
        folders.iter().map(|f| (PathBuf::from(f), true)).collect();

    while let Some((dir, is_root)) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            return Err(Cancelled);
        }

        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) => {
                if is_root {
                    log::warn!(target: "Library", "Can't read {}: {}", dir.display(), e);
                }
                continue;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            // Symlinked folders are not followed, which also rules out cycles
            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };

            if file_type.is_dir() {
                if recursive {
                    stack.push((path, false));
                }
                continue;
            }

            if !rom::has_rom_extension(&path) {
                continue;
            }

            seen += 1;
            if seen % 16 == 0 {
                progress(seen);
                if cancel.load(Ordering::Relaxed) {
                    return Err(Cancelled);
                }
            }

            match describe(&path) {
                Some(game) => games.push(game),
                None => {
                    log::debug!(target: "Library", "Skipped {} (no valid header)", path.display())
                }
            }
        }
    }

    progress(seen);
    games.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then(a.path.cmp(&b.path))
    });
    games.dedup_by(|a, b| a.path == b.path);
    Ok(games)
}

/// Read enough of `path` to describe it, or `None` if it isn't a usable ROM.
pub fn describe(path: &Path) -> Option<Game> {
    let data = if path
        .extension()
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
    {
        rom::read_rom(path).ok()?
    } else {
        let mut buf = vec![0u8; 0x150];
        let mut file = std::fs::File::open(path).ok()?;
        std::io::Read::read_exact(&mut file, &mut buf).ok()?;
        buf
    };

    let header = rom::parse_header(&data)?;
    let bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let stem = path.file_stem()?.to_string_lossy().to_string();
    let title = if stem.trim().is_empty() {
        header.title.clone()
    } else {
        stem
    };

    Some(Game {
        path: path.to_string_lossy().to_string(),
        file: path.file_name()?.to_string_lossy().to_string(),
        title,
        system: header.system,
        bytes,
    })
}

/// Cover colours for a title, stable across runs.
pub fn cover_colors(title: &str) -> (&'static str, &'static str) {
    // FNV-1a, so the choice doesn't depend on std's randomised hasher
    let mut hash: u32 = 0x811c_9dc5;
    for b in title.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    COVER_COLORS[(hash as usize) % COVER_COLORS.len()]
}

/// Human-readable size, e.g. `256 KB` or `1 MB`.
pub fn format_size(bytes: u64) -> String {
    const MB: u64 = 1024 * 1024;
    if bytes >= MB {
        let mb = bytes as f64 / MB as f64;
        if (mb - mb.round()).abs() < 0.05 {
            format!("{} MB", mb.round() as u64)
        } else {
            format!("{:.1} MB", mb)
        }
    } else {
        format!("{} KB", (bytes + 1023) / 1024)
    }
}

/// "Played yesterday"-style text for a last-played Unix timestamp (`None` = never).
pub fn played_text(last_played: Option<i64>, now: chrono::DateTime<Local>) -> String {
    let ts = match last_played.and_then(|t| Local.timestamp_opt(t, 0).single()) {
        Some(ts) => ts,
        None => return "Never played".to_string(),
    };

    let days = (now.date_naive() - ts.date_naive()).num_days().max(0);
    match days {
        0 => "Played today".to_string(),
        1 => "Played yesterday".to_string(),
        2..=6 => format!("Played {} days ago", days),
        7..=13 => "Played last week".to_string(),
        14..=30 => format!("Played {} weeks ago", days / 7),
        31..=61 => "Played last month".to_string(),
        62..=364 => format!("Played {} months ago", days / 30),
        _ => "Played over a year ago".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_rom(path: &Path, cgb: bool) {
        let mut data = vec![0u8; 0x8000];
        data[0x0134..0x0138].copy_from_slice(b"TEST");
        data[0x0143] = if cgb { 0xC0 } else { 0 };
        std::fs::write(path, data).unwrap();
    }

    #[test]
    fn scans_recursively_and_respects_flag_and_cancel() {
        let root = std::env::temp_dir().join(format!("pocketbox-scan-{}", std::process::id()));
        let sub = root.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        write_rom(&root.join("Zelda.gbc"), true);
        write_rom(&sub.join("alpha.gb"), false);
        std::fs::write(root.join("notes.txt"), "x").unwrap();
        std::fs::write(root.join("broken.gb"), [0u8; 10]).unwrap();

        let folders = vec![root.to_string_lossy().to_string()];
        let never = AtomicBool::new(false);

        let all = scan(&folders, true, &never, |_| {}).unwrap();
        let titles: Vec<_> = all.iter().map(|g| g.title.as_str()).collect();
        assert_eq!(titles, vec!["alpha", "Zelda"]);
        assert_eq!(all[1].system, System::GBC);
        assert_eq!(all[0].bytes, 0x8000);

        let top = scan(&folders, false, &never, |_| {}).unwrap();
        assert_eq!(top.len(), 1);

        let cancelled = AtomicBool::new(true);
        assert_eq!(scan(&folders, true, &cancelled, |_| {}), Err(Cancelled));

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn sizes_and_played_text() {
        assert_eq!(format_size(256 * 1024), "256 KB");
        assert_eq!(format_size(1024 * 1024), "1 MB");
        assert_eq!(format_size(1536 * 1024), "1.5 MB");

        let now = Local::now();
        let day = 24 * 3600;
        assert_eq!(played_text(None, now), "Never played");
        assert_eq!(played_text(Some(now.timestamp()), now), "Played today");
        assert_eq!(
            played_text(Some(now.timestamp() - 10 * day), now),
            "Played last week"
        );
    }

    #[test]
    fn cover_colors_are_stable() {
        assert_eq!(cover_colors("Tetris"), cover_colors("Tetris"));
    }
}
