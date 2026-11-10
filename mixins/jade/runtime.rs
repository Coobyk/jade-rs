// Jade: the block, fluid or entity under your crosshair in a corner of the
// screen. This file is both the mod's Cargo library (its tests run with
// `cargo test`) and the module the bundle adds to the game as
// crate::game::jade. It stays free of game types so `cargo test` compiles it
// standalone: the game-facing helpers translate between them and this logic.

/// The mod's id (its settings folder), name and version.
pub const ID: &str = "jade";
pub const NAME: &str = "Jade";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "The block, fluid or entity under your crosshair, like Java's Jade. /jade hides or shows it, the settings page moves it.";

/// Where the box sits: `top` is Jade's own spot, centred over the hotbar;
/// the four corners are the overlays' places. The index into this table is
/// the stored value.
pub const POSITIONS: [&str; 5] = [
    "top",
    "top_left",
    "top_right",
    "bottom_left",
    "bottom_right",
];
pub const POSITION_LABELS: [&str; 5] =
    ["Top", "Top left", "Top right", "Bottom left", "Bottom right"];
/// Centred at the top, as Java Jade places it.
pub const DEFAULT_POSITION: usize = 0;

/// The highest `age` a crop counts to, for `minecraft:wheat[age=7]`-style
/// progress — `None` for blocks whose `age` isn't a percentage (bamboo, kelp).
fn crop_max_age(id: &str) -> Option<u8> {
    match id.rsplit(':').next().unwrap_or(id) {
        "wheat" | "carrots" | "potatoes" | "beetroots" | "melon_stem"
        | "pumpkin_stem" | "torchflower_crop" | "pitcher_crop" => Some(7),
        "cocoa" => Some(2),
        "nether_wart" | "sweet_berry_bush" => Some(3),
        "chorus_flower" => Some(5),
        _ => None,
    }
}

/// `§771%` for `wheat[age=5]`'s growth line, `§7age: 4` for a counter's —
/// the gray line under a block's name, `None` when the block has no `age`.
pub fn crop_line(id: &str, age: &str) -> Option<String> {
    let age = age.parse::<u8>().ok()?;
    match crop_max_age(id) {
        Some(max) => Some(format!("§7{}%", age as u32 * 100 / max as u32)),
        None => Some(format!("§7age: {age}")),
    }
}

/// The mob's fact lines (`mob.extra` in words), as Jade's entity providers
/// write them between the name and the mod's line.
pub fn mob_lines(age: i32, sheared: bool, saddled: bool, chested: bool) -> Vec<&'static str> {
    let mut out = Vec::new();
    if age < 0 {
        out.push("§7Baby");
    }
    if sheared {
        out.push("§7Sheared");
    }
    if saddled {
        out.push("§7Saddled");
    }
    if chested {
        out.push("§7Chested");
    }
    out
}

/// The box texture, baked once by [`box_texture`] and uploaded into the mod
/// image sprite — a copy of Jade's own `dark` theme
/// (`tooltip/dark_background.png`): an opaque near-black box with a one-pixel
/// border whose grey lightens toward the top, wrapped in a fill-coloured
/// shadow ring with cut corners and a transparent margin. `BOX_TEX` px
/// square, cut into `BOX_SLICE` px ninths when drawn: a slice is the margin,
/// the shadow and the border, so only the fill ever stretches.
pub const BOX_TEX: u32 = 24;
pub const BOX_SLICE: f32 = 4.0;
/// `dark_background.png`'s colours: the opaque fill, which doubles as the
/// shadow ring, and the border's top and bottom greys.
const FILL: (u8, u8, u8, u8) = (19, 19, 19, 200);
const BORDER_TOP: (u8, u8, u8) = (56, 56, 56);
const BORDER_BOTTOM: (u8, u8, u8) = (36, 36, 36);
/// The transparent margin, in pixels — `BOX_SLICE` is margin + shadow +
/// border.
const MARGIN: u32 = 2;

