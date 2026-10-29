// /jade hides or shows the box; /jade coords the coordinates line;
// /jade top_left (and the other corners) moves it; /jade help lists this.
fn jade_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    use crate::game::jade as m;
    let Some(args) = command.strip_prefix("jade") else {
        return;
    };
    if !game.native_on(m::ID) || !(args.is_empty() || args.starts_with(' ')) {
        return;
    }
    *ran = true;
    match args.trim() {
        "" => {
            let shown = !m::shown(|| game.native_setting(m::ID, "shown"));
            m::set_shown(shown);
            game.native_set_setting(m::ID, "shown", if shown { "true" } else { "false" });
            game.native_chat(if shown { "Jade shown" } else { "Jade hidden" });
        }
        "coords" => {
            let coords = !m::coords(|| game.native_setting(m::ID, "coords"));
            m::set_coords(coords);
            game.native_set_setting(m::ID, "coords", if coords { "true" } else { "false" });
            game.native_chat(if coords { "Coordinates line shown" } else { "Coordinates line hidden" });
        }
        corner if m::CORNERS.contains(&corner) => {
            let index = m::CORNERS.iter().position(|c| *c == corner).unwrap_or(m::DEFAULT_CORNER);
            m::set_corner(index);
            game.native_set_setting(m::ID, "position", corner);
            game.native_chat(&format!("Jade moved to {}", m::CORNER_LABELS[index]));
        }
        "help" => {
            game.native_chat("Jade: /jade toggles the box, /jade coords its coordinates line,");
            game.native_chat("/jade top_left|top_right|bottom_left|bottom_right moves it,");
            game.native_chat("and Options > Mods > Jade rebinds the toggle key (J).");
        }
        other => game.native_chat(&format!("Unknown option {other}: use /jade help")),
    }
}
