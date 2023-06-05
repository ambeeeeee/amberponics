pub struct Frame<'b, T: FrameData<'b>> {
    id: Option<u8>,
    destination: u64,
    destination_small: u16,
    broadcast_radius: Option<u8>,
    data: &'b T,
}

impl<'b, T: FrameData<'b>> Frame<'b, T> {
    pub fn write(self, buffer: &mut [u8]) {
        let mut offset = 0;

        buffer[offset] = 0x7E;
        offset += 1;

        // Placeholder for length, we'll circle back for that one
        offset += 2;

        // Frame type
        buffer[offset] = self.data.frame_type();
        offset += 1;

        // Frame ID
        buffer[offset] = self.id.unwrap_or(0);
        offset += 1;

        // Destination Address
        let dest_array = self.destination.to_be_bytes();
        buffer[offset..dest_array.len()].copy_from_slice(&dest_array);
        offset += dest_array.len();

        // 16-Bit Destination Address
        let dest_array = self.destination_small.to_be_bytes();
        buffer[offset..dest_array.len()].copy_from_slice(&dest_array);
        offset += dest_array.len();

        // Broadcast Radius
        buffer[offset] = self.broadcast_radius.unwrap_or(0);
        offset += 1;
    }
}

pub trait FrameData<'b>: Sized {
    fn frame_type(&self) -> u8;
    fn write(self, buffer: &'b mut [u8]) -> usize;
    fn read(self, buffer: &'b [u8]) -> Option<Self>;
}

pub struct LocalATCommandRequest<'a> {
    command: [char; 2],
    value: &'a [u8],
}

impl FrameData<'_> for LocalATCommandRequest<'_> {
    fn frame_type(&self) -> u8 {
        0x08
    }

    fn write(self, buffer: &mut [u8]) -> usize {
        let mut offset = 0;

        // AT Command
        buffer[offset..1].copy_from_slice(&self.command.map(|element| element as u8));
        offset += 1;

        // Parameter Value
        buffer[offset..self.value.len()].copy_from_slice(self.value);
        offset += self.value.len();

        offset
    }

    fn read(self, _buffer: &[u8]) -> Option<Self> {
        todo!()
    }
}

pub struct LocalATCommandResponse<'a> {
    command: [char; 2],
    status: LocalATCommandResponseStatus,
    data: &'a [u8],
}

#[repr(u8)]
pub enum LocalATCommandResponseStatus {
    Ok = 0,
    Error = 1,
    InvalidCommand = 2,
    InvalidParameter = 3,
}

impl<'a, 'b: 'a> FrameData<'b> for LocalATCommandResponse<'a> {
    fn frame_type(&self) -> u8 {
        0x88
    }

    fn write(self, _buffer: &mut [u8]) -> usize {
        todo!()
    }

    fn read(self, buffer: &'b [u8]) -> Option<Self> {
        let mut offset = 0;

        // Command
        let command = &buffer[offset..1];
        offset += command.len();

        // Command Status
        let command_status = buffer[offset];
        offset += 1;

        // Command Data
        let command_data = if buffer.len() >= offset {
            &buffer[offset..buffer.len()]
        } else {
            &[0]
        };

        let command_status = match command_status {
            0 => LocalATCommandResponseStatus::Ok,
            1 => LocalATCommandResponseStatus::Error,
            2 => LocalATCommandResponseStatus::InvalidCommand,
            3 => LocalATCommandResponseStatus::InvalidParameter,

            _ => return None,
        };

        Some(Self {
            command: [command[0] as _, command[1] as _],
            status: command_status,
            data: command_data,
        })
    }
}

pub struct ModemStatus {
    pub status: ModemStatusType,
}

impl<'b> FrameData<'b> for ModemStatus {
    fn frame_type(&self) -> u8 {
        0x8A
    }

    fn write(self, _buffer: &'b mut [u8]) -> usize {
        todo!()
    }

    fn read(self, buffer: &'b [u8]) -> Option<Self> {
        let status = buffer[0];

        let status = match status {
            0x00 => ModemStatusType::PowerUp,
            0x01 => ModemStatusType::WatchdogReset,
            0x02 => ModemStatusType::JoinedNetwork,
            0x03 => ModemStatusType::Disassociated,
            0x06 => ModemStatusType::CoordinatorStarted,
            0x07 => ModemStatusType::NetworkSecurityKeyUpdated,
            0x0D => ModemStatusType::VoltageSupplyLimitExceeded,
            0x11 => ModemStatusType::ModemConfigurationChangedWhileJoining,
            0x3B => ModemStatusType::SecureSessionEstablished,
            0x3C => ModemStatusType::SecureSessionEnded,
            0x3D => ModemStatusType::SecureSessionAuthenticationFailed,
            0x3E => ModemStatusType::CoordinatorDetectedPanIdConflict,
            0x3F => ModemStatusType::CoordinatorChangedPanId,
            0x32 => ModemStatusType::BleConnect,
            0x33 => ModemStatusType::BleDisconnect,
            0x34 => ModemStatusType::NoSecureSessionConnection,
            0x40 => ModemStatusType::RouterPanIdChanged,
            0x42 => ModemStatusType::NetworkWatchdogTimerExpiredThrice,
            error => ModemStatusType::StackError(error),
        };

        Some(Self { status })
    }
}

pub enum ModemStatusType {
    PowerUp,
    WatchdogReset,
    JoinedNetwork,
    Disassociated,
    CoordinatorStarted,
    NetworkSecurityKeyUpdated,
    VoltageSupplyLimitExceeded,
    ModemConfigurationChangedWhileJoining,
    SecureSessionEstablished,
    SecureSessionEnded,
    SecureSessionAuthenticationFailed,
    CoordinatorDetectedPanIdConflict,
    CoordinatorChangedPanId,
    BleConnect,
    BleDisconnect,
    NoSecureSessionConnection,
    RouterPanIdChanged,
    NetworkWatchdogTimerExpiredThrice,
    StackError(u8),
}

pub struct TransmitRequest<'a> {
    destination: u64,
    destination_small: u16,
    broadcast_radius: Option<u8>,
    data: &'a [u8],
}

impl<'a, 'b> FrameData<'b> for TransmitRequest<'a> {
    fn frame_type(&self) -> u8 {
        0x10
    }

    fn write(self, buffer: &'b mut [u8]) -> usize {
        let mut offset = 0;

        // Destination Address
        let dest_array = self.destination.to_be_bytes();
        buffer[offset..dest_array.len()].copy_from_slice(&dest_array);
        offset += dest_array.len();

        // 16-Bit Destination Address
        let dest_array = self.destination_small.to_be_bytes();
        buffer[offset..dest_array.len()].copy_from_slice(&dest_array);
        offset += dest_array.len();

        // Broadcast Radius
        buffer[offset] = self.broadcast_radius.unwrap_or(0);
        offset += 1;

        // Transmit Options
        buffer[offset] = 0;
        offset += 1;

        // Data
        let data_buffer = &mut buffer[offset..];

        if data_buffer.len() <= self.data.len() {
            data_buffer[..self.data.len()].copy_from_slice(self.data);
        }

        offset
    }

    fn read(self, _buffer: &'b [u8]) -> Option<Self> {
        todo!()
    }
}

pub struct ExplicitAddressingCommandRequest<'a> {
    destination: u64,
    destination_small: u16,
    source_endpoint: u8,
    dest_endpoint: u8,
    cluster_id: u16,
    profile_id: u16,
    broadcast_radius: u8,
    data: &'a [u8],
}
