// The box in the chosen corner: what the crosshair is on — an entity, then a
// block, then a fluid, the order the game picks them in — with its name in
// the player's language, its registry id, where it is and how far away.
fn jade_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::jade as m;
    use horizon_client::extras::Corner;

    if !game.native_on(m::ID) || !m::shown(|| game.native_setting(m::ID, "shown")) {
        return;
    }
    let corner = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ][m::corner(|| game.native_setting(m::ID, "position"))];
    // The debug screen takes the top left: leave the corner to it (F3).
    if !game.native_corner_shown(corner) {
        return;
    }
    // The language lookup: the translation, or nothing when the key is missing.
    let translate =
        |key: &str| horizon_client::lang::has(key).then(|| horizon_client::lang::get(key));

    let mut title = None;
    let mut id = None;
    let mut pos = None;
    let mut dist = None;
    if let Some(Some(entity)) = game.pick_entity_hit() {
        let entity = u32::try_from(entity).ok().and_then(|id| {
            game.mobs.iter().find(|mob| mob.id == id)
        });
        if let Some(mob) = entity {
            title = Some(m::entity_name(&mob.kind.id, &translate));
            id = Some(mob.kind.id.clone());
        } else {
            // A player, a guest or a projectile: named, but not a mob.
            title = Some("Entity".to_owned());
        }
    } else if let Some(hit) = game.pick() {
        let facts = game
            .res
            .facts()
            .get(game.world.block(hit.pos[0], hit.pos[1], hit.pos[2]));
        let name: &str = &facts.name;
        title = Some(m::display_name(name, &translate));
        id = Some(name.to_owned());
        pos = Some(m::pos_line(hit.pos));
        if let Some(p) = game.native_player() {
            dist = Some(m::distance_line(m::distance(hit.point, [p.x, p.y, p.z])));
        }
    } else if let Some(hit) = game.pick_fluid() {
        let facts = game
            .res
            .facts()
            .get(game.world.block(hit.pos[0], hit.pos[1], hit.pos[2]));
        let name: &str = &facts.name;
        title = Some(m::display_name(name, &translate));
        id = Some(name.to_owned());
        pos = Some(m::pos_line(hit.pos));
        if let Some(p) = game.native_player() {
            dist = Some(m::distance_line(m::distance(hit.point, [p.x, p.y, p.z])));
        }
    }
    let Some(title) = title else {
        return;
    };
    let mut lines = vec![title];
    if let Some(id) = id {
        lines.push(id);
    }
    if m::coords(|| game.native_setting(m::ID, "coords")) {
        if let Some(pos) = pos {
            lines.push(pos);
        }
    }
    if let Some(dist) = dist {
        lines.push(dist);
    }

    let w = lines.iter().map(|l| game.native_text_width(l)).fold(0.0, f32::max) + 4.0;
    let h = 10.0 * lines.len() as f32 + 2.0;
    let (x, y) = corners.place(corner, w, h);
    game.native_fill(batch, [x, y, w, h], m::BACKGROUND);
    for (i, line) in lines.iter().enumerate() {
        // The name in white, the details in gray, as Jade writes them.
        let ink = if i == 0 { m::INK } else { 0xAA_AAAA };
        game.native_text(batch, line, [x + 2.0, y + 2.0 + 10.0 * i as f32], ink, true);
    }
}
