// A click on the settings page: flip shown or the coordinates line, cycle the
// position; each keeps its value.
fn jade_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::jade as m;
    if mod_id != m::ID {
        return;
    }
    match button {
        0 => {
            let shown = !m::shown(|| game.native_setting(m::ID, "shown"));
            m::set_shown(shown);
            game.native_set_setting(m::ID, "shown", if shown { "true" } else { "false" });
        }
        1 => {
            let coords = !m::coords(|| game.native_setting(m::ID, "coords"));
            m::set_coords(coords);
            game.native_set_setting(m::ID, "coords", if coords { "true" } else { "false" });
        }
        2 => {
            let corner = m::next_corner(m::corner(|| game.native_setting(m::ID, "position")));
            m::set_corner(corner);
            game.native_set_setting(m::ID, "position", m::CORNERS[corner]);
        }
        _ => {}
    }
}
