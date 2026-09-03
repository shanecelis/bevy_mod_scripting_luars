//! Type Lua expressions, press Enter, see the result on screen.
//!
//! Native:
//! ```sh
//! cargo run --example eval --features eval
//! ```
//!
//! Browser (Trunk):
//! ```sh
//! trunk serve --config web/Trunk.toml
//! ```
//!
//! Try: `1 + 2`, `string.format("%x", 255)`, `x = 10` then `x * 2`.

use bevy::{
    asset::AssetMetaCheck,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};
use bevy_mod_scripting::{
    BMSPlugin,
    asset::ScriptAsset,
    bindings::script_value::ScriptValue,
    core::script::{ScriptComponent, ScriptContexts},
};
use bevy_mod_scripting_luars::{
    LuarsScriptingPlugin, MultiLuaScriptValue, into_bms_error,
    luars::{LuaApi, LuaError},
};

const HISTORY_CAP: usize = 24;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevy_mod_scripting_luars eval".into(),
                        resolution: (720, 480).into(),
                        #[cfg(target_arch = "wasm32")]
                        canvas: Some("#bevy-canvas".into()),
                        #[cfg(target_arch = "wasm32")]
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // Trunk does not serve Bevy `.meta` sidecars.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins((BMSPlugin, LuarsScriptingPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_keyboard, refresh_ui).chain())
        .run();
}

#[derive(Resource)]
struct Repl {
    input: String,
    history: Vec<String>,
    ready: bool,
}

#[derive(Component)]
struct HistoryText;

#[derive(Component)]
struct PromptText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let handle = asset_server.load::<ScriptAsset>("eval.lua");
    commands.spawn(ScriptComponent::new([handle]));

    commands.insert_resource(Repl {
        input: String::new(),
        history: vec![
            "Lua 5.5 via luars. Type an expression and press Enter.".into(),
            "Examples:  1+2   string.upper(\"hi\")   x=3 then x*x".into(),
        ],
        ready: false,
    });

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.09, 0.11)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.82, 0.78)),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                HistoryText,
            ));
            parent.spawn((
                Text::new("> "),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.7)),
                PromptText,
            ));
        });
}

fn handle_keyboard(
    mut input: MessageReader<KeyboardInput>,
    mut repl: ResMut<Repl>,
    contexts: Res<ScriptContexts<LuarsScriptingPlugin>>,
) {
    // Context appears after the script asset finishes loading.
    if !repl.ready {
        let inner = contexts.read();
        if inner
            .first_resident_from_each_context()
            .any(|(_, ctx)| ctx.as_loaded().is_some())
        {
            repl.ready = true;
            push_history(&mut repl, "ready.".into());
        }
    }

    for ev in input.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }
        match &ev.logical_key {
            Key::Enter => {
                let line = repl.input.trim().to_owned();
                repl.input.clear();
                if line.is_empty() {
                    continue;
                }
                push_history(&mut repl, format!("> {line}"));
                let result = eval_line(&contexts, &line);
                push_history(&mut repl, result);
            }
            Key::Backspace => {
                repl.input.pop();
            }
            Key::Space => {
                repl.input.push(' ');
            }
            Key::Character(c) => {
                if !c.chars().any(char::is_control) {
                    repl.input.push_str(c);
                }
            }
            _ => {}
        }
    }
}

fn eval_line(contexts: &ScriptContexts<LuarsScriptingPlugin>, line: &str) -> String {
    let inner = contexts.read();
    let Some((_attachment, ctx)) = inner.first_resident_from_each_context().next() else {
        return "error: no Lua context yet (script still loading)".into();
    };
    let Some(arc) = ctx.as_loaded() else {
        return "error: Lua context not ready".into();
    };
    let mut lua = arc.lock();

    // Prefer expression form so `1+2` prints a value.
    match eval_as_expression(&mut lua, line) {
        Ok(text) => text,
        Err(expr_err) => match eval_as_statement(&mut lua, line) {
            Ok(text) => text,
            Err(_stmt_err) => format!("error: {expr_err}"),
        },
    }
}

fn eval_as_expression(
    lua: &mut bevy_mod_scripting_luars::LuarsContext,
    line: &str,
) -> Result<String, String> {
    let source = format!("return {line}");
    let multi: MultiLuaScriptValue = lua
        .lua
        .load(&source)
        .set_name("=eval")
        .eval_multi()
        .map_err(|e| format_lua_error(&mut lua.lua, e))?;
    Ok(format_multi(multi))
}

fn eval_as_statement(
    lua: &mut bevy_mod_scripting_luars::LuarsContext,
    line: &str,
) -> Result<String, String> {
    lua.lua
        .load(line)
        .set_name("=eval")
        .exec()
        .map_err(|e| format_lua_error(&mut lua.lua, e))?;
    Ok("ok".into())
}

fn format_lua_error(lua: &mut bevy_mod_scripting_luars::luars::Lua, e: LuaError) -> String {
    into_bms_error(lua, e).to_string()
}

fn format_multi(multi: MultiLuaScriptValue) -> String {
    if multi.0.is_empty() {
        return "nil".into();
    }
    multi
        .0
        .iter()
        .map(format_value)
        .collect::<Vec<_>>()
        .join("\t")
}

fn format_value(v: &ScriptValue) -> String {
    match v {
        ScriptValue::Unit => "nil".into(),
        ScriptValue::Bool(b) => b.to_string(),
        ScriptValue::Integer(i) => i.to_string(),
        ScriptValue::Float(f) => f.to_string(),
        ScriptValue::String(s) => format!("{s:?}"),
        ScriptValue::List(items) => {
            let inner = items.iter().map(format_value).collect::<Vec<_>>().join(", ");
            format!("[{inner}]")
        }
        ScriptValue::Tuple(t) => {
            let inner = t.0.iter().map(format_value).collect::<Vec<_>>().join(", ");
            format!("({inner})")
        }
        ScriptValue::Map(map) => {
            let mut parts: Vec<_> = map
                .iter()
                .map(|(k, v)| format!("{k}={}", format_value(v)))
                .collect();
            parts.sort();
            format!("{{{}}}", parts.join(", "))
        }
        ScriptValue::Reference(_) => "<reference>".into(),
        ScriptValue::Function(_) | ScriptValue::FunctionMut(_) => "<function>".into(),
        ScriptValue::Error(e) => format!("error: {e}"),
    }
}

fn push_history(repl: &mut Repl, line: String) {
    repl.history.push(line);
    if repl.history.len() > HISTORY_CAP {
        let drop_n = repl.history.len() - HISTORY_CAP;
        repl.history.drain(0..drop_n);
    }
}

fn refresh_ui(
    repl: Res<Repl>,
    mut history: Query<&mut Text, (With<HistoryText>, Without<PromptText>)>,
    mut prompt: Query<&mut Text, (With<PromptText>, Without<HistoryText>)>,
) {
    if !repl.is_changed() {
        return;
    }
    if let Ok(mut text) = history.single_mut() {
        *text = Text::new(repl.history.join("\n"));
    }
    if let Ok(mut text) = prompt.single_mut() {
        let suffix = if repl.ready { "" } else { "  (loading…)" };
        *text = Text::new(format!("> {}{}", repl.input, suffix));
    }
}
