use tss_serde::{TssDeserialize, TssError, TssSerialize};

pub mod commands {
    pub const STARTUP: u32 = 0x00000144;
    pub const GET_CAPABILITY: u32 = 0x0000017A;
    pub const READ_PCR: u32 = 0x0000017E;
}

pub mod capabilities {
    pub const TPM_PROPERTIES: u32 = 0x00000006;
    pub const COMMANDS: u32 = 0x00000002;
    pub const HANDLES_TRANSIENT: u32 = 0x00000001;
}

pub mod properties {
    pub const FAMILY_INDICATOR: u32 = 0x00000100;
    pub const LEVEL: u32 = 0x00000101;
    pub const REVISION: u32 = 0x00000102;
    pub const MANUFACTURER: u32 = 0x00000105;
}

pub mod tags {
    pub const NULL: u16 = 0x8000;
    pub const NO_SESSIONS: u16 = 0x8001;
    pub const SESSIONS: u16 = 0x8002;
    pub const ATTES_CERTIFY: u16 = 0x8017;
    pub const ATTES_QUOTE: u16 = 0x8018;
    pub const ATTES_CREATION: u16 = 0x801a;
    pub const AUTH_SECRET: u16 = 0x8023;
    pub const HASH_CHECK: u16 = 0x8024;
}

pub mod startup_type {
    pub const CLEAR: u16 = 0;
    pub const STATE: u16 = 1;
}

#[derive(TssSerialize)]
pub struct StartupCommand {
    pub startup_type: u16,
}

#[derive(TssSerialize)]
pub struct CommandHeader {
    pub tag: u16,
    pub length: u32,
    pub command_code: u32,
}

#[derive(TssSerialize)]
pub struct GetCapabilityCommand {
    pub capability: u32,
    pub property: u32,
    pub property_count: u32,
}

//mod response_codes {
//    const SUCCESS: u32 = 0x00000000;
//    const RETRY: u32 = 0x0000922;
//}

#[derive(TssDeserialize, Debug)]
pub struct ResponseHeader {
    pub tag: u16,
    pub size: u32,
    pub response_code: u32,
}

pub struct RawResponse {
    pub bytes: Vec<u8>,
}

impl TssDeserialize for RawResponse {
    fn from_tss_reader(reader: &mut tss_serde::TssReader) -> Result<Self, TssError> {
        let bytes = Vec::from_tss_reader(reader)?;
        Ok(RawResponse { bytes })
    }
}

pub struct Empty {}

impl TssDeserialize for Empty {
    fn from_tss_reader(reader: &mut tss_serde::TssReader) -> Result<Self, TssError> {
        if reader.remaining() != 0 {
            return Err(TssError::InvalidFormat);
        }
        Ok(Self {})
    }
}

#[derive(Debug)]
pub enum Capabilities {
    Handles(Vec<u32>),
    TaggedProperties(Vec<TaggedProperty>),
}

#[derive(Debug)]
pub struct CapabilitiesResponse {
    pub capabilities: Capabilities,
    pub more_data: bool,
}

#[derive(Debug, TssDeserialize)]
pub struct TaggedProperty {
    pub tag: u32,
    pub value: u32,
}

impl TssDeserialize for CapabilitiesResponse {
    fn from_tss_reader(reader: &mut tss_serde::TssReader) -> Result<Self, TssError> {
        let more_data = bool::from_tss_reader(reader)?;
        let capability = u32::from_tss_reader(reader)?;

        let capabilities = match capability {
            1 => Capabilities::Handles(Vec::from_tss_reader(reader)?),
            6 => Capabilities::TaggedProperties(Vec::from_tss_reader(reader)?),
            _ => {
                panic!("Not implemented");
            }
        };

        Ok(CapabilitiesResponse {
            capabilities,
            more_data,
        })
    }
}

pub struct ReadPcrCommand {
    pub hash: u16,
    pub pcr_index: Vec<u32>,
}

impl TssSerialize for ReadPcrCommand {
    fn to_tss_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        if self.pcr_index.is_empty() {
            // If no PCRs selected, write count of 0
            buffer.extend_from_slice(&0u32.to_tss_bytes());
            return buffer;
        }

        // Write hash algorithm
        buffer.extend_from_slice(&self.hash.to_tss_bytes());

        // Write size of PCR select (3 bytes for up to 24 PCRs)
        const SIZE_OF_PCR_SELECT: u8 = 3;
        buffer.push(SIZE_OF_PCR_SELECT);

        // Create PCR bitmask
        let mut pcr_mask = [0u8; SIZE_OF_PCR_SELECT as usize];

        for &pcr_index in &self.pcr_index {
            if pcr_index >= 8 * SIZE_OF_PCR_SELECT as u32 {
                panic!(
                    "PCR index {} is out of range (exceeds maximum value {})",
                    pcr_index,
                    8 * SIZE_OF_PCR_SELECT as u32 - 1
                );
            }

            let byte_num = (pcr_index / 8) as usize;
            let bit_pos = pcr_index % 8;
            pcr_mask[byte_num] |= 1 << bit_pos;
        }

        // Write the PCR mask
        buffer.extend_from_slice(&pcr_mask);

        buffer
    }
}
