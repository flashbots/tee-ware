//! TSS Serde - Serde-like binary encoding for TPM/TSS structures
//!
//! This crate provides procedural macros for encoding and decoding Rust structs
//! to/from the binary format used by TPM (Trusted Platform Module) via TSS.

// Re-export the derive macros
pub use tss_serde_derive::{TssDeserialize, TssSerialize};

/// A reader that consumes bytes from a buffer, tracking position automatically
#[derive(Debug)]
pub struct TssReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> TssReader<'a> {
    /// Create a new TssReader from a byte slice
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// Get the current position in the buffer
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the remaining bytes count
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    /// Check if we have at least `count` bytes remaining
    pub fn has_remaining(&self, count: usize) -> bool {
        self.remaining() >= count
    }

    /// Read a single byte
    pub fn read_u8(&mut self) -> Result<u8, TssError> {
        if !self.has_remaining(1) {
            return Err(TssError::InsufficientData);
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    /// Read exactly `count` bytes into a new Vec
    pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, TssError> {
        if !self.has_remaining(count) {
            return Err(TssError::InsufficientData);
        }
        let bytes = self.data[self.position..self.position + count].to_vec();
        self.position += count;
        Ok(bytes)
    }

    /// Read exactly `N` bytes into a fixed-size array
    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], TssError> {
        if !self.has_remaining(N) {
            return Err(TssError::InsufficientData);
        }
        let mut array = [0u8; N];
        array.copy_from_slice(&self.data[self.position..self.position + N]);
        self.position += N;
        Ok(array)
    }

    /// Get a slice of the remaining data without consuming it
    pub fn peek_remaining(&self) -> &[u8] {
        &self.data[self.position..]
    }

    /// Skip `count` bytes
    pub fn skip(&mut self, count: usize) -> Result<(), TssError> {
        if !self.has_remaining(count) {
            return Err(TssError::InsufficientData);
        }
        self.position += count;
        Ok(())
    }
}

/// Trait for types that can be serialized to TSS binary format
pub trait TssSerialize {
    fn to_tss_bytes(&self) -> Vec<u8>;
}

/// Trait for types that can be deserialized from TSS binary format
pub trait TssDeserialize: Sized {
    /// Deserialize from raw bytes (convenience method)
    fn from_tss_bytes(bytes: &[u8]) -> Result<Self, TssError> {
        let mut reader = TssReader::new(bytes);
        Self::from_tss_reader(&mut reader)
    }

    /// Deserialize from a TssReader (primary method)
    fn from_tss_reader(reader: &mut TssReader) -> Result<Self, TssError>;
}

/// Errors that can occur during TSS serialization/deserialization
#[derive(Debug, Clone, PartialEq)]
pub enum TssError {
    InsufficientData,
    InvalidFormat,
    Custom(String),
}

impl std::fmt::Display for TssError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TssError::InsufficientData => write!(f, "Insufficient data for deserialization"),
            TssError::InvalidFormat => write!(f, "Invalid TSS format"),
            TssError::Custom(msg) => write!(f, "TSS error: {}", msg),
        }
    }
}

impl std::error::Error for TssError {}

// Implement TssSerialize/TssDeserialize for basic types
impl TssSerialize for u8 {
    fn to_tss_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl TssSerialize for u16 {
    fn to_tss_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl TssSerialize for u32 {
    fn to_tss_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl TssSerialize for u64 {
    fn to_tss_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl TssDeserialize for u8 {
    fn from_tss_reader(reader: &mut TssReader) -> Result<Self, TssError> {
        reader.read_u8()
    }
}

impl TssDeserialize for u16 {
    fn from_tss_reader(reader: &mut TssReader) -> Result<Self, TssError> {
        let bytes = reader.read_array::<2>()?;
        Ok(u16::from_be_bytes(bytes))
    }
}

impl TssDeserialize for u32 {
    fn from_tss_reader(reader: &mut TssReader) -> Result<Self, TssError> {
        let bytes = reader.read_array::<4>()?;
        Ok(u32::from_be_bytes(bytes))
    }
}

impl TssDeserialize for u64 {
    fn from_tss_reader(reader: &mut TssReader) -> Result<Self, TssError> {
        let bytes = reader.read_array::<8>()?;
        Ok(u64::from_be_bytes(bytes))
    }
}

impl TssDeserialize for bool {
    fn from_tss_reader(reader: &mut TssReader) -> Result<Self, TssError> {
        let bytes = reader.read_array::<1>()?;
        match bytes[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(TssError::InvalidFormat),
        }
    }
}

// Implementation for fixed-size arrays
impl<const N: usize> TssSerialize for [u8; N] {
    fn to_tss_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl<const N: usize> TssDeserialize for [u8; N] {
    fn from_tss_reader(reader: &mut TssReader) -> Result<Self, TssError> {
        reader.read_array::<N>()
    }
}

impl<T> TssDeserialize for Vec<T>
where
    T: TssDeserialize,
{
    fn from_tss_reader(reader: &mut TssReader) -> Result<Self, TssError> {
        // Read length as u32
        let length = u32::from_tss_reader(reader)? as usize;

        // Read each element
        let mut vec = Vec::with_capacity(length);
        for _ in 0..length {
            vec.push(T::from_tss_reader(reader)?);
        }

        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestStruct {
        a: u16,
        b: u32,
        c: [u8; 4],
    }

    impl TssDeserialize for TestStruct {
        fn from_tss_reader(reader: &mut TssReader) -> Result<Self, TssError> {
            let a = u16::from_tss_reader(reader)?;
            let b = u32::from_tss_reader(reader)?;
            let c = <[u8; 4]>::from_tss_reader(reader)?;

            Ok(TestStruct { a, b, c })
        }
    }

    #[test]
    fn test_tss_reader() {
        let data = vec![0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0xAA, 0xBB, 0xCC, 0xDD];
        let mut reader = TssReader::new(&data);

        let a = u16::from_tss_reader(&mut reader).unwrap();
        let b = u32::from_tss_reader(&mut reader).unwrap();
        let c = <[u8; 4]>::from_tss_reader(&mut reader).unwrap();

        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(c, [0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(reader.position(), 10);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn test_struct_deserialize() {
        let data = vec![0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0xAA, 0xBB, 0xCC, 0xDD];
        let test_struct = TestStruct::from_tss_bytes(&data).unwrap();

        assert_eq!(test_struct.a, 1);
        assert_eq!(test_struct.b, 2);
        assert_eq!(test_struct.c, [0xAA, 0xBB, 0xCC, 0xDD]);
    }
}
