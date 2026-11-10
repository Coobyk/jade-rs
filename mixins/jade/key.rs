// The toggle key (rebindable on the mod's page, J by default): pressing it
// hides or shows the box.
fn jade_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::jade as m;
    if !game.native_on(m::ID) || !pressed || !game.native_playing() {
        return;
    }
    if key != game.native_mod_key(m::ID, m::TOGGLE_KEY_ID, m::TOGGLE_KEY) {
        return;
    }
    let (name, value, message) = m::act(m::Action::ToggleShown, |k| game.native_setting(m::ID, k));
    game.native_set_setting(m::ID, name, value);
    game.native_chat(&message);
}
