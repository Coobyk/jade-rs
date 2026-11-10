// /jade hides or shows the box; /jade coords the details lines; /jade <a
// detail's key> toggles just that line; /jade top (or a corner) moves it;
// /jade help lists this.
fn jade_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    use crate::game::jade as m;
    let Some(args) = command.strip_prefix("jade") else {
        return;
    };
    if !game.native_on(m::ID) || !(args.is_empty() || args.starts_with(' ')) {
        return;
    }
    *ran = true;
    let args = args.trim();
    let action = match args {
        "" => Some(m::Action::ToggleShown),
        "coords" => Some(m::Action::ToggleDetails),
        _ => m::POSITIONS
            .iter()
            .find(|p| **p == args)
            .map(|p| m::Action::SetPosition(*p))
            .or_else(|| m::detail_index(args).map(m::Action::ToggleDetail)),
    };
    match (action, args) {
        (Some(action), _) => {
            let (name, value, message) = m::act(action, |k| game.native_setting(m::ID, k));
            game.native_set_setting(m::ID, name, value);
            game.native_chat(&message);
        }
        (None, "help") => {
            game.native_chat("Jade: /jade toggles the box, /jade coords its details lines,");
            game.native_chat(&format!("/jade {} toggles one,", m::DETAILS.iter().map(|d| d.key).collect::<Vec<_>>().join("|")));
            game.native_chat(&format!("/jade {} moves it,", m::POSITIONS.join("|")));
            game.native_chat("and Options > Mods > Jade rebinds the toggle key (J).");
        }
        (None, other) => game.native_chat(&format!("Unknown option {other}: use /jade help")),
    }
}
