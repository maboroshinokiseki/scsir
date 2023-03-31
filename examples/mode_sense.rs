use std::{fmt::Debug, time::Duration};

use scsir::{
    self,
    shortcut::mode::{
        self, DescriptorStorage, HeaderStorage, ModePage, PageHeaderStorage, PageWrapper,
        PowerConditionPage,
    },
    Scsi,
};

fn main() -> scsir::Result<()> {
    let mut scsi = scsir::Scsi::new("/dev/sdX")?;
    scsi.set_timeout(Duration::from_secs(3));

    println!("Single Page:");
    print_power_condition(&scsi)?;
    println!("\nAll Pages:");
    print_all_modes(&scsi)?;

    Ok(())
}

fn print_power_condition(interface: &Scsi) -> scsir::Result<()> {
    let page_bytes = interface
        .mode_sense()
        .long_lba_accepted(true)
        .page_code(mode::POWER_CONDITION_PAGE_CODE)
        // need atlease 2 bytes for length info
        .allocation_length(2)
        .issue_10()?;

    let page = PageWrapper::<PowerConditionPage>::from_bytes(
        mode::HeaderType::Long,
        mode::DescriptorType::Long,
        &page_bytes,
    );

    let page_bytes = interface
        .mode_sense()
        .long_lba_accepted(true)
        .page_code(mode::POWER_CONDITION_PAGE_CODE)
        .allocation_length(page.header.required_allocation_length() as u16)
        .issue_10()?;

    let page = PageWrapper::<PowerConditionPage>::from_bytes(
        mode::HeaderType::Long,
        mode::DescriptorType::Long,
        &page_bytes,
    );

    println!("{:#?}", page);

    Ok(())
}

