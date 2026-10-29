// The mod's settings page: shown, the coordinates line, and the position.
fn jade_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::jade as m;
    if mod_id != m::ID {
        return;
    }
    buttons.push(m::shown_label(m::shown(|| game.native_setting(m::ID, "shown"))));
    buttons.push(m::coords_label(m::coords(|| game.native_setting(m::ID, "coords"))));
    buttons.push(m::corner_label(m::corner(|| game.native_setting(m::ID, "position"))));
}
