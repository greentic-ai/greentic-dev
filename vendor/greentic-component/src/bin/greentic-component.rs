#![cfg_attr(not(feature = "cli"), allow(dead_code))]

use std::process;

#[cfg(feature = "cli")]
use greentic_component::scaffold::validate::ValidationError;

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("greentic-component CLI requires the `cli` feature");
    process::exit(1);
}

#[cfg(feature = "cli")]
fn main() {
    if let Err(err) = greentic_component::cli::main() {
        match err.downcast::<ValidationError>() {
            Ok(diag) => {
                eprintln!("{:?}", miette::Report::new(diag));
            }
            Err(other) => {
                eprintln!("greentic-component: {other:?}");
            }
        }
        process::exit(1);
    }
}
