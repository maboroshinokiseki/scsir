#[derive(Debug)]
pub enum ProtocolIdentifier {
    None,
    FibreChannel,
    Ssa,
    IEEE1394,
    RemoteDirectMemoryAccess,
    InternetScsi,
    SasSerialScsiProtocol,
    Other(u8),
}