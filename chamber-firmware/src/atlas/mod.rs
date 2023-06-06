mod sensor;

pub use sensor::*;

#[derive(Clone, Copy)]
pub enum ResponseCode {
    Ok,
    UnknownCommand,
    OverVolt,
    UnderVolt,
    Reset,
    Ready,
    Sleeping,
    WakeUp,
}

impl ResponseCode {
    pub fn try_from_probe_response(buffer: &[u8]) -> Option<Self> {
        // Probe splits tokens by <CR>
        let split = buffer.split(|c| *c == '\r' as u8);

        let last_token = split.last()?;

        Self::try_from(core::str::from_utf8(last_token).unwrap()).ok()
    }
}

impl Into<&'static str> for ResponseCode {
    fn into(self) -> &'static str {
        match self {
            ResponseCode::Ok => "*OK",
            ResponseCode::UnknownCommand => "*ER",
            ResponseCode::OverVolt => "*OV",
            ResponseCode::UnderVolt => "*UV",
            ResponseCode::Reset => "*RS",
            ResponseCode::Ready => "*RE",
            ResponseCode::Sleeping => "*SL",
            ResponseCode::WakeUp => "*WA",
        }
    }
}

impl<'a> TryFrom<&'a str> for ResponseCode {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(match value {
            "*OK" => ResponseCode::Ok,
            "*ER" => ResponseCode::UnknownCommand,
            "*OV" => ResponseCode::OverVolt,
            "*UV" => ResponseCode::UnderVolt,
            "*RS" => ResponseCode::Reset,
            "*RE" => ResponseCode::Ready,
            "*SL" => ResponseCode::Sleeping,
            "*WA" => ResponseCode::WakeUp,
            _ => return Err(()),
        })
    }
}

#[derive(Debug)]
pub struct AtlasCommand {
    pub address: usize,
    pub command: heapless::Vec<u8, 64>,
}
