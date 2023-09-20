use crate::{Baudrate, UartTrait};

pub fn ow_reset(uart: &mut dyn UartTrait) -> Option<bool> {
    uart.set_baudrate(Baudrate::Br9600);
    uart.clear_all();

    uart.write_byte(0xF0);
    let r = uart.read_byte()?;

    Some(match r {
        0xF0 => false,
        _ => true,
    })
}

pub fn ow_write_bit(uart: &mut dyn UartTrait, bit: bool) -> Option<()> {
    uart.set_baudrate(Baudrate::Br115200);
    uart.clear_all();

    let r = uart.write_byte(match bit {
        true => 0xFF,
        false => 0,
    });

    match r {
        true => Some(()),
        false => None
    }
}

pub fn ow_read_bit(uart: &mut dyn UartTrait) -> Option<bool> {
    uart.set_baudrate(Baudrate::Br115200);
    uart.clear_all();

    if uart.write_byte(0xFF) == false {
        return None;
    }
    match uart.read_byte() {
        None => None,
        Some(x) => Some(x > 0xFE)
    }
}

pub fn ow_write_byte(uart: &mut dyn UartTrait, byte: u8) -> Option<()> {
    for i in 0..8 {
        ow_write_bit(uart, (byte >> i) & 1u8 != 0)?
    }
    Some(())
}

pub fn ow_read_byte(uart: &mut dyn UartTrait) -> Option<u8> {
    let mut v = 0u8;

    for i in 0..8 {
        if ow_read_bit(uart)? {
            v |= 1 << i;
        }
    }

    Some(v)
}
