use embedded_storage::{ReadStorage, Storage, nor_flash};

use crate::peripherals::{EEPROM, FLASH};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EepromError {
    OutOfRange,
}

impl ReadStorage for EEPROM {
    type Error = EepromError;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > self.capacity() {
            return Err(EepromError::OutOfRange);
        }

        todo!()
    }

    #[inline]
    fn capacity(&self) -> usize {
        #[cfg(any(
            feature = "lpc11u34",
            feature = "lpc11u35",
            feature = "lpc11u36",
            feature = "lpc11u37"
        ))]
        return 4096;

        #[cfg(not(any(
            feature = "lpc11u34",
            feature = "lpc11u35",
            feature = "lpc11u36",
            feature = "lpc11u37"
        )))]
        return 0;
    }
}

impl Storage for EEPROM {
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlashError {
    // TODO
}

impl nor_flash::NorFlashError for FlashError {
    fn kind(&self) -> nor_flash::NorFlashErrorKind {
        todo!()
    }
}

impl nor_flash::ErrorType for FLASH {
    type Error = FlashError;
}

impl nor_flash::ReadNorFlash for FLASH {
    const READ_SIZE: usize = 4;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn capacity(&self) -> usize {
        todo!()
    }
}

impl nor_flash::NorFlash for FLASH {
    const WRITE_SIZE: usize = 0;

    const ERASE_SIZE: usize = 0;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        todo!()
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }
}

