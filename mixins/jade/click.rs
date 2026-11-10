// A click on the settings page: act on the button's action (PAGE's order);
// each action keeps its value.
fn jade_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::jade as m;
    if mod_id != m::ID {
        return;
    }
    let Some(action) = m::PAGE.get(button) else { return };
    let (name, value, _) = m::act(*action, |k| game.native_setting(m::ID, k));
    game.native_set_setting(m::ID, name, value);
}
