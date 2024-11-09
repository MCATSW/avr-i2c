#![no_std]

pub mod hardware;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Read,
    Write,
}

pub trait I2CBus {
    type StartConditionError;
    type StopCondidionError;
    type SendHeaderError;
    type SendError;
    type ReadError;

    fn init(&self);
    fn start_condition(&self) -> Result<(), Self::StartConditionError>;
    fn stop_condition(&self) -> Result<(), Self::StopCondidionError>;
    fn send_header(&self, address: u8, direction: Direction) -> Result<(), Self::SendHeaderError>;
    fn send(&self, data: &[u8]) -> Result<(), Self::SendError>;
    fn read(&self, data: &mut [u8], end_with_nack: bool) -> Result<(), Self::ReadError>;
}
