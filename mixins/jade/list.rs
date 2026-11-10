// The mod's row in Options > Mods, with its rebindable toggle key.
fn jade_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::jade as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: vec![[m::TOGGLE_KEY_ID.into(), "Toggle Jade".into(), m::TOGGLE_KEY.into()]],
    });
}
