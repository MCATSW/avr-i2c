//! A hardware implementation of I2C for the ATMega328P.

use crate::{Direction, I2CBus};

/// Represents the hardware 2-wire interface
pub struct TWI {
    pub twbr: u8,
}

impl TWI {
    /// Creates a new TWI instance
    pub const fn new(freq_hz: u32) -> Self {
        Self {
            twbr: (16_000_000 / (2 * freq_hz) - 8) as u8,
        }
    }
}

/// Represents status variants of the 2-wire status register
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TWSRStatus {
    StartTransmitted,
    RepeatedStartTransmitted,
    WriteHeaderTransmittedAckReceived,
    WriteHeaderTransmittedNackReceived,
    ReadHeaderTransmittedAckReceived,
    ReadHeaderTransmittedNackReceived,
    DataTransmittedAckReceived,
    DataTransmittedNackReceived,
    DataReceivedAckTransmitted,
    DataReceivedNackTransmitted,
    ArbitrationLost,
    NoInformation,
    BusError,
}

impl TWSRStatus {
    /// Translates a raw TWSR value to a variant of [`Self`].
    ///
    /// If `value` is a valid TWSR status value, a `Some`
    /// value containing [`Self`] is returned.
    /// Otherwise, `None` is returned.
    ///
    /// [`Self`]: Self
    pub fn from_byte(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::BusError),
            0x08 => Some(Self::StartTransmitted),
            0x10 => Some(Self::RepeatedStartTransmitted),
            0x18 => Some(Self::WriteHeaderTransmittedAckReceived),
            0x20 => Some(Self::WriteHeaderTransmittedNackReceived),
            0x28 => Some(Self::DataTransmittedAckReceived),
            0x30 => Some(Self::DataTransmittedNackReceived),
            0x38 => Some(Self::ArbitrationLost),
            0x40 => Some(Self::ReadHeaderTransmittedAckReceived),
            0x48 => Some(Self::ReadHeaderTransmittedNackReceived),
            0x50 => Some(Self::DataReceivedAckTransmitted),
            0x58 => Some(Self::DataReceivedNackTransmitted),
            0xF8 => Some(Self::NoInformation),
            _ => None,
        }
    }
}

/// The address of the 2-wire bit rate register
pub const TWBR: *mut u8 = 0x00B8 as *mut u8;

/// The address of the 2-wire status register
pub const TWSR: *mut u8 = 0x00B9 as *mut u8;

/// The address of the 2-wire data register
pub const TWDR: *mut u8 = 0x00BB as *mut u8;

/// The address of the 2-wire control register
pub const TWCR: *mut u8 = 0x00BC as *mut u8;

/// The mask for setting the TWCR interrupt bit
pub const TWINT: u8 = 0x80;

/// The mask for setting the TWCR enable acknowledge bit
pub const TWEA: u8 = 0x40;

/// The mask for setting the TWCR start condition bit
pub const TWSTA: u8 = 0x20;

/// The mask for setting the TWCR stop condition bit
pub const TWSTO: u8 = 0x10;

/// The mask for setting the TWCR enable bit
pub const TWEN: u8 = 0x04;

/// Awaits TWI hardware availability
///
/// Repeatedly polls the TWCR until it signifies that
/// the TWI hardware has finished its assigned
/// operation, then returns.
pub fn await_hardware() {
    while unsafe { TWCR.read_volatile() } & TWINT == 0 {
        continue;
    }
}

impl I2CBus for TWI {
    type StartConditionError = TWSRStatus;
    type StopCondidionError = ();
    type SendHeaderError = TWSRStatus;
    type SendError = TWSRStatus;
    type ReadError = TWSRStatus;

    fn init(&self) {
        unsafe {
            TWSR.write_volatile(0x00);
            TWBR.write_volatile(self.twbr);
            TWCR.write_volatile(TWEN);
        }
    }

    fn start_condition(&self) -> Result<(), Self::StartConditionError> {
        unsafe {
            TWCR.write_volatile(TWINT | TWSTA | TWEN);
        }
        await_hardware();
        match TWSRStatus::from_byte(unsafe { TWSR.read_volatile() }).unwrap() {
            TWSRStatus::StartTransmitted => Ok(()),
            x => Err(x),
        }
    }

    fn stop_condition(&self) -> Result<(), ()> {
        unsafe {
            TWCR.write_volatile(TWINT | TWSTO | TWEN);
        }
        await_hardware();
        Ok(())
    }

    fn send_header(&self, address: u8, direction: Direction) -> Result<(), Self::SendHeaderError> {
        let payload: u8 = (address << 1)
            | match direction {
                Direction::Read => 1,
                Direction::Write => 0,
            };
        unsafe {
            TWDR.write_volatile(payload);
            TWCR.write_volatile(TWINT | TWEN);
        }
        await_hardware();
        match TWSRStatus::from_byte(unsafe { TWSR.read_volatile() }).unwrap() {
            TWSRStatus::ReadHeaderTransmittedAckReceived if direction == Direction::Read => Ok(()),
            TWSRStatus::WriteHeaderTransmittedAckReceived if direction == Direction::Write => {
                Ok(())
            }
            x => Err(x),
        }
    }

    fn send(&self, data: &[u8]) -> Result<(), Self::SendError> {
        for byte in data {
            unsafe {
                TWDR.write_volatile(*byte);
                TWCR.write_volatile(TWINT | TWEN);
            }
            await_hardware();
            match TWSRStatus::from_byte(unsafe { TWSR.read_volatile() }).unwrap() {
                TWSRStatus::DataTransmittedAckReceived => (),
                x => return Err(x),
            }
        }
        Ok(())
    }

    fn read(&self, data: &mut [u8], end_with_nack: bool) -> Result<(), Self::ReadError> {
        for byte in data {
            unsafe {
                TWCR.write_volatile(TWINT | TWEN | if end_with_nack { TWEA } else { 0 });
            }
            await_hardware();
            match TWSRStatus::from_byte(unsafe { TWSR.read_volatile() }).unwrap() {
                TWSRStatus::DataReceivedAckTransmitted if end_with_nack => (),
                TWSRStatus::DataReceivedNackTransmitted if !end_with_nack => (),
                x => return Err(x),
            }
            *byte = unsafe { TWDR.read_volatile() };
        }
        Ok(())
    }
}
