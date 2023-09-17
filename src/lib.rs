#![cfg_attr(not(std), no_std)]

mod low_level;

pub enum Baudrate {
    Br9600,
    Br115200
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

pub fn reset(uart: &mut dyn UartTrait) -> bool {
    low_level::ow_reset(uart)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
