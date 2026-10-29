// The toggle key (rebindable on the mod's page, J by default): pressing it
// hides or shows the box.
fn jade_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::jade as m;
    if !game.native_on(m::ID) || !pressed || !game.native_playing() {
        return;
    }
    if key != game.native_mod_key(m::ID, "toggle", "key.keyboard.j") {
        return;
    }
    let shown = !m::shown(|| game.native_setting(m::ID, "shown"));
    m::set_shown(shown);
    game.native_set_setting(m::ID, "shown", if shown { "true" } else { "false" });
    game.native_chat(if shown { "Jade shown" } else { "Jade hidden" });
}
