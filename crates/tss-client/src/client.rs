use crate::primitives::{self, CapabilitiesResponse, Empty, RawResponse, ResponseHeader};
use tss_serde::{TssDeserialize, TssSerialize};

/// A trait for abstracting the underlying transport mechanism used to communicate with a TPM.
///
/// This trait allows the TSS client to work with different transport implementations
/// (e.g., TCP, USB, SPI) without being tightly coupled to any specific transport method.
pub trait Transport {
    fn send_command(&mut self, command: &[u8]) -> eyre::Result<(ResponseHeader, Vec<u8>)>;
}

/// A TSS (TPM Software Stack) client for communicating with Trusted Platform Modules (TPMs).
///
/// The `TssClient` provides a high-level interface for TPM operations, handling command
/// serialization, transport communication, and response deserialization. It is generic
/// over the transport mechanism, allowing it to work with different communication channels.
///
/// # Type Parameters
///
/// * `T` - The transport type that implements the [`Transport`] trait
///
/// # Examples
///
/// ```rust
/// use your_crate::{TssClient, TcpTransport};
///
/// let mut client = TssClient::new(TcpTransport::default());
/// client.startup(primitives::startup_type::CLEAR)?;
/// ```
pub struct TssClient<T> {
    transport: T,
}

impl<T> TssClient<T>
where
    T: Transport,
{
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn startup(&mut self, startup_type: u16) -> eyre::Result<()> {
        let command = primitives::StartupCommand { startup_type };
        let _ = self.run_command::<Empty>(primitives::commands::STARTUP, command)?;
        Ok(())
    }

    pub fn get_capabilities(
        &mut self,
        capability: u32,
        property: u32,
        property_count: u32,
    ) -> eyre::Result<CapabilitiesResponse> {
        let result: CapabilitiesResponse = self.run_command(
            primitives::commands::GET_CAPABILITY,
            primitives::GetCapabilityCommand {
                capability,
                property,
                property_count,
            },
        )?;
        Ok(result)
    }

    pub fn read_pcr(&mut self, input: primitives::ReadPcrCommand) -> eyre::Result<RawResponse> {
        let result: RawResponse = self.run_command(primitives::commands::READ_PCR, input)?;
        Ok(result)
    }

    pub fn run_command<TS: TssDeserialize>(
        &mut self,
        command_code: u32,
        command_body: impl TssSerialize,
    ) -> eyre::Result<TS> {
        let body = command_body.to_tss_bytes();

        let header = primitives::CommandHeader {
            tag: primitives::tags::NO_SESSIONS,
            command_code: command_code,
            length: 10 + body.len() as u32,
        };
        let header_bytes = header.to_tss_bytes();
        let input = [header_bytes, body].concat();

        let (_header, body_response) = self.transport.send_command(&input)?;

        let result = TS::from_tss_bytes(&body_response)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tcp_transport::TcpTransport;

    #[test]
    fn simple_test() -> eyre::Result<()> {
        let mut tss_client = TssClient::new(TcpTransport::default());
        let _ = tss_client.startup(primitives::startup_type::CLEAR)?;

        let result = tss_client.get_capabilities(
            primitives::capabilities::TPM_PROPERTIES,
            primitives::properties::FAMILY_INDICATOR,
            1,
        )?;

        println!("result: {:?}", result);

        Ok(())
    }
}
