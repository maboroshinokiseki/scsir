#![allow(dead_code)]

use std::{
    borrow::BorrowMut,
    fs::OpenOptions,
    io,
    mem::size_of_val,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{file_descriptor::FileDescriptor, Command};

#[derive(Debug)]
pub struct Scsi {
    path: PathBuf,
    file_descriptor: FileDescriptor,
    timeout: Duration,
}

impl Scsi {
    pub fn new<P: AsRef<Path> + ?Sized>(path: &P) -> crate::Result<Scsi> {
        let mut options = OpenOptions::new();
        options.read(true).write(true);
        let file_descriptor = FileDescriptor::open(&path, options)?;

        if !file_descriptor.is_block()? {
            return Err(crate::Error::NotBlockDevice(path.as_ref().to_owned()));
        }

        if !Self::is_scsi_device(&file_descriptor)? {
            return Err(crate::Error::NotScsiDevice(path.as_ref().to_owned()));
        }

        Ok(Scsi {
            path: path.as_ref().to_owned(),
            file_descriptor,
            timeout: Duration::from_millis(SG_DEFAULT_TIMEOUT),
        })
    }

    #[cfg(target_os = "linux")]
    pub fn issue<T: Command>(&self, command: &T) -> T::ReturnType {
        use nix::libc;

        use crate::{
            command::sense::{SenseData, MAX_SENSE_BUFFER_LENGTH},
            os::linux::{AccessFlags, AuxiliaryInfo, DriverStatus, SgIoHeader},
            result_data::{ResultData, Status},
        };

        const SG_IO: u32 = 0x2285;

        let command_buffer = command.command();
        let mut data_buffer = command.data();
        let raw_pointer_to_data_buffer = &mut data_buffer as *mut _;
        let mut sense_buffer = [0u8; MAX_SENSE_BUFFER_LENGTH];

        let size_of_command_buffer = size_of_val(&command_buffer) as u8;
        let size_of_data_buffer = command.data_size();
        let size_of_sense_buffer = size_of_val(&sense_buffer) as u8;

        let pointer_of_command_buffer = Some(&command_buffer);

        let pointer_of_data_buffer = if size_of_data_buffer == 0 {
            None
        } else {
            Some(data_buffer.borrow_mut())
        };

        let pointer_of_sense_buffer = Some(&mut sense_buffer);

        let mut sg_header = SgIoHeader {
            interface_id: b'S' as i32,
            data_direction: command.direction().into(),
            command_length: size_of_command_buffer,
            max_sense_buffer_length: size_of_sense_buffer,
            iovec_count: 0,
            data_length: size_of_data_buffer,
            data: pointer_of_data_buffer,
            command: pointer_of_command_buffer,
            sense_buffer: pointer_of_sense_buffer,
            timeout: self
                .timeout
                .as_millis()
                .clamp(u32::MIN as u128, u32::MAX as u128) as u32,
            flags: AccessFlags::DEFAULT,
            pack_id: 0,
            user_pointer: 0,
            status: 0,
            masked_status: 0,
            message_status: 0,
            sense_buffer_written: 0,
            host_status: 0,
            driver_status: DriverStatus::OK,
            residual_count: 0,
            duration: 0,
            info: AuxiliaryInfo::OK,
        };

        let ioctl_result = unsafe {
            libc::ioctl(
                self.file_descriptor.raw(),
                SG_IO.try_into().unwrap(),
                &mut sg_header,
            )
        };

        let sense_buffer_written = sg_header.sense_buffer_written as usize;
        let sense_data = sg_header
            .sense_buffer
            .map(|b| SenseData::parse(b, sense_buffer_written))
            .unwrap_or(SenseData::None);

        let result_data = ResultData {
            ioctl_result,
            transfered_data_length: sg_header.data_length as usize
                - sg_header.residual_count as usize,
            data: unsafe { &mut *raw_pointer_to_data_buffer },
            transfered_sense_length: sense_buffer_written,
            sense_buffer: &sense_data,
            status: Status::from(sg_header.status),
            host_status: sg_header.host_status.into(),
            driver_status: sg_header.driver_status,
        };

        command.process_result(result_data)
    }

    #[cfg(target_os = "windows")]
    pub fn issue<T: Command>(&self, command: &T) -> T::ReturnType {
        use std::slice;

        use windows::Win32::{
            Foundation::HANDLE,
            Storage::IscsiDisc::{
                IOCTL_SCSI_PASS_THROUGH_DIRECT, SCSI_IOCTL_DATA_BIDIRECTIONAL, SCSI_IOCTL_DATA_IN,
                SCSI_IOCTL_DATA_OUT, SCSI_IOCTL_DATA_UNSPECIFIED,
            },
            System::IO::DeviceIoControl,
        };

        use crate::{
            command::sense::SenseData,
            os::windows::ScsiPassThroughDirectWrapper,
            result_data::{ResultData, Status},
        };

        const MAX_COMMAND_LENGTH: u8 = 16;

        let command_buffer = command.command();
        let mut data_buffer = command.data();

        let size_of_command_buffer = size_of_val(&command_buffer) as u8;
        let size_of_data_buffer = command.data_size();

        if size_of_command_buffer > MAX_COMMAND_LENGTH {
            panic!(
                "Current command length is {}, max command length is {}",
                size_of_command_buffer, MAX_COMMAND_LENGTH
            );
        }
        let command_pointer = &command_buffer as *const _ as *const u8;
        let command_slice =
            unsafe { slice::from_raw_parts(command_pointer, size_of_command_buffer as usize) };

        let mut header = ScsiPassThroughDirectWrapper::default();
        let address_of_header = std::ptr::addr_of!(header) as usize;
        let mut spt = &mut header.scsi_pass_through;
        spt.Length = size_of_val(spt) as u16;
        spt.CdbLength = size_of_command_buffer;
        spt.SenseInfoLength = size_of_val(&header.sense) as u8;
        spt.DataIn = match command.direction() {
            crate::DataDirection::None => SCSI_IOCTL_DATA_UNSPECIFIED,
            crate::DataDirection::ToDevice => SCSI_IOCTL_DATA_OUT,
            crate::DataDirection::FromDevice => SCSI_IOCTL_DATA_IN,
            crate::DataDirection::ToFromDevice => SCSI_IOCTL_DATA_BIDIRECTIONAL,
            crate::DataDirection::Unknown => SCSI_IOCTL_DATA_UNSPECIFIED,
        } as u8;

        spt.DataTransferLength = size_of_data_buffer;

        spt.TimeOutValue = match self
            .timeout
            .as_secs()
            .clamp(u32::MIN as u64, u32::MAX as u64)
        {
            0 => 1,
            n => n as u32,
        };

        spt.DataBuffer = data_buffer.borrow_mut() as *mut _ as _;

        spt.SenseInfoOffset =
            (std::ptr::addr_of!(header.sense) as usize - address_of_header) as u32;

        spt.Cdb[..command_slice.len()].copy_from_slice(command_slice);

        let mut bytes_returned = 0;

        let success = unsafe {
            DeviceIoControl(
                HANDLE(self.file_descriptor.raw() as isize),
                IOCTL_SCSI_PASS_THROUGH_DIRECT,
                Some(&header as *const _ as _),
                size_of_val(&header) as u32,
                Some(&mut header as *mut _ as _),
                size_of_val(&header) as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        let ioctl_result = match success.as_bool() {
            true => 0,
            false => -1,
        };

        let sense_data = SenseData::parse(
            &header.sense,
            header.scsi_pass_through.SenseInfoLength as usize,
        );

        let result_data = ResultData {
            ioctl_result,
            transfered_data_length: header.scsi_pass_through.DataTransferLength as usize,
            data: &mut data_buffer,
            transfered_sense_length: header.scsi_pass_through.SenseInfoLength as usize,
            sense_buffer: &sense_data,
            status: Status::from(header.scsi_pass_through.ScsiStatus),
        };

        command.process_result(result_data)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    #[cfg(target_os = "linux")]
    fn is_scsi_device(file: &FileDescriptor) -> crate::Result<bool> {
        use nix::libc;

        const SG_GET_VERSION_NUM: u32 = 0x2282;

        let mut version = 0_i32;
        let result = unsafe {
            libc::ioctl(
                file.raw(),
                SG_GET_VERSION_NUM.try_into().unwrap(),
                &mut version,
            )
        };

        if result != 0 {
            Err(io::Error::last_os_error())?;
        }

        if version < 30000 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    #[cfg(target_os = "windows")]
    fn is_scsi_device(file: &FileDescriptor) -> crate::Result<bool> {
        use std::mem::size_of;

        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Storage::IscsiDisc::{IOCTL_SCSI_GET_ADDRESS, SCSI_ADDRESS};
        use windows::Win32::System::IO::DeviceIoControl;

        let mut scsi_address = SCSI_ADDRESS::default();
        let mut bytes_returned = 0;
        let success = unsafe {
            DeviceIoControl(
                HANDLE(file.raw() as isize),
                IOCTL_SCSI_GET_ADDRESS,
                None,
                0,
                Some(&mut scsi_address as *mut _ as _),
                size_of::<SCSI_ADDRESS>() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };

        if success == false {
            Err(io::Error::last_os_error())?;
        }

        if bytes_returned == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

const SG_DEFAULT_TIMEOUT: u64 = 60_000;
