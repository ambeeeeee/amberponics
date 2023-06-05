pub struct Sensors<const SIZE: usize> {
    pub sensors: [&'static mut dyn AtlasSensor; SIZE],
    pub current_operation: Option<PendingOperation>,
}

pub struct PendingOperation {
    pub sensor: &'static dyn AtlasSensor,
    pub operation: PendingAction,
}

pub enum PendingAction {
    Sample { deadline: () },
    Receive { deadline: () },
}

impl Default for PendingAction {
    fn default() -> Self {
        Self::Sample { deadline: () }
    }
}

pub struct OxygenSensor {
    pub last_reading: f64,
    pub action: PendingAction,
}

impl OxygenSensor {
    pub const fn new() -> Self {
        Self {
            last_reading: 0.0,
            action: PendingAction::Sample { deadline: () },
        }
    }
}

impl AtlasSensor for OxygenSensor {
    fn address(&self) -> u32 {
        0x6C
    }

    fn sample_command(&self) -> &'static [u8] {
        &[b'R']
    }

    fn handle_response(&mut self, _response: &[u8]) {
        todo!()
    }
}

#[derive(Default)]
pub struct HumiditySensor {
    pub last_humidity: f64,
    pub last_temperature: f64,
    pub action: PendingAction,
}

impl HumiditySensor {
    pub const fn new() -> Self {
        Self {
            last_humidity: 0.0,
            last_temperature: 0.0,
            action: PendingAction::Sample { deadline: () },
        }
    }
}

impl AtlasSensor for HumiditySensor {
    fn address(&self) -> u32 {
        0x6F
    }

    fn sample_command(&self) -> &'static [u8] {
        &[b'R']
    }

    fn setup_commands(&self) -> &'static [&'static [u8]] {
        &[
            &[b'O', b',', b'T', b',', b'1'],
            &[b'O', b',', b'H', b'U', b'M', b',', b'1'],
        ]
    }

    fn handle_response(&mut self, _response: &[u8]) {
        todo!()
    }
}

pub struct GrowUnitFloat1 {
    pub last_reading: bool,
    pub action: PendingAction,
}

pub struct GrowUnitFloat2 {
    pub last_reading: bool,
    pub action: PendingAction,
}

pub trait AtlasSensor {
    /// Returns a sensor's I2C address.
    fn address(&self) -> u32;

    /// Returns the command string used to sample the device.
    ///
    /// This command is sent to sample the device. The response is
    /// passed to the [`AtlasSensor::handle_response`] implementation
    /// for parsing.
    fn sample_command(&self) -> &'static [u8];

    /// Returns any command strings needed to set up the device.
    ///
    /// They will be executed, and the output will be checked for
    /// [`SensorResponse::Ok`]. If a command fails the device will
    /// be considered faulted.
    fn setup_commands(&self) -> &'static [&'static [u8]] {
        &[]
    }

    /// Handles a response to a sample command for the device.
    fn handle_response(&mut self, response: &[u8]);
}

pub enum SensorResponse {
    Ok,
    UnknownCommand,
    OverVolt,
    UnderVolt,
    Reset,
    Ready,
    Sleeping,
    WakeUp,
}
