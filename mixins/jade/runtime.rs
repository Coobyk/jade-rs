// Jade: the block, fluid or entity under your crosshair in a corner of the
// screen. This file is both the mod's Cargo library (its tests run with
// `cargo test`) and the module the bundle adds to the game as
// crate::game::jade. It stays free of game types so `cargo test` compiles it
// standalone: the game-facing helpers translate between them and this logic.

/// The mod's id (its settings folder), name and version.
pub const ID: &str = "jade";
pub const NAME: &str = "Jade";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "The block, fluid or entity under your crosshair in a corner of the screen. /jade hides or shows it, the settings page moves it.";
/// The WebAssembly mods' panel: translucent black, white text with a shadow.
pub const BACKGROUND: u32 = 0x7800_0000;
pub const INK: u32 = 0xFF_FFFF;

/// The box's corners, as the position setting stores them: their index into
/// this table is the whole stored value (`0` is the default, top right).
pub const CORNERS: [&str; 4] = ["top_left", "top_right", "bottom_left", "bottom_right"];
pub const CORNER_LABELS: [&str; 4] = ["Top left", "Top right", "Bottom left", "Bottom right"];
/// The default corner: top right, where the debug screen (F3, top left) and
/// Horizon's own HUD boxes leave room.
pub const DEFAULT_CORNER: usize = 1;

