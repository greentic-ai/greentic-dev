use anyhow::{Result, anyhow};
use wit_component::{DecodedWasm, metadata};
use wit_parser::{Resolve, WorldId};

/// Indicates how a world was decoded from a wasm binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldSource {
    /// Metadata was decoded from a core wasm module.
    Metadata,
    /// WIT was reconstructed from a fully-fledged component.
    Component,
}

/// Resolved world information extracted from a wasm binary.
pub struct DecodedWorld {
    pub resolve: Resolve,
    pub world: WorldId,
    pub source: WorldSource,
}

/// Decode a wasm module or component into its WIT world description.
pub fn decode_world(bytes: &[u8]) -> Result<DecodedWorld> {
    match metadata::decode(bytes) {
        Ok((_maybe_module, bindgen)) => Ok(DecodedWorld {
            resolve: bindgen.resolve,
            world: bindgen.world,
            source: WorldSource::Metadata,
        }),
        Err(module_err) => match wit_component::decode(bytes) {
            Ok(DecodedWasm::Component(resolve, world)) => Ok(DecodedWorld {
                resolve,
                world,
                source: WorldSource::Component,
            }),
            Ok(DecodedWasm::WitPackage(_, _)) => Err(module_err),
            Err(component_err) => Err(anyhow!(
                "failed to decode module metadata ({module_err}) and component ({component_err})"
            )),
        },
    }
}
