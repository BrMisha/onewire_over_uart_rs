#![cfg_attr(not(std), no_std)]

pub mod ds18x20;
pub mod low_level;
pub mod search;

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
    fn write_byte(&self, data: u8) -> bool; // block till transferring is done
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Uart,
    ResetError,
    CrcMismatch,
    UnexpectedResponse
}

const OW_ROMCODE_SIZE: usize = 8;

#[derive(Debug, Copy, Clone, Default, PartialOrd, PartialEq)]
pub struct Rom(pub [u8; OW_ROMCODE_SIZE]);
impl Rom {
    pub fn family_code(&self) -> u8 {
        self.0[0]
    }
}

impl core::str::FromStr for Rom {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 23 {
            let _ = u8::from_str_radix("", 16)?; // this causes a ParseIntError::Empty
        }
        Ok(Rom {
            0: [
                u8::from_str_radix(&s[0..2], 16)?,
                u8::from_str_radix(&s[3..5], 16)?,
                u8::from_str_radix(&s[6..8], 16)?,
                u8::from_str_radix(&s[9..11], 16)?,
                u8::from_str_radix(&s[12..14], 16)?,
                u8::from_str_radix(&s[15..17], 16)?,
                u8::from_str_radix(&s[18..20], 16)?,
                u8::from_str_radix(&s[21..23], 16)?,
            ],
        })
    }
}

pub enum Cmd {
    SEARCHROM = 0xF0,
    READROM = 0x33,
    MATCHROM = 0x55,
    SKIPROM = 0xCC,
}

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

pub fn reset(uart: &mut dyn UartTrait) -> Result<bool, Error> {
    low_level::ow_reset(uart).ok_or(Error::Uart)
}

pub fn read_rom(uart: &mut dyn UartTrait) -> Result<Rom, Error> {
    if reset(uart)? != true {
        return Err(Error::ResetError);
    }

    low_level::ow_write_byte(uart, Cmd::READROM as u8);

    let mut rom: Rom = Default::default();

    for i in rom.0.iter_mut() {
        *i = low_level::ow_read_byte(uart).ok_or(Error::Uart)?;
    }

    Ok(rom)
}

pub fn match_rom(uart: &mut dyn UartTrait, rom: &Rom) -> Result<(), Error> {
    if reset(uart)? != true {
        return Err(Error::ResetError);
    }

    low_level::ow_write_byte(uart, Cmd::MATCHROM as u8);

    for x in rom.0 {
        low_level::ow_write_byte(uart, x);
    }

    Ok(())
}

/*
pub struct DeviceSearchIter<'a> {
    search: Option<DeviceSearch>,
    wire: &'a mut dyn UartTrait,
}

impl<'a> Iterator for DeviceSearchIter<'a> {
    type Item = Result<Rom, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut search = self.search.take()?;
        let result = self
            .wire
            .search_next(&mut search)
            .transpose()?;
        self.search = Some(search);
        Some(result)
    }
}*/
