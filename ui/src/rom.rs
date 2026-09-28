//! ROM file handling for the frontend: reading `.gb`/`.gbc`/`.zip` files and parsing the
//! cartridge header without going through the core (whose `Cartridge::new` panics on
//! unusual headers; see ui/PLANNED_WORK.md).

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// File extensions the library and "Load ROM…" accept.
pub const ROM_EXTENSIONS: [&str; 3] = ["gb", "gbc", "zip"];

/// Header offset of the CGB flag byte.
const CGB_FLAG: usize = 0x0143;

/// Which console a ROM targets, from the header's CGB flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum System {
    /// Original Game Boy (DMG) only.
    GB,
    /// Game Boy Color: either CGB-enhanced or CGB-only.
    GBC,
}

impl System {
    /// Short label shown on cover badges ("GB" / "GBC").
    pub fn label(self) -> &'static str {
        match self {
            System::GB => "GB",
            System::GBC => "GBC",
        }
    }
}

/// The parts of a cartridge header the UI displays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RomHeader {
    /// Title from the header, trimmed; may be empty.
    pub title: String,
    /// Target console.
    pub system: System,
}

/// Parse the cartridge header of `data`. Returns `None` when `data` is too short to hold one.
pub fn parse_header(data: &[u8]) -> Option<RomHeader> {
    if data.len() < 0x0150 {
        return None;
    }

    let cgb = data[CGB_FLAG] & 0x80 != 0;

    // CGB cartridges reuse the last title bytes for the manufacturer code and CGB flag
    let title_end = if cgb { 0x013F } else { 0x0144 };
    let title: String = data[0x0134..title_end]
        .iter()
        .take_while(|b| **b != 0)
        .map(|b| {
            if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                ' '
            }
        })
        .collect();

    Some(RomHeader {
        title: title.trim().to_string(),
        system: if cgb { System::GBC } else { System::GB },
    })
}

/// Whether `path` has one of the [`ROM_EXTENSIONS`] (case-insensitive).
pub fn has_rom_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| ROM_EXTENSIONS.iter().any(|x| x.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

/// Read a ROM image from disk. For a `.zip`, the first `.gb`/`.gbc` entry is returned.
pub fn read_rom(path: &Path) -> Result<Vec<u8>, String> {
    let is_zip = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false);

    if !is_zip {
        return std::fs::read(path).map_err(|e| format!("Couldn't read {}: {}", path.display(), e));
    }

    let file = File::open(path).map_err(|e| format!("Couldn't open {}: {}", path.display(), e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("{} isn't a valid zip: {}", path.display(), e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let lower = name.to_ascii_lowercase();
        if entry.is_file() && (lower.ends_with(".gb") || lower.ends_with(".gbc")) {
            let mut data = Vec::with_capacity(entry.size() as usize);
            entry
                .read_to_end(&mut data)
                .map_err(|e| format!("Couldn't extract {} from {}: {}", name, path.display(), e))?;
            return Ok(data);
        }
    }

    Err(format!("{} contains no .gb or .gbc file", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_with(title: &[u8], cgb_flag: u8) -> Vec<u8> {
        let mut data = vec![0u8; 0x8000];
        data[0x0134..0x0134 + title.len()].copy_from_slice(title);
        data[CGB_FLAG] = cgb_flag;
        data
    }

    #[test]
    fn parses_dmg_title() {
        let h = parse_header(&header_with(b"TETRIS", 0x00)).unwrap();
        assert_eq!(h.title, "TETRIS");
        assert_eq!(h.system, System::GB);
    }

    #[test]
    fn cgb_title_excludes_manufacturer_code() {
        let h = parse_header(&header_with(b"POKEMON YELLAPSE", 0x80)).unwrap();
        assert_eq!(h.title, "POKEMON YEL");
        assert_eq!(h.system, System::GBC);
    }

    #[test]
    fn non_ascii_title_does_not_panic() {
        let h = parse_header(&header_with(&[0xFF, b'A', 0xC3], 0)).unwrap();
        assert_eq!(h.title, "A");
    }

    #[test]
    fn short_file_has_no_header() {
        assert!(parse_header(&[0u8; 0x100]).is_none());
    }

    #[test]
    fn extension_check_is_case_insensitive() {
        assert!(has_rom_extension(Path::new("a/b/Game.GBC")));
        assert!(has_rom_extension(Path::new("x.zip")));
        assert!(!has_rom_extension(Path::new("x.state")));
    }

    #[test]
    fn reads_rom_from_zip() {
        let dir = std::env::temp_dir().join(format!("pocketbox-zip-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let zip_path = dir.join("game.zip");
        {
            let mut w = zip::ZipWriter::new(File::create(&zip_path).unwrap());
            let opts = zip::write::SimpleFileOptions::default();
            w.start_file("readme.txt", opts).unwrap();
            std::io::Write::write_all(&mut w, b"hi").unwrap();
            w.start_file("game.gb", opts).unwrap();
            std::io::Write::write_all(&mut w, &[1, 2, 3]).unwrap();
            w.finish().unwrap();
        }
        assert_eq!(read_rom(&zip_path).unwrap(), vec![1, 2, 3]);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
