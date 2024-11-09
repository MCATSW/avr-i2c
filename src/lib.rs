#![no_std]

//! A basic AVR rust I2C implementation.

#[cfg(feature = "hardware-atmega328p")]
pub mod hardware_atmega328p;

/// Represents data direction used for the R/W bit in the I2C header.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Read,
    Write,
}

/// Represents an I2C driver.
pub trait I2CBus {
    /// An error type for the [`start_condition`] method.
    ///
    /// [`start_condition`]: Self::start_condition
    type StartConditionError;

    /// An error type for the [`stop_condition`] method.
    ///
    /// [`stop_condition`]: Self::stop_condition
    type StopCondidionError;

    /// An error type for the [`send_header`] method.
    ///
    /// [`send_header`]: Self::send_header
    type SendHeaderError;

    /// An error type for the [`send`] method.
    ///
    /// [`send`]: Self::send
    type SendError;

    /// An error type for the [`read`] method.
    ///
    /// [`read`]: Self::read
    type ReadError;

    /// Initializes the [`I2CBus`]
    ///
    /// [`I2CBus`]: Self
    fn init(&self);

    /// Creates an I2C start condition on the bus.
    fn start_condition(&self) -> Result<(), Self::StartConditionError>;

    /// Creates an I2C stop condition on the bus.
    fn stop_condition(&self) -> Result<(), Self::StopCondidionError>;

    /// Sends an I2C header to the bus.
    fn send_header(&self, address: u8, direction: Direction) -> Result<(), Self::SendHeaderError>;

    /// Sends data to an I2C slave.
    fn send(&self, data: &[u8]) -> Result<(), Self::SendError>;

    /// Reads data from an I2C slave.
    fn read(&self, data: &mut [u8], end_with_nack: bool) -> Result<(), Self::ReadError>;
}
