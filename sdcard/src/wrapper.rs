/// Because `k210_hal` uses `embedded_hal` version 1.0.0-alpha
/// which causes a conflicting trait implementation with the
/// version 0.2.4 that `embedded_sdmmc` uses, I have to write a
/// wrapper to make it compatible.
use core::convert::Infallible;
use core::ops::{Deref, DerefMut};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use k210_hal::gpiohs::{Floating, Gpiohs0, Output};
use k210_hal::prelude::*;
use k210_hal::spi::{Spi, Spi01, SpiError};
use nb;

pub struct GpiohsWrapper {
    pub gpiohs: Gpiohs0<Output<Floating>>,
}

impl OutputPin for GpiohsWrapper {
    type Error = Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.gpiohs.try_set_low()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.gpiohs.try_set_high()
    }
}

pub struct SpiWrapper<SPI> {
    pub spi: Spi<SPI>,
}

impl<SPI: Spi01> Deref for SpiWrapper<SPI> {
    type Target = Spi<SPI>;

    fn deref(&self) -> &Self::Target {
        &self.spi
    }
}

impl<SPI: Spi01> DerefMut for SpiWrapper<SPI> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.spi
    }
}

impl<SPI: Spi01> FullDuplex<u8> for SpiWrapper<SPI> {
    type Error = SpiError;

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.spi.try_send(word)
    }

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.spi.try_read()
    }
}
