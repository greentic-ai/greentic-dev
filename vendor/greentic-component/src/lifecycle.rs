use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lifecycle {
    pub init: bool,
    pub health: bool,
    pub shutdown: bool,
}

impl Lifecycle {
    pub fn is_noop(&self) -> bool {
        !(self.init || self.health || self.shutdown)
    }
}