/// `BOX_TEX`² RGBA pixels: margin, shadow ring (its four corners cut),
/// gradient border ring, fill — read as a `d`-deep ring per pixel.
pub fn box_texture() -> Vec<u8> {
    let n = BOX_TEX;
    let mut rgba = vec![0u8; (n * n * 4) as usize];
    for py in 0..n {
        for px in 0..n {
            let d = px.min(py).min(n - 1 - px).min(n - 1 - py);
            if d < MARGIN {
                continue;
            }
            let i = ((py * n + px) * 4) as usize;
            if d == MARGIN {
                // The shadow ring, in the fill's colour, its corners cut.
                if px == MARGIN && py == MARGIN
                    || px == n - 1 - MARGIN && py == MARGIN
                    || px == MARGIN && py == n - 1 - MARGIN
                    || px == n - 1 - MARGIN && py == n - 1 - MARGIN
                {
                    continue;
                }
                rgba[i..i + 4].copy_from_slice(&[FILL.0, FILL.1, FILL.2, FILL.3]);
            } else if d == MARGIN + 1 {
                // The border ring, grey lightening toward the top.
                let t = py as f32 / (n - 1) as f32;
                let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
                rgba[i..i + 4].copy_from_slice(&[
                    mix(BORDER_TOP.0, BORDER_BOTTOM.0),
                    mix(BORDER_TOP.1, BORDER_BOTTOM.1),
                    mix(BORDER_TOP.2, BORDER_BOTTOM.2),
                    255,
                ]);
            } else {
                rgba[i..i + 4].copy_from_slice(&[FILL.0, FILL.1, FILL.2, FILL.3]);
            }
        }
    }
    rgba
}

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