/// `Stone of the World` style name from a registry path (`iron_ore`).
pub fn pretty(path: &str) -> String {
    let mut out = String::new();
    for word in path.split('_') {
        if word.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

/// The block's or fluid's name in the player's language: `block.<namespace>.<path>`
/// in the language file, else the path in words. `translate` is a `lang::get`
/// that answers `None` when the key is missing.
pub fn display_name(full_id: &str, translate: impl Fn(&str) -> Option<String>) -> String {
    let (namespace, path) = split_id(full_id);
    translate(&format!("block.{namespace}.{path}")).unwrap_or_else(|| pretty(path))
}

/// The entity's name in the player's language: `entity.<namespace>.<path>`,
/// else the path in words. Horizon's mob ids carry no namespace, so mobs are
/// looked up under `minecraft`.
pub fn entity_name(id: &str, translate: impl Fn(&str) -> Option<String>) -> String {
    let (namespace, path) = split_id(id);
    translate(&format!("entity.{namespace}.{path}")).unwrap_or_else(|| pretty(path))
}

/// `("minecraft", "stone")` from `minecraft:stone`.
pub fn split_id(full_id: &str) -> (&str, &str) {
    full_id.split_once(':').unwrap_or(("minecraft", full_id))
}

/// `12, 64, -4` — the block's coordinates as `/where` writes them.
pub fn pos_line(pos: [i32; 3]) -> String {
    format!("{}, {}, {}", pos[0], pos[1], pos[2])
}

/// `4.3 m` — how far the hit point is from the player's eyes.
pub fn distance_line(meters: f64) -> String {
    format!("{meters:.1} m")
}

/// The distance in blocks between two positions.
pub fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// The stored `shown` value: shown unless it says `false`.
pub fn read_shown(stored: Option<String>) -> bool {
    stored.is_none_or(|v| v.trim() != "false")
}

/// The stored `coords` value: the coordinates line shows unless it says `false`.
pub fn read_coords(stored: Option<String>) -> bool {
    stored.is_none_or(|v| v.trim() != "false")
}

/// The stored position: the corner's index, `top_left` first in the table.
pub fn read_corner(stored: Option<String>) -> usize {
    let Some(v) = stored else {
        return DEFAULT_CORNER;
    };
    CORNERS
        .iter()
        .position(|c| *c == v.trim())
        .unwrap_or(DEFAULT_CORNER)
}

pub fn shown_label(shown: bool) -> String {
    format!("Shown: {}", if shown { "ON" } else { "OFF" })
}

pub fn coords_label(coords: bool) -> String {
    format!("Coordinates: {}", if coords { "ON" } else { "OFF" })
}

pub fn corner_label(corner: usize) -> String {
    format!("Position: {}", CORNER_LABELS[corner % CORNERS.len()])
}

/// The next corner in the table, cycling through all four.
pub fn next_corner(corner: usize) -> usize {
    (corner + 1) % CORNERS.len()
}

thread_local! {
    /// Whether the box shows, the coordinates line shows, and which corner
    /// it sits in, each read once from its file, then kept.
    static SHOWN: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    static COORDS: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    static CORNER: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

/// Whether the box shows, read once through `get` (the `shown` file).
pub fn shown(get: impl FnOnce() -> Option<String>) -> bool {
    SHOWN.with(|s| {
        let found = s.get().unwrap_or_else(|| read_shown(get()));
        s.set(Some(found));
        found
    })
}

/// Replaces it (the toggle key, `/jade` or the settings page changed it).
pub fn set_shown(shown: bool) {
    SHOWN.with(|s| s.set(Some(shown)));
}

/// Whether the coordinates line shows, read once through `get`.
pub fn coords(get: impl FnOnce() -> Option<String>) -> bool {
    COORDS.with(|c| {
        let found = c.get().unwrap_or_else(|| read_coords(get()));
        c.set(Some(found));
        found
    })
}

pub fn set_coords(coords: bool) {
    COORDS.with(|c| c.set(Some(coords)));
}

/// The corner's index, read once through `get`.
pub fn corner(get: impl FnOnce() -> Option<String>) -> usize {
    CORNER.with(|c| {
        let found = c.get().unwrap_or_else(|| read_corner(get()));
        c.set(Some(found));
        found
    })
}

pub fn set_corner(corner: usize) {
    CORNER.with(|c| c.set(Some(corner)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translate(key: &str) -> Option<String> {
        match key {
            "block.minecraft.stone" => Some("Stone".into()),
            "block.kaisyn.ruby_block" => Some("Block of Ruby".into()),
            "entity.minecraft.pig" => Some("Pig".into()),
            _ => None,
        }
    }

    #[test]
    fn pretty_capitalizes_words() {
        assert_eq!(pretty("stone"), "Stone");
        assert_eq!(pretty("iron_ore"), "Iron Ore");
        assert_eq!(pretty("deepslate_diamond_ore"), "Deepslate Diamond Ore");
        assert_eq!(pretty(""), "");
    }

    #[test]
    fn display_name_uses_the_language_file() {
        assert_eq!(display_name("minecraft:stone", translate), "Stone");
        assert_eq!(display_name("kaisyn:ruby_block", translate), "Block of Ruby");
    }

    #[test]
    fn display_name_falls_back_to_words() {
        assert_eq!(display_name("minecraft:iron_ore", translate), "Iron Ore");
        assert_eq!(display_name("stone", translate), "Stone");
    }

    #[test]
    fn entity_names_are_looked_up_under_their_namespace() {
        assert_eq!(entity_name("pig", translate), "Pig");
        assert_eq!(entity_name("minecraft:pig", translate), "Pig");
        assert_eq!(entity_name("minecraft:zombie_horse", translate), "Zombie Horse");
    }

    #[test]
    fn lines_read_well() {
        assert_eq!(pos_line([12, 64, -4]), "12, 64, -4");
        assert_eq!(distance_line(4.26), "4.3 m");
        assert_eq!(distance([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]), 5.0);
    }

    #[test]
    fn shown_unless_stored_false() {
        assert!(read_shown(None));
        assert!(read_shown(Some("true".into())));
        assert!(!read_shown(Some("false\n".into())));
        assert_eq!(shown_label(false), "Shown: OFF");
    }

    #[test]
    fn coords_unless_stored_false() {
        assert!(read_coords(None));
        assert!(!read_coords(Some("false".into())));
        assert_eq!(coords_label(false), "Coordinates: OFF");
    }

    #[test]
    fn corners_read_and_cycle() {
        assert_eq!(read_corner(None), DEFAULT_CORNER);
        assert_eq!(read_corner(Some("bottom_left".into())), 2);
        assert_eq!(read_corner(Some("nonsense".into())), DEFAULT_CORNER);
        assert_eq!(corner_label(1), "Position: Top right");
        assert_eq!(next_corner(3), 0);
        assert_eq!(next_corner(2), 3);
    }
}
