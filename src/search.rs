use crate::UartTrait;

#[derive(Debug)]
pub struct SearchState {
    // The address of the last found device
    address: u64,

    // bitflags of discrepancies found
    discrepancies: u64,

    // index of the last (leftmost / closest to MSB) discrepancy bit. This can be calculated from the
    // discrepancy bitflags, but it's cheaper to just save it. Index is an offset from the LSB
    last_discrepancy_index: u8,
}

fn device_search(
    uart: &mut dyn UartTrait,
    search_state: Option<&SearchState>,
    only_alarming: bool,
) -> Result<Option<(crate::Rom, SearchState)>, crate::Error> {
    if let Some(search_state) = search_state {
        if search_state.discrepancies == 0 {
            return Ok(None);
        }
    }
    
    if !crate::low_level::ow_reset(uart).ok_or(crate::Error::Uart)?  {
        return Ok(None);
    }

    crate::low_level::ow_write_byte(uart, match only_alarming {
        true => crate::Cmd::SearchAlarm as u8,
        false => crate::Cmd::SearchNormal as u8,
    }).ok_or(crate::Error::Uart)?;


    let mut last_discrepancy_index: u8 = 0;
    let mut address;
    let mut discrepancies;
    let continue_start_bit;

    if let Some(search_state) = search_state {
        // follow up to the last discrepancy
        for bit_index in 0..search_state.last_discrepancy_index {
            let _false_bit = !crate::low_level::ow_read_bit(uart).ok_or(crate::Error::Uart)?;
            let _true_bit = !crate::low_level::ow_read_bit(uart).ok_or(crate::Error::Uart)?;
            let was_discrepancy_bit =
                (search_state.discrepancies & (1_u64 << (bit_index as u64))) != 0;
            if was_discrepancy_bit {
                last_discrepancy_index = bit_index;
            }
            let previous_chosen_bit =
                (search_state.address & (1_u64 << (bit_index as u64))) != 0;

            // choose the same as last time
            crate::low_level::ow_write_bit(uart, previous_chosen_bit).ok_or(crate::Error::Uart)?;
        }
        address = search_state.address;
        // This is the discrepancy bit. False is always chosen to start, so choose true this time
        {
            let false_bit = !crate::low_level::ow_read_bit(uart).ok_or(crate::Error::Uart)?;
            let true_bit = !crate::low_level::ow_read_bit(uart).ok_or(crate::Error::Uart)?;
            if !(false_bit && true_bit) {
                // A different response was received than last search
                return Err(crate::Error::UnexpectedResponse);
            }
            let address_mask = 1_u64 << (search_state.last_discrepancy_index as u64);
            address |= address_mask;
            crate::low_level::ow_write_bit(uart, true).ok_or(crate::Error::Uart)?;
        }

        //keep all discrepancies except the last one
        discrepancies = search_state.discrepancies
            & !(1_u64 << (search_state.last_discrepancy_index as u64));
        continue_start_bit = search_state.last_discrepancy_index + 1;
    } else {
        address = 0;
        discrepancies = 0;
        continue_start_bit = 0;
    }
    for bit_index in continue_start_bit..64 {
        let false_bit = !crate::low_level::ow_read_bit(uart).ok_or(crate::Error::Uart)?;
        let true_bit = !crate::low_level::ow_read_bit(uart).ok_or(crate::Error::Uart)?;
        let chosen_bit = match (false_bit, true_bit) {
            (false, false) => {
                // No devices responded to the search request
                return Err(crate::Error::UnexpectedResponse);
            }
            (false, true) => {
                // All remaining devices have the true bit set
                true
            }
            (true, false) => {
                // All remaining devices have the false bit set
                false
            }
            (true, true) => {
                // Discrepancy, multiple values reported
                // choosing the lower value here
                discrepancies |= 1_u64 << (bit_index as u64);
                last_discrepancy_index = bit_index;
                false
            }
        };
        let address_mask = 1_u64 << (bit_index as u64);
        if chosen_bit {
            address |= address_mask;
        } else {
            address &= !address_mask;
        }
        crate::low_level::ow_write_bit(uart, chosen_bit).ok_or(crate::Error::Uart)?;
    }
    one_wire_bus::crc::check_crc8::<()>(&address.to_le_bytes()).map_err(|_| crate::Error::CrcMismatch)?;
    Ok(Some((
        crate::Rom(address.to_le_bytes()),
        SearchState {
            address,
            discrepancies,
            last_discrepancy_index,
        },
    )))
}

pub struct DeviceSearch<'a> {
    uart: &'a mut dyn UartTrait,
    state: Option<SearchState>,
    finished: bool,
    only_alarming: bool,
}

impl<'a> DeviceSearch<'a> {
    pub fn new(uart: &'a mut dyn UartTrait, only_alarming: bool) -> Self {
        Self {
            uart,
            state: None,
            finished: false,
            only_alarming,
        }
    }
}

impl<'a> Iterator for DeviceSearch<'a>
{
    type Item = Result<crate::Rom, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let result = device_search(self.uart, self.state.as_ref(), self.only_alarming);
        match result {
            Ok(Some((address, search_state))) => {
                self.state = Some(search_state);
                Some(Ok(address))
            }
            Ok(None) => {
                self.state = None;
                self.finished = true;
                None
            }
            Err(err) => {
                self.state = None;
                self.finished = true;
                Some(Err(err))
            }
        }
    }
}