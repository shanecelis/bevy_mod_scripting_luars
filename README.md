# bevy_mod_scripting_luars

A [Lua 5.5](https://www.lua.org/manual/5.5/) backend for
[bevy_mod_scripting](https://github.com/makspll/bevy_mod_scripting), using
[luars](https://github.com/CppCXY/lua-rs) that supports WASM instead of
[mlua](https://github.com/mlua-rs/mlua), which does not support WASM yet.

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

## Example: eval

A small Bevy window that acts as a Lua REPL. Type an expression, press Enter,
and the result shows above the prompt. Globals persist for the session (assign
`x = 3`, then evaluate `x * x`).

### Native

``` sh
cargo run --example eval --features eval
```

### WASM

Run the eval example in the browser (needs trunk and the
`wasm32-unknown-unknown` target).

``` sh
trunk serve --config web/Trunk.toml
```

Then open http://localhost:8080/. Click the canvas so it receives keyboard
focus.

Try:

``` lua
1 + 2
string.upper("hi")
x = 10
x * x
```

## Wasm

On `wasm32` the VM opens a subset of the standard library: basic, math, string,
table, utf8, coroutine, package, and debug. Not `io` or `os`.

`getrandom` uses the `wasm_js` backend (see `.cargo/config.toml`). The eval
binary is linked with a 16MB stack. Trunk copies `assets/` next to the wasm so
`eval.lua` loads in the browser.

## Provenance

This project was extracted from the
[Nano-9](https://github.com/shanecelis/nano9) code base.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
