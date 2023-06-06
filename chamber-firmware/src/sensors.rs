use crate::atlas::{AtlasSensor, PendingOperation};

pub struct AtlasScientificSensors<const SIZE: usize> {
    pub sensors: [&'static mut dyn AtlasSensor; SIZE],
    pub current_operation: Option<PendingOperation>,
}
