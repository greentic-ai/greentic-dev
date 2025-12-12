use std::process;

#[cfg(feature = "cli")]
use greentic_component::cmd::inspect;

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("component-inspect requires the `cli` feature");
    process::exit(1);
}

#[cfg(feature = "cli")]
fn main() {
    let args = inspect::parse_from_cli();
    match inspect::run(&args) {
        Ok(result) => {
            inspect::emit_warnings(&result.warnings);
            if args.strict && !result.warnings.is_empty() {
                eprintln!(
                    "component-inspect: {} warning(s) treated as errors (--strict)",
                    result.warnings.len()
                );
                process::exit(2);
            }
        }
        Err(err) => {
            if args.json {
                let failure = serde_json::json!({
                    "error": {"code": err.code(), "message": err.to_string()}
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&failure).expect("serialize failure report")
                );
            } else {
                eprintln!("component-inspect[{}]: {err}", err.code());
            }
            process::exit(1);
        }
    }
}
