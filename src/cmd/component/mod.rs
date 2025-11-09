mod doctor;
mod new;
mod templates;
mod types;
mod validate;

pub use doctor::run_doctor;
pub use new::run_new;
pub use templates::run_templates;
pub use validate::run_validate;

pub(super) use types::{ComponentNewResponse, ComponentTemplatesResponse};

pub(super) const TOOL_NAME: &str = "greentic-component";
const HUMAN_HINT: &str = "Try: greentic-dev component doctor";

pub(super) fn emit_json_error(command: &str, code: &str, message: &str) -> anyhow::Result<()> {
    let wrapper = serde_json::json!({
        "tool": TOOL_NAME,
        "command": command,
        "ok": false,
        "error": {
            "code": code,
            "message": message,
            "hint": HUMAN_HINT,
        }
    });
    println!("{}", serde_json::to_string(&wrapper)?);
    Ok(())
}

pub(super) fn emit_human_hint(context: &str, error: &anyhow::Error) {
    eprintln!("{context}: {error}");
    eprintln!("{HUMAN_HINT}");
}
