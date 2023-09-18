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


pub const OW_SEARCH_FIRST: u8 = 0xFF;		// start new search
pub const OW_PRESENCE_ERR: u8 = 0xFF;
pub const OW_DATA_ERR: u8 = 0xFE;
pub const OW_LAST_DEVICE: u8 = 0x00;		// last device found
//			0x01 ... 0x40: continue searching


pub fn ow_search_rom(uart: &mut dyn UartTrait, mut diff: u8, rom: &mut crate::Rom ) -> u8
{
    let mut i: u8;
    let mut j: u8;
    let mut next_diff: u8;
    let mut b: bool;

    if ow_reset(uart) != true {
        return OW_PRESENCE_ERR;       // error, no device found
    }

    ow_write_byte(uart, crate::Cmd::SEARCHROM as u8);     // ROM search command
    next_diff = OW_LAST_DEVICE;      // unchanged on last device

    let mut i = (crate::OW_ROMCODE_SIZE * 8) as u8;         // 8 bytes
    for id in rom.0.iter_mut() {
        j = 8;                        // 8 bits
        loop
        {
            b = ow_read_bit(uart);			// read bit
            if ow_read_bit(uart) { // read complement bit
                if b {                 // 11
                    return OW_DATA_ERR;  // data error
                }
            }
            else
            {
                if b == false { // 00 = 2 devices
                    if diff > i || (((*id & 1) != 0) && diff != i)  {
                        b = true;               // now 1
                        next_diff = i;       // next pass 0
                    }
                }
            }
            ow_write_bit(uart, b);               // write bit
            *id >>= 1;
            if b {    // store bit
                *id |= 0x80;
            }
            i -= 1;

            j -= 1;
            if j == 0 { break }
        }
        //while( --j );
        //id.next();                           // next byte
        if i == 0 { break }
    }

    return next_diff;                  // to continue search
}

pub fn ow_find_rom(uart: &mut dyn UartTrait, diff: &mut u8, rom: &mut crate::Rom)
{
    loop
    {
        *diff = ow_search_rom(uart, *diff, rom);
        if *diff==OW_PRESENCE_ERR || *diff==OW_DATA_ERR || *diff == OW_LAST_DEVICE {
            return;
        }
    }
}
