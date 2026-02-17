pub mod ata;
pub mod background_control;
pub mod format_unit;
pub mod get_lba_status;
pub mod get_stream_status;
pub mod inquiry;
pub mod log_select;
pub mod log_sense;
pub mod mode_select;
pub mod mode_sense;
pub mod persistent_reserve_in;
pub mod persistent_reserve_out;
pub mod read;
pub mod read_buffer;
pub mod read_capacity;
pub mod read_defect_data;
pub mod reassign_blocks;
pub mod receive_diagnostic_results;
pub mod report_identifying_information;
pub mod report_luns;
pub mod report_supported_operation_codes;
pub mod report_supported_task_management_functions;
pub mod report_timestamp;
pub mod request_sense;
pub mod sanitize;
pub mod security_protocol_in;
pub mod security_protocol_out;
pub mod send_diagnostic;
pub mod sense;
pub mod set_identifying_information;
pub mod set_timestamp;
pub mod shortcut;
pub mod start_stop_unit;
pub mod stream_control;
pub mod synchronize_cache;
pub mod test_unit_ready;
pub mod unmap;
pub mod verify;
pub mod write;
pub mod write_and_verify;
pub mod write_atomic;
pub mod write_buffer;
pub mod write_long;
pub mod write_same;
pub mod write_stream;

use std::{borrow::BorrowMut, mem::size_of};

use crate::{result_data::ResultData, DataDirection};

pub trait Command {
    type CommandBuffer;
    type DataBuffer;
    /// usually set it to the same as DataBuffer, but it can also be something like Box<DataBuffer>
    type DataBufferWrapper: BorrowMut<Self::DataBuffer>;
    type ReturnType;

    fn direction(&self) -> DataDirection;
    fn command(&self) -> Self::CommandBuffer;
    fn data(&self) -> Self::DataBufferWrapper;

    /// useful if have some custom data wrapper or want to trim data
    fn data_size(&self) -> u32 {
        size_of::<Self::DataBuffer>() as u32
    }

    fn process_result(&self, result: ResultData<Self::DataBufferWrapper>) -> Self::ReturnType;
}

pub(crate) fn get_array<const N: usize>(bytes: &[u8]) -> ([u8; N], &[u8]) {
    let mut array: [u8; N] = [0; N];
    let min_len = usize::min(array.len(), bytes.len());
    array[..min_len].copy_from_slice(&bytes[..min_len]);

    (array, &bytes[min_len..])
}

macro_rules! bitfield_bound_check {
    ( $num:expr, $bit_count:expr, $name:literal ) => {
        if std::mem::size_of_val(&$num) as u32 * 8 - $num.leading_zeros() > $bit_count {
            Err(crate::Error::ArgumentOutOfBounds(format!(
                concat!(
                    $name,
                    " is out of bounds. The maximum possible value is {}, but {} was provided."
                ),
                1u128.wrapping_shl($bit_count) - 1,
                $num
            )))
        } else {
            Ok(())
        }
    };
}

pub(crate) use bitfield_bound_check;
