//! luars 0.26.2 with `unsafe-send` and host callbacks.

use luars::{Lua, LuaApi, SafeOption, Stdlib};

fn new_lua() -> Lua {
    let mut lua = Lua::new(SafeOption::default());
    #[cfg(target_arch = "wasm32")]
    lua.open_stdlibs(&[
        Stdlib::Basic,
        Stdlib::Math,
        Stdlib::String,
        Stdlib::Table,
        Stdlib::Utf8,
        Stdlib::Coroutine,
        Stdlib::Package,
        Stdlib::Debug,
    ])
    .expect("open wasm stdlibs");
    #[cfg(not(target_arch = "wasm32"))]
    lua.open_stdlib(Stdlib::All).expect("open stdlibs");
    lua
}

#[test]
fn lua_is_send_with_unsafe_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Lua>();
}

#[test]
fn host_callback_and_eval() {
    let mut lua = new_lua();
    lua.register_function("host_add", |a: i64, b: i64| a + b)
        .expect("register host_add");
    let sum: i64 = lua
        .load("return host_add(40, 2)")
        .eval()
        .expect("call host_add");
    assert_eq!(sum, 42);
}

#[test]
fn stdlib_string_format() {
    let mut lua = new_lua();
    let s: String = lua
        .load(r#"return string.format("%d", 42)"#)
        .eval()
        .expect("string.format");
    assert_eq!(s, "42");
}
