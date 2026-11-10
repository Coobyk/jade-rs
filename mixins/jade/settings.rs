// The mod's settings page: shown, the details lines, and the position.
fn jade_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::jade as m;
    if mod_id != m::ID {
        return;
    }
    for action in m::PAGE {
        buttons.push(m::action_label(action, |k| game.native_setting(m::ID, k)));
    }
}
