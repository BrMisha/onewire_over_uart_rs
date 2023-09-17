#![cfg_attr(not(std), no_std)]

mod low_level;

pub enum Baudrate {
    Br9600,
    Br115200,
}

pub trait UartTrait {
    fn set_baudrate(&mut self, br: Baudrate);

    fn clear_rx(&mut self);
    fn clear_tx(&mut self);
    fn clear_all(&mut self) {
        self.clear_tx();
        self.clear_rx();
    }

    fn read_byte(&mut self) -> Option<u8>;
    fn write_byte(&self, data: u8); // block till transferring is done
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    // #[cfg(crc)]
    ResetError,
}

const OW_ROMCODE_SIZE: usize = 8;
#[derive(Debug, Copy, Clone, Default)]
pub struct Rom(pub [u8; OW_ROMCODE_SIZE]);

pub enum Cmd {
    SEARCHROM = 0xF0,
    READROM = 0x33,
    MATCHROM = 0x55,
    SKIPROM = 0xCC,
}

/*
#define	OW_SEARCH_FIRST	0xFF		// start new search
#define	OW_PRESENCE_ERR	0xFF
#define	OW_DATA_ERR	    0xFE
#define OW_LAST_DEVICE	0x00		// last device found
//			0x01 ... 0x40: continue searching
*/



pub enum FamilyCode {
    DS1990 = 1,
    DS2405 = 5,
    DS2413 = 0x3A,
    DS1822 = 0x22,
    DS2430 = 0x14,
    DS2431 = 0x2d,
    DS18S20 = 0x10,
    DS18B20 = 0x28,
    DS2433 = 0x23,
}

pub fn reset(uart: &mut dyn UartTrait) -> bool {
    low_level::ow_reset(uart)
}

pub fn read_rom(uart: &mut dyn UartTrait) -> Result<Rom, Error> {
    if reset(uart) != true {
        return Err(Error::ResetError);
    }

    low_level::ow_write_byte(uart, Cmd::READROM as u8);

    let mut rom: Rom = Default::default();

    for i in 0..8 {
        rom.0[i] = low_level::ow_read_byte(uart);
    }

    Ok(rom)
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
