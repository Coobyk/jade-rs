# Jade for Horizon

The block, fluid or entity under your crosshair, in a corner of the screen —
a rebuild of [Jade](https://modrinth.com/mod/jade) as a
[Horizon](https://playhorizon.dev) native mod (`.hnmod`).

Jade reads the game's crosshair picking (`Game::pick`, `pick_fluid`,
`pick_entity_hit`) and its block facts, which are not native hook points, so
unlike hook-point-only mods it pins the Horizon commit it was built on. The
launcher builds it on that Nightly.

## What it shows

A translucent box in a corner (top right by default) with, in order:

- The target's name in your language (`block.<namespace>.<path>` /
  `entity.minecraft.<path>` in the language file, else the path in words)
- Its registry id (`minecraft:iron_ore`)
- Its coordinates (`12, 64, -4`), on by default
- Its distance from you (`4.3 m`)

The crosshair's order is the game's: an entity first, then a block, then a
fluid. Players, guests and projectiles show as `Entity`.

## Controls

- **Toggle key** — J by default, rebindable in Options > Mods > Jade
- `/jade` — hide or show the box
- `/jade coords` — hide or show the coordinates line
- `/jade top_left` / `top_right` / `bottom_left` / `bottom_right` — move the box
- `/jade help` — the list above

The settings page (Options > Mods > Jade) has the shown, coordinates and
position toggles. The box yields its corner to the debug screen (F3).

## Repository layout

```
├── Cargo.toml                 # cargo test: the runtime logic's unit tests
├── compatibility.json         # pinned Horizon + horizon-mixins revisions
└── mixins/jade/
    ├── horizon-mixin.toml     # the bundle manifest: module + 6 injections
    ├── runtime.rs             # the mod's logic, free of game types
    ├── hud.rs                 # native_hud: pick the target, draw the box
    ├── key.rs                 # native_key: the toggle key
    ├── command.rs             # native_command: /jade
    ├── list.rs                # native_mods: the mod's row and its key
    ├── settings.rs            # native_settings: the settings page
    └── click.rs               # native_settings_click: the page's buttons
```

## Building

Tools: Rust, Python 3.11+, rsync; [horizon-mods](https://github.com/horizon-rust/horizon-mods)
checked out beside this (the guide is at [playhorizon.dev/docs/make-a-mod](https://playhorizon.dev/docs/make-a-mod)).

```sh
cargo test                                           # the runtime logic's tests
python3 /path/to/horizon-mods/native/tools/native.py check .   # apply + cargo check against the pinned Horizon
horizon-launcher native-build mixins/jade --trust-native-code  # build a playable game
```

## Agent control

The Horizon launcher ships an MCP server (`horizon-launcher mcp`, stdio) for
driving the launcher and the game — see
[playhorizon.dev/docs/agents](https://playhorizon.dev/docs/agents). For ZCode,
add it to `~/.zcode/cli/config.json`:

```json
{ "mcp": { "servers": { "horizon": {
    "command": "/usr/lib/horizon-launcher/horizon-launcher", "args": ["mcp"] } } } }
```

## License

MIT, like Jade.
