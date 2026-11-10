// The box Java Jade draws at the crosshair: a rounded, bordered, translucent
// panel centred at the top, the target's item icon, its name, the mod it came
// from in italic blue, and a row of hearts for a living entity. It pops in
// and settles on every target change and fades out when the crosshair moves
// off — Horizon's HUD primitives (sprite ninths, tinted rects, styled text,
// a scale pass) approximating Jade's own look.
fn jade_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::jade as m;
    use horizon_client::extras::Corner;
    use horizon_client::hud::rgba;

    // Padding around the contents, the line pitch (8px glyphs, a pixel of
    // air), the icon's slot and the gap after it, the `top` position's
    // margin — all in GUI pixels.
    const PAD_X: f32 = 3.0;
    const PAD_Y: f32 = 3.0;
    const PITCH: f32 = 9.0;
    const ICON: f32 = 16.0;
    const GAP: f32 = 3.0;
    const TOP_Y: f32 = 0.0;
    const CORNER_MAP: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];

    /// What one frame's box holds, ready to draw — kept a moment past the
    /// target so the box can fade out instead of snapping off.
    struct Tip {
        /// The icon's item id (`minecraft:pig_spawn_egg`), none to skip it.
        icon: Option<String>,
        /// The text lines, `§`-styled already (name, mod, id, details); the
        /// hearts row goes between the name and the rest.
        lines: Vec<String>,
        /// `(full, half, empty)` heart slots, `None` for blocks and fluids.
        hearts: Option<(u32, u32, u32)>,
        /// The weakest harvest tool's item id at the box's bottom right —
        /// `minecraft:barrier` when the block is unbreakable.
        tool: Option<String>,
        w: f32,
        h: f32,
    }

    thread_local! {
        /// The last box drawn: the fade-out draws it while `animate` ramps.
        static LAST: std::cell::RefCell<Option<Tip>> =
            const { std::cell::RefCell::new(None) };
        /// How many `jade_hud` calls have run: the box texture re-uploads now
        /// and then so an atlas reload (a pack change) gets it back.
        static FRAME: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    }

    /// `text` without its `§x` styling pairs, for measuring.
    fn unstyled(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut code = false;
        for c in text.chars() {
            if code {
                code = false;
            } else if c == '§' {
                code = true;
            } else {
                out.push(c);
            }
        }
        out
    }

    /// The box's top left in GUI pixels: centred for `top`, a corner's stack
    /// for the rest.
    fn place(
        game: &mut crate::game::Game<'_>,
        batch: &mut horizon_client::hud::HudBatch,
        corners: &mut horizon_client::extras_hud::Corners,
        position: usize,
        w: f32,
        h: f32,
    ) -> (f32, f32) {
        match position {
            // Centred at the top, pushed below the boss bars, as
            // `BossBarOverlapMode.PUSH_DOWN`.
            0 => {
                let bars = game
                    .boss_overlay
                    .bars(horizon_client::boss_overlay::now_millis())
                    .len();
                ((batch.width - w) / 2.0, TOP_Y + bars as f32 * 12.0)
            }
            i => corners.place(CORNER_MAP[(i - 1) % 4], w, h),
        }
    }

    /// `src` (`[x, y, w, h]` in the sprite's own pixels) of sprite `name`
    /// into `dst` in GUI pixels, faded by `a` — `native_sprite` with an
    /// alpha, so the intro and outro can fade the box itself.
    fn sprite_a(
        game: &crate::game::Game<'_>,
        batch: &mut horizon_client::hud::HudBatch,
        name: &str,
        src: [f32; 4],
        dst: [f32; 4],
        a: f32,
    ) {
        let Some(mut r) = game.gui.atlas.sprites.get(name).copied() else {
            return;
        };
        let (du, dv) = ((r.u1 - r.u0) / r.w as f32, (r.v1 - r.v0) / r.h as f32);
        r.u0 += src[0] * du;
        r.v0 += src[1] * dv;
        r.u1 = r.u0 + src[2] * du;
        r.v1 = r.v0 + src[3] * dv;
        batch.rect_uv(
            dst[0],
            dst[1],
            dst[2],
            dst[3],
            r,
            rgba(255, 255, 255, (255.0 * a) as u8),
        );
    }

    /// The frame's box: the crosshair's entity first, then its block, then
    /// its fluid — the game's pick order. `None` when it hits nothing.
    fn gather(game: &crate::game::Game<'_>) -> Option<Tip> {
        let translate = |key: &str| {
            horizon_client::lang::has(key).then(|| horizon_client::lang::get(key))
        };
        let details = m::coords(|| game.native_setting(m::ID, "coords"))
            || game.keys.held(horizon_client::actions::Action::Sneak);
        // Each detail line's own toggle; coordinates and distance still
        // need the master details flag (or sneak) too.
        let detail =
            |i: usize| m::detail_on(i, || game.native_setting(m::ID, m::DETAILS[i].key));
        let tooltips = game.settings.file.bool("advancedItemTooltips");
        let item = |id: &str| game.res.items().get(id).map(|_| id.to_owned());

        let mut icon = None;
        let mut lines = Vec::new();
        let mut hearts = None;
        let mut tool = None;
        if let Some(entity) = game.pick_entity_hit() {
            // A mob is its name and hearts; a player, a guest or a
            // projectile is `Entity`, as Jade writes it.
            let mob = entity
                .and_then(|id| u32::try_from(id).ok())
                .and_then(|id| game.mobs.iter().find(|mob| mob.id == id));
            if let Some(mob) = mob {
                icon = item(&format!("{}_spawn_egg", mob.kind.id));
                lines.push(
                    mob.custom_name
                        .clone()
                        .unwrap_or_else(|| m::entity_name(&mob.kind.id, &translate)),
                );
                if detail(m::MOB_INFO) {
                    for line in m::mob_lines(
                        mob.extra.age,
                        mob.extra.sheared,
                        mob.extra.saddled || mob.extra.saddle_stack.is_some(),
                        mob.extra.chested,
                    ) {
                        lines.push(line.to_owned());
                    }
                }
                lines.push(format!("§9§o{}", m::mod_name(&mob.kind.id)));
                if tooltips {
                    lines.push(format!("§8{}", mob.kind.id));
                }
                hearts = Some(m::heart_row(mob.health, mob.attrs.max_health));
            } else {
                lines.push("Entity".to_owned());
            }
        } else {
            let (kind, hit) = game
                .pick()
                .map(|h| ("block", h))
                .or_else(|| game.pick_fluid().map(|h| ("fluid", h)))?;
            let state_id = game.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
            let facts = game.res.facts().get(state_id);
            let name: &str = &facts.name;
            lines.push(m::display_name(name, &translate));
            // The block's properties and harvest notes, as Jade's providers
            // write them under the name.
            if detail(m::BLOCK_INFO) {
                if let Some(state) = game.pack.assets.registry.state(state_id) {
                    let props = &state.properties;
                    if let Some(line) = props.get("age").and_then(|a| m::crop_line(name, a)) {
                        lines.push(line);
                    }
                    if let Some(power) = props.get("power") {
                        if power.as_str() != "0" {
                            lines.push(format!("§7power: {power}"));
                        }
                    }
                    if props.get("lit").is_some_and(|v| v.as_str() == "true") {
                        lines.push("§7lit".into());
                    }
                    if let Some(honey) = props.get("honey_level") {
                        if honey.as_str() != "0" {
                            lines.push(format!("§7honey: {honey}"));
                        }
                    }
                }
            }
            if detail(m::TOOL) {
                tool = m::harvest_tool(facts.hardness, |tag| {
                    game.res.items().block_in_tag(tag, name)
                });
            }
            if tool.is_none() && facts.requires_tool {
                lines.push("§7Requires tool".into());
            }
            lines.push(format!("§9§o{}", m::mod_name(name)));
            if tooltips {
                lines.push(format!("§8{name}"));
            }
            if details && detail(m::COORDINATES) {
                lines.push(format!("§7{}", m::pos_line(hit.pos)));
            }
            if details && detail(m::DISTANCE) {
                if let Some(p) = game.native_player() {
                    let d = m::distance(hit.point, [p.x, p.y, p.z]);
                    lines.push(format!("§7{}", m::distance_line(d)));
                }
            }
            icon = item(name).or_else(|| match name {
                "minecraft:water" => item("minecraft:water_bucket"),
                "minecraft:lava" => item("minecraft:lava_bucket"),
                _ => None,
            });
        }

        // Layout: the icon's slot on the left, the text's widest line or the
        // hearts' row on the right; the icon is as tall as two lines.
        let heart_w = hearts.map_or(0.0, |(f, h, e)| (f + h + e) as f32 * 8.0 - 1.0);
        let text_w = lines
            .iter()
            .map(|l| game.native_text_width(&unstyled(l)))
            .fold(heart_w, f32::max);
        let icon_w = if icon.is_some() { ICON + GAP } else { 0.0 };
        // The harvest tool sits at the top right: make room past the text.
        let tool_w = if tool.is_some() { ICON + GAP } else { 0.0 };
        let text_h =
            lines.len() as f32 * PITCH + f32::from(hearts.is_some()) * PITCH - 1.0;
        let h = PAD_Y * 2.0 + text_h.max(ICON);
        let w = PAD_X * 2.0 + icon_w + text_w + tool_w;
        Some(Tip { icon, lines, hearts, tool, w, h })
    }

    /// The box and its contents at `rect` (the animated chase target), alpha
    /// `a` — the contents anchor at the box's top left while it morphs.
    fn draw(
        game: &mut crate::game::Game<'_>,
        batch: &mut horizon_client::hud::HudBatch,
        tip: &Tip,
        rect: [f32; 4],
        a: f32,
    ) {
        let [x, y, w, h] = rect;
        let s = m::BOX_SLICE;
        let start = batch.vertices.len();

        // The baked texture, as nine slices, its transparent margin out over
        // the contents' padding: the border stays a pixel wide, the shadow's
        // corners keep their cut, the middle stretches.
        const T: &str = "horizon/modimage";
        let (bx, by, bw, bh) = (x - 2.0, y - 2.0, w + 4.0, h + 4.0);
        // The source tile's inner edge: `s`-px slices around a `m`-px middle.
        let e = m::BOX_TEX as f32 - s;
        let m2 = e - s;
        for (src, dst) in [
            ([0.0, 0.0, s, s], [bx, by, s, s]),
            ([s, 0.0, m2, s], [bx + s, by, bw - 2.0 * s, s]),
            ([e, 0.0, s, s], [bx + bw - s, by, s, s]),
            ([0.0, s, s, m2], [bx, by + s, s, bh - 2.0 * s]),
            ([s, s, m2, m2], [bx + s, by + s, bw - 2.0 * s, bh - 2.0 * s]),
            ([e, s, s, m2], [bx + bw - s, by + s, s, bh - 2.0 * s]),
            ([0.0, e, s, s], [bx, by + bh - s, s, s]),
            ([s, e, m2, s], [bx + s, by + bh - s, bw - 2.0 * s, s]),
            ([e, e, s, s], [bx + bw - s, by + bh - s, s, s]),
        ] {
            sprite_a(game, batch, T, src, dst, a);
        }

        // The icon, vertically centred in its slot.
        let mut tx = x + PAD_X;
        if let Some(icon) = &tip.icon {
            game.native_item(batch, icon, [tx, y + (h - ICON) / 2.0], ICON);
            tx += ICON + GAP;
        }
        // The name, the hearts under it for a mob, then the rest — the mod's
        // name stays the last line, as Jade writes it.
        let mut ty = y + PAD_Y;
        let ink = rgba(255, 255, 255, (255.0 * a) as u8);
        let mut lines = tip.lines.iter();
        if let Some(name) = lines.next() {
            batch.text(&game.gui.atlas, name, tx, ty, ink, true);
            ty += PITCH;
        }
        if let Some((full, half, empty)) = tip.hearts {
            let mut hx = tx;
            for (name, n) in [
                ("gui/sprites/hud/heart/full", full),
                ("gui/sprites/hud/heart/half", half),
                ("gui/sprites/hud/heart/container", empty),
            ] {
                for _ in 0..n {
                    sprite_a(game, batch, name, [0.0, 0.0, 9.0, 9.0], [hx, ty, 9.0, 9.0], a);
                    hx += 8.0;
                }
            }
            ty += PITCH;
        }
        for line in lines {
            batch.text(&game.gui.atlas, line, tx, ty, ink, true);
            ty += PITCH;
        }
        // The weakest harvest tool at the box's top right, as Jade's
        // harvest-tool line draws it.
        if let Some(tool) = &tip.tool {
            game.native_item(batch, tool, [x + w - PAD_X - ICON, y + PAD_Y], ICON);
        }
        // `autoScaleThreshold`: a box taller than half the screen shrinks
        // toward it about its centre (but never below half size).
        let threshold = batch.height * 0.5;
        if h > threshold {
            let s = (threshold / h).max(0.5);
            batch.scale_since(start, x + w / 2.0, y + h / 2.0, s, s);
        }
    }

    FRAME.with(|f| {
        // The mod image slot holds one image; refresh it every 300 frames.
        if f.get() % 300 == 0 {
            game.native_mod_image(m::BOX_TEX, m::BOX_TEX, &m::box_texture());
        }
        f.set(f.get() + 1);
    });
    if !game.native_on(m::ID) || !m::shown(|| game.native_setting(m::ID, "shown")) {
        return;
    }
    let position = m::position(|| game.native_setting(m::ID, "position"));
    // The debug screen takes the top left: a corner placement yields to it.
    if position > 0 && !game.native_corner_shown(CORNER_MAP[(position - 1) % 4]) {
        return;
    }
    match gather(game) {
        Some(tip) => {
            let (x, y) = place(game, batch, corners, position, tip.w, tip.h);
            if let Some((rect, alpha)) = m::animate([x, y, tip.w, tip.h], true) {
                draw(game, batch, &tip, rect, alpha);
            }
            LAST.with(|l| *l.borrow_mut() = Some(tip));
        }
        None => LAST.with(|l| {
            let l = l.borrow();
            let Some(tip) = &*l else { return };
            let (x, y) = place(game, batch, corners, position, tip.w, tip.h);
            if let Some((rect, alpha)) = m::animate([x, y, tip.w, tip.h], false) {
                draw(game, batch, tip, rect, alpha);
            }
        }),
    }
}
