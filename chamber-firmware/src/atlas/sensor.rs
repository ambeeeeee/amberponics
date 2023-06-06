use rtic_monotonics::systick::fugit::Instant;

pub struct PendingOperation {
    pub sensor: usize,
    pub operation: PendingAction,
}

#[derive(Clone, Copy)]
pub enum PendingAction {
    Startup { command_index: usize },
    Sample { deadline: Instant<u32, 1, 1000> },
    Receive { deadline: Instant<u32, 1, 1000> },
}

impl Default for PendingAction {
    fn default() -> Self {
        Self::Startup { command_index: 0 }
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
            action: PendingAction::Startup { command_index: 0 },
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

    fn pending_action(&self) -> &PendingAction {
        &self.action
    }

    fn pending_action_mut(&mut self) -> &mut PendingAction {
        &mut self.action
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
            action: PendingAction::Startup { command_index: 0 },
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

    fn pending_action(&self) -> &PendingAction {
        &self.action
    }

    fn pending_action_mut(&mut self) -> &mut PendingAction {
        &mut self.action
    }
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

    /// Returns the current pending action.
    fn pending_action(&self) -> &PendingAction;

    fn pending_action_mut(&mut self) -> &mut PendingAction;

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
