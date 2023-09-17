use crate::{Baudrate, UartTrait};

pub fn ow_reset(uart: &mut dyn UartTrait) -> bool {
    uart.set_baudrate(Baudrate::Br9600);
    uart.clear_all();

    uart.write_byte(0xF0);
    matches!(uart.read_byte(), Some(x) if x != 0xF0)
}

pub fn ow_write_bit(uart: &mut dyn UartTrait, bit: bool) {
    uart.set_baudrate(Baudrate::Br115200);
    uart.clear_all();

    uart.write_byte(match bit {
        true => 0xFF,
        false => 0,
    })
}

pub fn ow_read_bit(uart: &mut dyn UartTrait) -> bool {
    uart.set_baudrate(Baudrate::Br115200);
    uart.clear_all();

    uart.write_byte(0xFF);
    matches!(uart.read_byte(), Some(x) if x > 0xFE)
}

pub fn ow_write_byte(uart: &mut dyn UartTrait, byte: u8) {
    for i in 0..8 {
        ow_write_bit(uart, (byte >> i)&1u8 != 0)
    }
}

pub fn ow_read_byte(uart: &mut dyn UartTrait) -> u8 {
    let mut v = 0u8;

    for i in 0..8 {
        if ow_read_bit(uart) {
            v |= 1 << i;
        }
    }

    v
}