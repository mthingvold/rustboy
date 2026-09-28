//! `Library`: scan folders, game list and persisted settings, exposed to QML.
//!
//! Lists are passed to QML as JSON strings; QML parses them into JS arrays for its views.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, folders_json)]
        #[qproperty(QString, games_json)]
        #[qproperty(i32, game_count)]
        #[qproperty(bool, scanning)]
        #[qproperty(i32, scanned_files)]
        #[qproperty(bool, has_scanned)]
        #[qproperty(bool, include_subfolders)]
        #[qproperty(bool, rescan_on_start)]
        #[qproperty(i32, lcd_palette)]
        type Library = super::LibraryRust;

        /// Load settings and the cached library; rescan if the user asked for that.
        #[qinvokable]
        fn startup(self: Pin<&mut Library>);

        /// Add a folder (local path or `file://` URL). Duplicates are ignored.
        #[qinvokable]
        fn add_folder(self: Pin<&mut Library>, folder: &QString);

        /// Remove a folder. Its games disappear after the next scan.
        #[qinvokable]
        fn remove_folder(self: Pin<&mut Library>, folder: &QString);

        /// Scan all folders in the background.
        #[qinvokable]
        fn scan(self: Pin<&mut Library>);

        /// Stop a running scan; the previous game list is kept.
        #[qinvokable]
        fn cancel_scan(self: Pin<&mut Library>);

        /// Record that a ROM was just launched (for "Recently played").
        #[qinvokable]
        fn mark_played(self: Pin<&mut Library>, path: &QString);

        /// Persist the option properties (subfolders, rescan, palette).
        #[qinvokable]
        fn save_settings(self: Pin<&mut Library>);

        /// A scan ended. `count` is the number of ROMs found.
        #[qsignal]
        fn scan_finished(self: Pin<&mut Library>, count: i32, cancelled: bool);
    }

    impl cxx_qt::Threading for Library {}
}

use core::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use super::emulator::local_path;
use crate::library::{self, Game};
use crate::settings::{self, Settings};

/// Rust state behind `Library`.
pub struct LibraryRust {
    folders_json: QString,
    games_json: QString,
    game_count: i32,
    scanning: bool,
    scanned_files: i32,
    has_scanned: bool,
    include_subfolders: bool,
    rescan_on_start: bool,
    lcd_palette: i32,
    settings: Settings,
    games: Vec<Game>,
    cancel: Arc<AtomicBool>,
    /// Settings have been read from disk; nothing is written before that.
    started: bool,
}

impl Default for LibraryRust {
    fn default() -> Self {
        LibraryRust {
            folders_json: QString::from("[]"),
            games_json: QString::from("[]"),
            game_count: 0,
            scanning: false,
            scanned_files: 0,
            has_scanned: false,
            include_subfolders: true,
            rescan_on_start: true,
            lcd_palette: 0,
            settings: Settings::default(),
            games: Vec::new(),
            cancel: Arc::new(AtomicBool::new(false)),
            started: false,
        }
    }
}

/// One grid entry as QML sees it.
#[derive(serde::Serialize)]
struct GameView<'a> {
    path: &'a str,
    file: &'a str,
    title: &'a str,
    sys: &'static str,
    meta: String,
    c1: &'static str,
    c2: &'static str,
    #[serde(rename = "playedAt")]
    played_at: i64,
}

impl qobject::Library {
    fn persist(&self) {
        if !self.started {
            log::warn!(target: "Core", "Settings changed before startup(); not saved");
            return;
        }
        let dir = settings::config_dir();
        if let Err(e) = self.settings.save(&dir) {
            log::warn!(target: "Core", "Couldn't save settings: {}", e);
        }
    }

    fn publish_folders(mut self: Pin<&mut Self>) {
        let json = serde_json::to_string(&self.settings.folders).unwrap_or_else(|_| "[]".into());
        self.as_mut().set_folders_json(QString::from(&json));
    }

    fn publish_games(mut self: Pin<&mut Self>) {
        let now = chrono::Local::now();
        let views: Vec<GameView> = self
            .games
            .iter()
            .map(|g| {
                let played = self.settings.last_played.get(&g.path).copied();
                let (c1, c2) = library::cover_colors(&g.title);
                GameView {
                    path: &g.path,
                    file: &g.file,
                    title: &g.title,
                    sys: g.system.label(),
                    meta: format!(
                        "{} · {} · {}",
                        g.system.label(),
                        library::format_size(g.bytes),
                        library::played_text(played, now)
                    ),
                    c1,
                    c2,
                    played_at: played.unwrap_or(0),
                }
            })
            .collect();
        let json = serde_json::to_string(&views).unwrap_or_else(|_| "[]".into());
        let count = self.games.len() as i32;
        self.as_mut().set_games_json(QString::from(&json));
        self.as_mut().set_game_count(count);
    }

