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

    uart.write_byte(
        match bit {
            true => 0xFF,
            false => 0
        }
    )
}

pub fn ow_read_bit(uart: &mut dyn UartTrait) -> bool {
    uart.set_baudrate(Baudrate::Br115200);
    uart.clear_all();

    uart.write_byte(0xFF);
    matches!(uart.read_byte(), Some(x) if x > 0xFE)
}

pub fn ow_transfer_byte(uart: &mut dyn UartTrait, mut byte: u8) -> Option<u8> {
    uart.set_baudrate(Baudrate::Br115200);
    uart.clear_all();

    for i in [0..8] {
        uart.write_byte(
            match (byte & 1) != 0 {
                true => 0xFF,
                false => 0,
            }
        );
        byte>>=1;

        match uart.read_byte() {
            Some(x) if x > 0xFE => byte |= 128,
            _ => return None,
        }
    }

    Some(byte & 0xFF)
}