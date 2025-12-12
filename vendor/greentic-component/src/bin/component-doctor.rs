use std::process;

#[cfg(feature = "cli")]
use greentic_component::cmd::doctor;

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("component-doctor requires the `cli` feature");
    process::exit(1);
}

#[cfg(feature = "cli")]
fn main() {
    if let Err(err) = doctor::run(doctor::parse_from_cli()) {
        eprintln!("component-doctor[{}]: {err}", err.code());
        process::exit(1);
    }
}