/// `("minecraft", "stone")` from `minecraft:stone`.
pub fn split_id(full_id: &str) -> (&str, &str) {
    full_id.split_once(':').unwrap_or(("minecraft", full_id))
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

/// The mod that added the thing, as Jade writes it under the name:
/// "Minecraft" for vanilla, the namespace in words otherwise.
pub fn mod_name(full_id: &str) -> String {
    let (namespace, _) = split_id(full_id);
    if namespace == "minecraft" {
        "Minecraft".into()
    } else {
        pretty(namespace)
    }
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

/// The item id of the weakest tool that harvests the block — drawn at the
/// tooltip's corner like Jade's harvest-tool line — or a barrier when it's
/// unbreakable. `in_tag` answers a block tag's membership.
pub fn harvest_tool(hardness: f32, in_tag: impl Fn(&str) -> bool) -> Option<String> {
    if hardness < 0.0 {
        return Some("minecraft:barrier".to_owned());
    }
    let tool = ["pickaxe", "axe", "shovel", "hoe"]
        .iter()
        .find(|t| in_tag(&format!("minecraft:mineable/{t}")))?;
    let tier = ["diamond", "iron", "stone"]
        .iter()
        .find(|t| in_tag(&format!("minecraft:needs_{t}_tool")))
        .copied()
        .unwrap_or("wooden");
    Some(format!("minecraft:{tier}_{tool}"))
}

/// `(full hearts, half hearts, container hearts)` for an entity's health
/// row, at most ten slots wide as Jade caps it.
pub fn heart_row(health: f32, max_health: f32) -> (u32, u32, u32) {
    let slots = (max_health / 2.0).ceil().clamp(1.0, 10.0) as u32;
    let full = (health / 2.0).floor().max(0.0) as u32;
    let half = u32::from(health > 0.0 && health % 2.0 != 0.0);
    let full = full.min(slots);
    let half = half.min(slots - full);
    (full, half, slots - full - half)
}

/// The stored `shown` value: shown unless it says `false`.
pub fn read_shown(stored: Option<String>) -> bool {
    stored.is_none_or(|v| v.trim() != "false")
}

/// The stored `coords` value: off unless it says `true` (Java Jade shows no
/// coordinates; `/jade coords` or the page turns the lines on).
pub fn read_coords(stored: Option<String>) -> bool {
    stored.is_some_and(|v| v.trim() == "true")
}

/// The stored detail-line value: on unless it says `false` — details opt
/// out, where the master `coords` opts in.
pub fn read_detail(stored: Option<String>) -> bool {
    stored.is_none_or(|v| v.trim() != "false")
}

/// The stored position: the position's index, `top` first in the table.
pub fn read_position(stored: Option<String>) -> usize {
    let Some(v) = stored else {
        return DEFAULT_POSITION;
    };
    POSITIONS
        .iter()
        .position(|c| *c == v.trim())
        .unwrap_or(DEFAULT_POSITION)
}

pub fn shown_label(shown: bool) -> String {
    format!("Shown: {}", if shown { "ON" } else { "OFF" })
}

pub fn coords_label(coords: bool) -> String {
    format!("Details: {}", if coords { "ON" } else { "OFF" })
}

pub fn position_label(position: usize) -> String {
    format!("Position: {}", POSITION_LABELS[position % POSITIONS.len()])
}

/// The next position in the table, cycling through all five.
pub fn next_position(position: usize) -> usize {
    (position + 1) % POSITIONS.len()
}

/// Jade's own animation (`OverlayRenderer.renderOverlay478757`,
/// `BoxElementImpl.updateRect`/`chase`): the box's rect chases the new
/// layout over `MORPH_MS`, linearly; show and hide are a plain ramp at
/// `ALPHA_SPEED` a tick (~83 ms). No easing, no scaling, no dimming —
/// a target change morphs the rect while the contents stay opaque.
pub const MORPH_MS: f32 = 75.0;
pub const ALPHA_SPEED: f32 = 0.6;

/// `start + progress * (end - start)` per field — `chase`.
pub fn lerp_rect(start: [f32; 4], end: [f32; 4], progress: f32) -> [f32; 4] {
    std::array::from_fn(|i| start[i] + progress * (end[i] - start[i]))
}

/// `OverlayRenderer.animation`: the rect on screen now, morphing toward
/// `expected` from `start` since `start_t`; `lingering` is `lingerTooltip`
/// (the last contents kept for the fade-out), `shown` is `updateRect`'s
/// "width 0 marks dirty".
struct Anim {
    rect: [f32; 4],
    start: [f32; 4],
    start_t: Option<std::time::Instant>,
    expected: [f32; 4],
    lingering: bool,
    shown: bool,
    show_hide_alpha: f32,
    last: std::time::Instant,
}

thread_local! {
    /// Whether the box shows, the details lines show, and where it sits,
    /// each read once from its file, then kept.
    static SHOWN: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    static COORDS: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    static DETAIL: std::cell::RefCell<[Option<bool>; 5]> = const { std::cell::RefCell::new([None; 5]) };
    static POSITION: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
    static ANIM: std::cell::RefCell<Anim> = std::cell::RefCell::new(Anim {
        rect: [0.0; 4],
        start: [0.0; 4],
        start_t: None,
        expected: [0.0; 4],
        lingering: false,
        shown: false,
        show_hide_alpha: 0.0,
        last: std::time::Instant::now(),
    });
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

/// Whether the details lines show, read once through `get`.
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

/// Whether detail line `i` shows, read once through `get` (its file).
pub fn detail_on(i: usize, get: impl FnOnce() -> Option<String>) -> bool {
    DETAIL.with(|c| {
        let found = c.borrow()[i].unwrap_or_else(|| read_detail(get()));
        c.borrow_mut()[i] = Some(found);
        found
    })
}

pub fn set_detail(i: usize, on: bool) {
    DETAIL.with(|c| c.borrow_mut()[i] = Some(on));
}

/// The position's index, read once through `get`.
pub fn position(get: impl FnOnce() -> Option<String>) -> usize {
    POSITION.with(|c| {
        let found = c.get().unwrap_or_else(|| read_position(get()));
        c.set(Some(found));
        found
    })
}

pub fn set_position(position: usize) {
    POSITION.with(|c| c.set(Some(position)));
}

/// The toggle key's id and default binding — `key.rs` listens for it and
/// `list.rs` registers it, so both read it here.
pub const TOGGLE_KEY_ID: &str = "toggle";
pub const TOGGLE_KEY: &str = "key.keyboard.j";

/// A toggleable detail line — the settings-file key and page label —
/// each on by default, all gated by the master details flag (and sneak).
pub struct Detail {
    pub key: &'static str,
    pub label: &'static str,
}

/// Indexes into [`DETAILS`] so `hud.rs` asks by name, not magic number.
pub const COORDINATES: usize = 0;
pub const DISTANCE: usize = 1;
pub const BLOCK_INFO: usize = 2;
pub const MOB_INFO: usize = 3;
pub const TOOL: usize = 4;
pub const DETAILS: [Detail; 5] = [
    Detail { key: "coordinates", label: "Coordinates" },
    Detail { key: "distance", label: "Distance" },
    Detail { key: "block_info", label: "Block info" },
    Detail { key: "mob_info", label: "Entity info" },
    Detail { key: "tool", label: "Harvest tool" },
];

/// The detail line a settings-file key names, if any.
pub fn detail_index(key: &str) -> Option<usize> {
    DETAILS.iter().position(|d| d.key == key)
}

/// A thing the toggle key, `/jade` or a settings-page click asks the mod to
/// do — every caller funnels through [`act`], so the read-flip-keep rule and
/// the file's `"true"`/`"false"` live once.
#[derive(Clone, Copy)]
pub enum Action {
    ToggleShown,
    ToggleDetails,
    /// Toggle one line of [`DETAILS`]; the master flag still gates them all.
    ToggleDetail(usize),
    CyclePosition,
    SetPosition(&'static str),
}

/// The settings page's buttons in click order — `settings.rs` labels them and
/// `click.rs` acts on the same index, so the order exists once.
pub const PAGE: [Action; 8] = [
    Action::ToggleShown,
    Action::ToggleDetails,
    Action::ToggleDetail(COORDINATES),
    Action::ToggleDetail(DISTANCE),
    Action::ToggleDetail(BLOCK_INFO),
    Action::ToggleDetail(MOB_INFO),
    Action::ToggleDetail(TOOL),
    Action::CyclePosition,
];

/// Applies `action` to the kept settings (`get` answers a file's contents for
/// its key the first time each is read) and returns the `(file key, value to
/// store, chat message)` — the caller writes and chats.
pub fn act(action: Action, get: impl Fn(&str) -> Option<String>) -> (&'static str, &'static str, String) {
    match action {
        Action::ToggleShown => {
            let v = !shown(|| get("shown"));
            set_shown(v);
            ("shown", bool_str(v), format!("Jade {}", if v { "shown" } else { "hidden" }))
        }
        Action::ToggleDetails => {
            let v = !coords(|| get("coords"));
            set_coords(v);
            ("coords", bool_str(v), format!("Details {}", if v { "shown" } else { "hidden" }))
        }
        Action::ToggleDetail(i) => {
            let v = !detail_on(i, || get(DETAILS[i].key));
            set_detail(i, v);
            (DETAILS[i].key, bool_str(v), format!("{} {}", DETAILS[i].label, if v { "shown" } else { "hidden" }))
        }
        Action::CyclePosition => act(Action::SetPosition(POSITIONS[next_position(position(|| get("position")))]), get),
        Action::SetPosition(name) => {
            let index = read_position(Some(name.to_owned()));
            set_position(index);
            ("position", POSITIONS[index], format!("Jade moved to {}", POSITION_LABELS[index]))
        }
    }
}

/// The button's text on the settings page.
pub fn action_label(action: Action, get: impl Fn(&str) -> Option<String>) -> String {
    match action {
        Action::ToggleShown => shown_label(shown(|| get("shown"))),
        Action::ToggleDetails => coords_label(coords(|| get("coords"))),
        Action::ToggleDetail(i) => {
            format!("{}: {}", DETAILS[i].label, if detail_on(i, || get(DETAILS[i].key)) { "ON" } else { "OFF" })
        }
        Action::CyclePosition => position_label(position(|| get("position"))),
        Action::SetPosition(_) => String::new(),
    }
}

/// `true`/`"true"`, `false`/`"false"` — the settings file's words.
fn bool_str(v: bool) -> &'static str {
    if v { "true" } else { "false" }
}

/// The rect and alpha to draw this frame, or `None` — `renderOverlay478757`
/// and `updateRect` ported: `expected` is where the box wants to be,
/// `present` whether the crosshair has a target (its last contents draw the
/// fade-out while `lingering` holds).
pub fn animate(expected: [f32; 4], present: bool) -> Option<([f32; 4], f32)> {
    ANIM.with(|cell| {
        let a = &mut *cell.borrow_mut();
        let now = std::time::Instant::now();
        let delta = now.duration_since(a.last).as_secs_f32() * 20.0;
        a.last = now;
        if present {
            a.lingering = true;
        }
        // `disappearingDelay` is 0: nothing holds the box up once lost.
        let show_hide = if a.lingering {
            let dir = if present { ALPHA_SPEED } else { -ALPHA_SPEED };
            a.show_hide_alpha = (a.show_hide_alpha + dir * delta).clamp(0.0, 1.0);
            a.show_hide_alpha
        } else {
            f32::from(present)
        };
        if !a.lingering && !present {
            return None;
        }
        if show_hide < 0.1 {
            a.lingering = false;
            a.shown = false;
            return None;
        }
        if !a.shown {
            // Width 0 marks dirty: the box snaps on, content fades in.
            a.rect = expected;
            a.start = expected;
            a.expected = expected;
            a.start_t = None;
            a.shown = true;
            return Some((a.rect, show_hide));
        }
        if expected != a.expected {
            a.start = a.rect;
            a.start_t = Some(now);
        }
        a.expected = expected;
        let progress = a
            .start_t
            .map(|t| (t.elapsed().as_secs_f32() * 1000.0 / MORPH_MS).min(1.0))
            .unwrap_or(1.0);
        a.rect = lerp_rect(a.start, expected, progress);
        if progress >= 1.0 {
            a.start_t = None;
            a.rect = expected;
        }
        // The box morphs between targets but never dims: the alpha is the
        // show/hide ramp's alone.
        Some((a.rect, show_hide))
    })
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
    fn mod_names() {
        assert_eq!(mod_name("minecraft:stone"), "Minecraft");
        assert_eq!(mod_name("kaisyn:ruby_block"), "Kaisyn");
        assert_eq!(mod_name("pig"), "Minecraft");
    }

    #[test]
    fn lines_read_well() {
        assert_eq!(pos_line([12, 64, -4]), "12, 64, -4");
        assert_eq!(distance_line(4.26), "4.3 m");
        assert_eq!(distance([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]), 5.0);
    }

    #[test]
    fn hearts_split_rows() {
        assert_eq!(heart_row(10.0, 10.0), (5, 0, 0));
        assert_eq!(heart_row(9.0, 10.0), (4, 1, 0));
        assert_eq!(heart_row(3.0, 20.0), (1, 1, 8));
        assert_eq!(heart_row(100.0, 200.0), (10, 0, 0));
        assert_eq!(heart_row(0.0, 10.0), (0, 0, 5));
    }

    #[test]
    fn shown_unless_stored_false() {
        assert!(read_shown(None));
        assert!(read_shown(Some("true".into())));
        assert!(!read_shown(Some("false\n".into())));
        assert_eq!(shown_label(false), "Shown: OFF");
    }

    #[test]
    fn coords_only_when_stored_true() {
        assert!(!read_coords(None));
        assert!(read_coords(Some("true".into())));
        assert!(!read_coords(Some("false".into())));
        assert_eq!(coords_label(false), "Details: OFF");
    }

    #[test]
    fn positions_read_and_cycle() {
        assert_eq!(read_position(None), DEFAULT_POSITION);
        assert_eq!(read_position(Some("top_right".into())), 2);
        assert_eq!(read_position(Some("nonsense".into())), DEFAULT_POSITION);
        assert_eq!(position_label(0), "Position: Top");
        assert_eq!(next_position(4), 0);
        assert_eq!(next_position(0), 1);
    }

    #[test]
    fn the_box_texture_is_dark_and_bordered() {
        let t = box_texture();
        assert_eq!(t.len(), (BOX_TEX * BOX_TEX * 4) as usize);
        let px = |x: u32, y: u32| &t[((y * BOX_TEX + x) * 4) as usize..][..4];
        // The margin is transparent, the shadow's corners cut.
        assert_eq!(px(0, 0), [0, 0, 0, 0]);
        assert_eq!(px(12, 0), [0, 0, 0, 0]);
        assert_eq!(px(2, 2), [0, 0, 0, 0]);
        // The shadow and the middle are the fill.
        assert_eq!(px(12, 2), [FILL.0, FILL.1, FILL.2, FILL.3]);
        assert_eq!(px(12, 12), [FILL.0, FILL.1, FILL.2, FILL.3]);
        // The border's top is brighter than its bottom.
        assert!(px(12, 3)[0] > px(12, 20)[0]);
        assert_eq!(px(12, 3)[3], 255);
    }

    #[test]
    fn providers_write_lines() {
        assert_eq!(crop_line("minecraft:wheat", "5").as_deref(), Some("§771%"));
        assert_eq!(crop_line("minecraft:lever", "3").as_deref(), Some("§7age: 3"));
        assert_eq!(crop_line("minecraft:stone", "x"), None);
        assert_eq!(mob_lines(-1, false, false, false), vec!["§7Baby"]);
        assert_eq!(mob_lines(0, true, true, false), vec!["§7Sheared", "§7Saddled"]);
        assert!(mob_lines(0, false, false, false).is_empty());
    }

    #[test]
    fn actions_flip_and_keep() {
        let get = |_: &str| None;
        // Each PAGE action applies to the kept state and names its file.
        let (name, _, _) = act(Action::ToggleShown, get);
        assert_eq!(name, "shown");
        assert!(!shown(|| None));
        let (_, _, msg) = act(Action::SetPosition("top_left"), get);
        assert_eq!(msg, "Jade moved to Top left");
        let (_, value, _) = act(Action::CyclePosition, get);
        assert_eq!(value, "top_right");
        let (_, _, msg) = act(Action::ToggleDetails, get);
        assert_eq!(msg, "Details shown");
        let (name, value, msg) = act(Action::ToggleDetail(DISTANCE), get);
        assert_eq!((name, value), ("distance", "false"));
        assert_eq!(msg, "Distance hidden");
        // Labels mirror the kept state.
        assert_eq!(action_label(Action::CyclePosition, get), "Position: Top right");
        assert_eq!(action_label(Action::ToggleDetail(DISTANCE), get), "Distance: OFF");
        assert!(PAGE.len() == DETAILS.len() + 3);
    }

    #[test]
    fn harvest_tool_picks_the_weakest_tier() {
        let tags = |ts: &'static [&'static str]| move |t: &str| ts.contains(&t);
        // A log: mineable by axe, no needs-tag, so a wooden axe.
        assert_eq!(
            harvest_tool(2.0, tags(&["minecraft:mineable/axe"])).as_deref(),
            Some("minecraft:wooden_axe")
        );
        // Diamond ore: a pickaxe, at least iron.
        assert_eq!(
            harvest_tool(3.0, tags(&["minecraft:mineable/pickaxe", "minecraft:needs_iron_tool"])).as_deref(),
            Some("minecraft:iron_pickaxe")
        );
        // Bedrock: unbreakable, a barrier.
        assert_eq!(harvest_tool(-1.0, tags(&[])).as_deref(), Some("minecraft:barrier"));
        // Dirt: no mineable tag, bare hands — nothing.
        assert_eq!(harvest_tool(0.5, tags(&[])), None);
    }

    #[test]
    fn morph_maths() {
        // lerp_rect: linear chase.
        assert_eq!(lerp_rect([0.0; 4], [10.0; 4], 0.5), [5.0; 4]);
        assert_eq!(lerp_rect([1.0, 2.0, 3.0, 4.0], [1.0, 2.0, 3.0, 4.0], 0.3), [1.0, 2.0, 3.0, 4.0]);
    }
}
