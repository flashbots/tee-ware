use std::io::{Read, Write};
use std::net::TcpStream;
use std::net::ToSocketAddrs;

use tss_serde::TssDeserialize;

use crate::primitives::ResponseHeader;
use crate::Transport;

pub struct TcpTransport {
    stream: TcpStream,
}

impl TcpTransport {
    fn connect<A: ToSocketAddrs>(addr: A) -> eyre::Result<Self> {
        Self::reset_platform()?;

        let stream = TcpStream::connect(addr)?;
        Ok(Self { stream })
    }

    fn reset_platform() -> eyre::Result<()> {
        let mut platform_stream = TcpStream::connect("localhost:2322")?;

        // Power Off
        platform_stream.write_all(&[0x00, 0x00, 0x00, 0x02])?;
        platform_stream.flush()?;
        let mut response = [0u8; 4];
        platform_stream.read_exact(&mut response)?;
        println!("Power Off response: {:02X?}", response);

        // Power On
        platform_stream.write_all(&[0x00, 0x00, 0x00, 0x01])?;
        platform_stream.flush()?;
        let mut response = [0u8; 4];
        platform_stream.read_exact(&mut response)?;
        println!("Power On response: {:02X?}", response);

        // NV On
        platform_stream.write_all(&[0x00, 0x00, 0x00, 0x0B])?;
        platform_stream.flush()?;
        platform_stream.read_exact(&mut response)?;
        println!("NV On response: {:02X?}", response);

        drop(platform_stream);

        Ok(())
    }
}

impl Default for TcpTransport {
    fn default() -> Self {
        Self::connect("localhost:2321").unwrap()
    }
}

impl Transport for TcpTransport {
    fn send_command(&mut self, command: &[u8]) -> eyre::Result<(ResponseHeader, Vec<u8>)> {
        // 1. Command type: TPM_SEND_COMMAND
        self.stream.write_all(&[0x00, 0x00, 0x00, 0x08])?; // TPM_SEND_COMMAND

        // 2. Locality
        self.stream.write_all(&[0x00])?; // Locality 0

        // 3. Length-prefixed TPM packet (big-endian length)
        let tpm_len = (command.len() as u32).to_be_bytes();
        self.stream.write_all(&tpm_len)?;

        // 4. TPM packet
        self.stream.write_all(&command)?;
        self.stream.flush()?;

        // 5. Wait for the response
        #[allow(unused_assignments)]
        let mut response_len = 0;
        loop {
            let mut len_bytes = [0u8; 4];
            self.stream.read_exact(&mut len_bytes)?;
            response_len = u32::from_be_bytes(len_bytes);

            if response_len != 0 {
                break;
            }
        }

        // Read TPM response
        let mut tpm_response = vec![0u8; response_len as usize];
        self.stream.read_exact(&mut tpm_response)?;

        let header = ResponseHeader::from_tss_bytes(&tpm_response)?;
        if header.response_code != 0 {
            return Err(eyre::eyre!("Command failed"));
        }

        // Try to read the rest of the body response
        let body_response = &tpm_response[10..];
        Ok((header, body_response.to_vec()))
    }
}
