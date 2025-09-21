use base64::prelude::*;
use heapless::{String, Vec};
use serde::{Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct BcsData<T: serde::Serialize> {
    data: T,
}

impl<T: serde::Serialize> BcsData<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }

    pub fn as_bcs_bytes(&self) -> Result<alloc::vec::Vec<u8>, bcs::Error> {
        bcs::to_bytes(&self.data)
    }

    pub fn as_base64_string<const N: usize>(&self) -> Result<String<N>, EncodingError> {
        let bcs_bytes = self
            .as_bcs_bytes()
            .map_err(|_| EncodingError::SerializationFailed)?;

        let mut buf = [0u8; N];
        let size = BASE64_STANDARD
            .encode_slice(&bcs_bytes, &mut buf)
            .map_err(|_| EncodingError::TooLong)?;

        let vec_data = Vec::from_slice(&buf[..size]).map_err(|_| EncodingError::InvalidData)?;
        let result = String::from_utf8(vec_data).map_err(|_| EncodingError::TooLong)?;

        Ok(result)
    }
}

impl<T: serde::Serialize> Serialize for BcsData<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let base64_str: String<4096> = self
            .as_base64_string()
            .map_err(|_| serde::ser::Error::custom("Failed to encode transaction"))?;

        serializer.serialize_str(base64_str.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Base64Signature {
    bytes: Vec<u8, 128>,
}

impl Base64Signature {
    pub fn new(signature_bytes: &[u8]) -> Result<Self, EncodingError> {
        let bytes = Vec::from_slice(signature_bytes).map_err(|_| EncodingError::TooLong)?;

        Ok(Self { bytes })
    }

    pub fn as_base64_string<const N: usize>(&self) -> Result<String<N>, EncodingError> {
        let mut buf = [0u8; N];
        let size = BASE64_STANDARD
            .encode_slice(&self.bytes, &mut buf)
            .map_err(|_| EncodingError::TooLong)?;

        let vec_data = Vec::from_slice(&buf[..size]).map_err(|_| EncodingError::InvalidData)?;
        let result = String::from_utf8(vec_data).map_err(|_| EncodingError::TooLong)?;

        Ok(result)
    }
}

impl Serialize for Base64Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let base64_str: String<512> = self
            .as_base64_string()
            .map_err(|_| serde::ser::Error::custom("Failed to encode signature"))?;

        serializer.serialize_str(base64_str.as_str())
    }
}

#[derive(Debug)]
pub enum EncodingError {
    SerializationFailed,
    TooLong,
    InvalidData,
}
