//! WebView frontend から呼べる command の登録。
//!
//! Rust 側から見ると、ここが TypeScript UI との約束事になる。command 名や payload
//! の形を変えるときは `src-gui` 側の `invoke(...)` / subscription と一緒に変更する。

use std::rc::Rc;
use std::sync::Arc;

use serde_json::json;
use wrac_clap_adapter::HostParameterEditNotifier;
use wxp::{Channel, WxpCommandHandler};

use crate::gui::{GuiStateNotifier, gain_payload};
use crate::plugin::PARAM_GAIN_ID;
use crate::state::SharedState;

/// WebView (フロントエンド) から呼べる command を登録する。
///
/// フロントエンド側 (`src-gui` の TypeScript) は `invoke("set_gain", { value })`
/// のような形でこれらの command を呼び出す。
pub(crate) fn register_commands(
    command_handler: Rc<WxpCommandHandler>,
    shared: Arc<SharedState>,
    gui_notifier: Arc<GuiStateNotifier>,
    host_parameter_edit_notifier: Arc<dyn HostParameterEditNotifier>,
) {
    // 現在の gain 値を取得 (GUI 起動直後の初期表示などに使う)。
    {
        let shared = shared.clone();
        command_handler.register_sync("get_gain_state", move |_ctx| {
            Ok::<_, String>(gain_payload(shared.gain()))
        });
    }

    // slider に触れ始めたタイミング。host に「これから undo 単位」と伝える。
    {
        let host_parameter_edit_notifier = host_parameter_edit_notifier.clone();
        command_handler.register_sync("begin_parameter_gesture", move |_ctx| {
            host_parameter_edit_notifier.begin_edit(PARAM_GAIN_ID);
            Ok::<_, String>(json!({ "ok": true }))
        });
    }

    // slider が動いたタイミング。値を反映して host にも通知する。
    {
        let shared = shared.clone();
        let gui_notifier = gui_notifier.clone();
        let host_parameter_edit_notifier = host_parameter_edit_notifier.clone();
        command_handler.register_sync("set_gain", move |ctx| {
            let value = ctx.arg::<f64>("value").map_err(|e| e.to_string())?;
            let applied = shared.set_gain(value);
            gui_notifier.notify_gain(applied);
            host_parameter_edit_notifier.update_edit(PARAM_GAIN_ID, applied as f64);
            Ok::<_, String>(gain_payload(applied))
        });
    }

    // slider から指を離したタイミング。undo 単位の終了を host に伝える。
    {
        let host_parameter_edit_notifier = host_parameter_edit_notifier.clone();
        command_handler.register_sync("end_parameter_gesture", move |_ctx| {
            host_parameter_edit_notifier.end_edit(PARAM_GAIN_ID);
            Ok::<_, String>(json!({ "ok": true }))
        });
    }

    // gain の変化を継続的に受け取るための subscription を開始する。
    // 引数の `channel` は JS 側で作った callback channel で、これに対して plugin が
    // 値の変化を push してくる仕組み。
    {
        let shared = shared.clone();
        let gui_notifier = gui_notifier.clone();
        command_handler.register_sync("subscribe_gain", move |ctx| {
            let channel = ctx.arg::<Channel>("channel").map_err(|e| e.to_string())?;
            // 登録直後に現在値を 1 度送って初期同期する。
            channel
                .send(gain_payload(shared.gain()))
                .map_err(|e| e.to_string())?;
            gui_notifier.set_channel(channel);
            Ok::<_, String>(json!({ "ok": true }))
        });
    }

    // subscription を解除する。
    {
        let gui_notifier = gui_notifier.clone();
        command_handler.register_sync("unsubscribe_gain", move |_ctx| {
            gui_notifier.clear_channel();
            Ok::<_, String>(json!({ "ok": true }))
        });
    }
}
