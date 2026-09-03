# bevy_mod_scripting_luars

A [Lua 5.5](https://www.lua.org/manual/5.5/) backend for
[bevy_mod_scripting](https://github.com/makspll/bevy_mod_scripting), using
[luars](https://github.com/CppCXY/lua-rs) that supports WASM instead of
[mlua](https://github.com/mlua-rs/mlua), which does not support WASM yet.


> [!CAUTION]
> Early days. Published BMS 0.20 tracks Bevy 0.18. This crate needs the
> `feat/bevy-0.19` branch, so keep a `[patch.crates-io]` until BMS publishes
> Bevy 0.19. crates.io does not accept git dependencies in `[dependencies]`.

## Usage

Leave BMS's bundled Lua (mlua) off. Add this plugin instead. 

``` toml
[dependencies]
bevy = "0.19"
bevy_mod_scripting = { version = "0.21", default-features = false, features = ["core_functions"] }
bevy_mod_scripting_luars = { git = "https://github.com/shanecelis/bevy_mod_scripting_luars" }

```

``` rust,ignore
use bevy::prelude::*;
use bevy_mod_scripting::BMSPlugin;
use bevy_mod_scripting_luars::LuarsScriptingPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((BMSPlugin, LuarsScriptingPlugin::default()))
        .run();
}
```

Do not enable BMS `lua` or `lua54` features alongside this crate; that would
pull in two Lua VMs.

Assets with `.lua` and `.luau` suffixes load as Lua 5.5. `luars` is enabled with
`unsafe-send` so the VM is `Send` for Bevy.

## Example

Type Lua into a Bevy window and see the result:

``` sh
cargo run --example eval
```

## Wasm

On `wasm32` the VM opens a subset of the standard library: basic, math, string,
table, utf8, coroutine, package, and debug. Not `io` or `os`.

`getrandom` is enabled with `wasm_js`. Give the wasm binary a large enough
stack; [Nano-9](https://github.com/shanecelis/nano9) uses 16MB.

## Provenance

This project was extracted from the
[Nano-9](https://github.com/shanecelis/nano9) code base.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