fn print_all_modes(interface: &Scsi) -> scsir::Result<()> {
    let page_bytes = interface
        .mode_sense()
        .long_lba_accepted(true)
        .page_code(0x3F)
        .subpage_code(0xFF)
        // need atlease 2 bytes for length info
        .allocation_length(2)
        .issue_10()?;

    let page = PageWrapper::<PageHeaderStorage>::from_bytes(
        mode::HeaderType::Long,
        mode::DescriptorType::Long,
        &page_bytes,
    );

    let bytes = interface
        .mode_sense()
        .long_lba_accepted(true)
        .page_code(0x3F)
        .subpage_code(0xFF)
        .allocation_length(page.header.required_allocation_length() as u16)
        .issue_10()?;

    let (mode_header, bytes) = HeaderStorage::from_bytes(mode::HeaderType::Long, &bytes);
    println!("mode_header: {:#?}", mode_header);

    let block_descriptor_length = mode_header.block_descriptor_length() as usize;
    let (mut descriptor_bytes, mut bytes) =
        bytes.split_at(usize::min(block_descriptor_length, bytes.len()));

    println!("descriptors:");
    while !descriptor_bytes.is_empty() {
        let descriptor;
        (descriptor, descriptor_bytes) =
            DescriptorStorage::from_bytes(mode::DescriptorType::Long, descriptor_bytes);

        println!("{:#?}", descriptor);
    }

    println!("pages:");
    while !bytes.is_empty() {
        let (page_header, _) = PageHeaderStorage::from_bytes(bytes);
        match (page_header.page_code(), page_header.subpage_code()) {
            (mode::APPLICATION_TAG_PAGE_CODE, mode::APPLICATION_TAG_SUBPAGE_CODE) => {
                bytes = print_page::<mode::ApplicationTagPage>(bytes);
            }
            (mode::BACKGROUND_CONTROL_PAGE_CODE, mode::BACKGROUND_CONTROL_SUBPAGE_CODE) => {
                bytes = print_page::<mode::BackgroundControlPage>(bytes);
            }
            (
                mode::BACKGROUND_OPERATION_CONTROL_PAGE_CODE,
                mode::BACKGROUND_OPERATION_CONTROL_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::BackgroundOperationControlPage>(bytes);
            }
            (mode::CACHING_PAGE_CODE, mode::CACHING_SUBPAGE_CODE) => {
                bytes = print_page::<mode::CachingPage>(bytes);
            }
            (
                mode::COMMAND_DURATION_LIMIT_PAGE_CODE,
                mode::COMMAND_DURATION_LIMIT_A_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::CommandDurationLimitPage>(bytes);
            }
            (
                mode::COMMAND_DURATION_LIMIT_PAGE_CODE,
                mode::COMMAND_DURATION_LIMIT_B_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::CommandDurationLimitPage>(bytes);
            }
            (mode::CONTROL_EXTENSION_PAGE_CODE, mode::CONTROL_EXTENSION_SUBPAGE_CODE) => {
                bytes = print_page::<mode::ControlExtensionPage>(bytes);
            }
            (mode::CONTROL_PAGE_CODE, mode::CONTROL_SUBPAGE_CODE) => {
                bytes = print_page::<mode::ControlPage>(bytes);
            }
            (
                mode::DISCONNECT_RECONNECT_SAS_PAGE_CODE,
                mode::DISCONNECT_RECONNECT_SAS_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::DisconnectReconnectSasPage>(bytes);
            }
            (mode::ENHANCED_PHY_CONTROL_PAGE_CODE, mode::ENHANCED_PHY_CONTROL_SUBPAGE_CODE) => {
                bytes = print_page::<mode::EnhancedPhyControlPage>(bytes);
            }
            (
                mode::INFORMATIONAL_EXCEPTIONS_CONTROL_PAGE_CODE,
                mode::INFORMATIONAL_EXCEPTIONS_CONTROL_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::InformationalExceptionsControlPage>(bytes);
            }
            (
                mode::IO_ADVICE_HINTS_GROUPING_PAGE_CODE,
                mode::IO_ADVICE_HINTS_GROUPING_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::IoAdviceHintsGroupingPage>(bytes);
            }
            (
                mode::LOGICAL_BLOCK_PROVISIONING_PAGE_CODE,
                mode::LOGICAL_BLOCK_PROVISIONING_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::LogicalBlockProvisioningPage>(bytes);
            }
            (
                mode::LOGICAL_UNIT_CONTROL_SAS_PAGE_CODE,
                mode::LOGICAL_UNIT_CONTROL_SAS_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::LogicalUnitControlSasPage>(bytes);
            }
            (
                mode::PHY_CONTROL_AND_DISCOVER_PAGE_CODE,
                mode::PHY_CONTROL_AND_DISCOVER_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::PhyControlAndDiscoverPage>(bytes);
            }
            (mode::POWER_CONDITION_PAGE_CODE, mode::POWER_CONDITION_SUBPAGE_CODE) => {
                bytes = print_page::<mode::PowerConditionPage>(bytes);
            }
            (mode::POWER_CONSUMPTION_PAGE_CODE, mode::POWER_CONSUMPTION_SUBPAGE_CODE) => {
                bytes = print_page::<mode::PowerConsumptionPage>(bytes);
            }
            (
                mode::PROTOCOL_SPECIFIC_PORT_SAS_PAGE_CODE,
                mode::PROTOCOL_SPECIFIC_PORT_SAS_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::ProtocolSpecificPortSasPage>(bytes);
            }
            (
                mode::READ_WRITE_ERROR_RECOVERY_PAGE_CODE,
                mode::READ_WRITE_ERROR_RECOVERY_SUBPAGE_CODE,
            ) => {
                bytes = print_page::<mode::ReadWriteErrorRecoveryPage>(bytes);
            }
            (mode::SHARED_PORT_CONTROL_PAGE_CODE, mode::SHARED_PORT_CONTROL_SUBPAGE_CODE) => {
                bytes = print_page::<mode::SharedPortControlPage>(bytes);
            }
            (mode::VERIFY_ERROR_RECOVERY_PAGE_CODE, mode::VERIFY_ERROR_RECOVERY_SUBPAGE_CODE) => {
                bytes = print_page::<mode::VerifyErrorRecoveryPage>(bytes);
            }

            _ => {
                bytes = print_page::<mode::GeneralPage>(bytes);
            }
        }
    }

    Ok(())
}

fn print_page<P: ModePage + Debug>(bytes: &[u8]) -> &[u8] {
    let (page, bytes) = P::from_bytes(bytes);
    println!("{:#?}", page);
    bytes
}