    fn startup(mut self: Pin<&mut Self>) {
        if self.started {
            return;
        }
        let dir = settings::config_dir();
        let loaded = Settings::load(&dir);
        let games = settings::load_library(&dir);

        self.as_mut()
            .set_include_subfolders(loaded.include_subfolders);
        self.as_mut().set_rescan_on_start(loaded.rescan_on_start);
        self.as_mut()
            .set_lcd_palette(if loaded.lcd_palette == "Gray" { 1 } else { 0 });
        self.as_mut().set_has_scanned(loaded.scanned);
        let rescan = loaded.rescan_on_start && !loaded.folders.is_empty();
        {
            let mut rust = self.as_mut().rust_mut();
            rust.settings = loaded;
            rust.games = games;
            rust.started = true;
        }
        self.as_mut().publish_folders();
        self.as_mut().publish_games();

        if rescan {
            self.scan();
        }
    }

    fn add_folder(mut self: Pin<&mut Self>, folder: &QString) {
        let path = local_path(&folder.to_string());
        if path.is_empty() || self.settings.folders.contains(&path) {
            return;
        }
        log::info!(target: "Library", "Added folder {}", path);
        self.as_mut().rust_mut().settings.folders.push(path);
        self.as_ref().persist();
        self.publish_folders();
    }

    fn remove_folder(mut self: Pin<&mut Self>, folder: &QString) {
        let path = folder.to_string();
        self.as_mut()
            .rust_mut()
            .settings
            .folders
            .retain(|f| *f != path);
        log::info!(target: "Library", "Removed folder {}", path);
        self.as_ref().persist();
        self.publish_folders();
    }

    fn scan(mut self: Pin<&mut Self>) {
        if self.scanning || self.settings.folders.is_empty() {
            return;
        }
        let folders = self.settings.folders.clone();
        let recursive = self.include_subfolders;
        let cancel = Arc::new(AtomicBool::new(false));
        self.as_mut().rust_mut().cancel = cancel.clone();
        self.as_mut().set_scanned_files(0);
        self.as_mut().set_scanning(true);

        log::info!(
            target: "Library",
            "Scanning {} folder{} (subfolders {})",
            folders.len(),
            if folders.len() == 1 { "" } else { "s" },
            if recursive { "on" } else { "off" }
        );

        let qt_thread = self.qt_thread();
        std::thread::spawn(move || {
            let progress_thread = qt_thread.clone();
            let result = library::scan(&folders, recursive, &cancel, |seen| {
                let _ = progress_thread.queue(move |obj| obj.set_scanned_files(seen as i32));
            });
            let _ = qt_thread.queue(move |obj| obj.finish_scan(result.ok()));
        });
    }

    fn finish_scan(mut self: Pin<&mut Self>, games: Option<Vec<Game>>) {
        self.as_mut().set_scanning(false);
        let games = match games {
            Some(games) => games,
            None => {
                log::info!(target: "Library", "Scan cancelled");
                self.scan_finished(0, true);
                return;
            }
        };

        let count = games.len() as i32;
        log::info!(target: "Library", "Found {} ROMs", count);
        if let Err(e) = settings::save_library(&settings::config_dir(), &games) {
            log::warn!(target: "Library", "Couldn't cache library: {}", e);
        }
        {
            let mut rust = self.as_mut().rust_mut();
            rust.games = games;
            rust.settings.scanned = true;
        }
        self.as_ref().persist();
        self.as_mut().set_has_scanned(true);
        self.as_mut().publish_games();
        self.scan_finished(count, false);
    }

    fn cancel_scan(self: Pin<&mut Self>) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    fn mark_played(mut self: Pin<&mut Self>, path: &QString) {
        let path = local_path(&path.to_string());
        let now = chrono::Local::now().timestamp();
        self.as_mut()
            .rust_mut()
            .settings
            .last_played
            .insert(path, now);
        self.as_ref().persist();
        self.publish_games();
    }

    fn save_settings(mut self: Pin<&mut Self>) {
        let include = self.include_subfolders;
        let rescan = self.rescan_on_start;
        let palette = if self.lcd_palette == 1 {
            "Gray"
        } else {
            "Green"
        };
        {
            let mut rust = self.as_mut().rust_mut();
            rust.settings.include_subfolders = include;
            rust.settings.rescan_on_start = rescan;
            rust.settings.lcd_palette = palette.to_string();
        }
        self.as_ref().persist();
    }
}
