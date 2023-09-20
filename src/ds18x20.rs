use crate::{low_level, Error, Rom, UartTrait};

enum Cmd {
    ConvertTemp = 0x44,
    RscratchPad = 0xBE,
}

pub fn start_measure(uart: &mut dyn UartTrait, rom: Option<&Rom>) -> Result<(), Error> {
    //Reset, skip ROM and send command to read Scratchpad
    if !low_level::ow_reset(uart).ok_or(Error::Uart)? {
        return Err(Error::ResetError);
    }

    match rom {
        None => low_level::ow_write_byte(uart, crate::Cmd::SkipRom as u8).ok_or(Error::Uart)?,
        Some(rom) => crate::match_rom(uart, rom)?,
    };

    low_level::ow_write_byte(uart, Cmd::ConvertTemp as u8);

    Err(Error::ResetError)
}

pub fn read_data(
    uart: &mut dyn UartTrait,
    rom: Option<&Rom>,
    check_crc: bool,
) -> Result<[u8; 2], Error> {
    //Reset, skip ROM and send command to read Scratchpad
    if !low_level::ow_reset(uart).ok_or(Error::Uart)? {
        return Err(Error::ResetError);
    }

    match rom {
        None => low_level::ow_write_byte(uart, crate::Cmd::SkipRom as u8).ok_or(Error::Uart)?,
        Some(rom) => crate::match_rom(uart, rom)?,
    };

    low_level::ow_write_byte(uart, Cmd::RscratchPad as u8);

    match check_crc {
        true => {
            let mut buff: [u8; 9] = Default::default();
            for i in &mut buff {
                *i = low_level::ow_read_byte(uart).ok_or(Error::Uart)?;
            }

            match one_wire_bus::crc::check_crc8::<()>(&buff) {
                Err(_) => Err(crate::Error::CrcMismatch),
                Ok(_) => Ok([buff[0], buff[1]]),
            }
        }
        false => {
            let r = [
                low_level::ow_read_byte(uart).ok_or(Error::Uart)?,
                low_level::ow_read_byte(uart).ok_or(Error::Uart)?,
            ];
            Ok(r)
        }
    }
}

pub fn convert_to_celsius(data: &[u8; 2]) -> f32 {
    //Store temperature integer digits and decimal digits
    let mut digit: u8 = data[0] >> 4;
    digit |= (data[1] & 0x07) << 4;
    //Store decimal digits
    let mut decimal: u8 = data[0] & 0xf;
    decimal *= 6;

    if data[1] > 0xFB {
        digit = 127 - digit;
    }
    let mut temperature = if decimal < 100 {
        digit as f32 + (decimal as f32 / 100.0)
    } else {
        digit as f32 + (decimal as f32 / 1000.0)
    };

    if data[1] > 0xFB {
        temperature = -temperature;
    }

    temperature
}
