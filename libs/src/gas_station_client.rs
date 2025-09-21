use core::fmt::{Debug, Write};
use serde_bytes::deserialize;

use crate::encoding::{Base64Signature, BcsData};
use crate::json_client::{ClientError, JsonClient};
use crate::transaction_types;
use crate::transaction_types::IOTA_ADDRESS_LENGTH;
use defmt::Format;
use embedded_nal_async::{Dns, TcpConnect};
use heapless::{String, Vec};
use reqwless::client::HttpClient;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
pub enum ObjectIDError {
    InvalidHex,
    TooLong,
    WrongLength { expected: usize, actual: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Format)]
pub struct ObjectID([u8; IOTA_ADDRESS_LENGTH]);

impl Serialize for ObjectID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut str = heapless::String::<128>::new();
        write!(str, "0x{}", hex::encode(self.0))
            .map_err(|_| serde::ser::Error::custom("Invalid hex"))?;
        serializer.serialize_str(&str)
    }
}

impl<'de> Deserialize<'de> for ObjectID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = heapless::String::<128>::deserialize(deserializer)?;
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(&hex_str);
        let mut buffer = [0u8; IOTA_ADDRESS_LENGTH];

        hex::decode_to_slice(hex_str, &mut buffer)
            .map_err(|_| serde::de::Error::custom("Invalid hex"))?;

        Ok(ObjectID(buffer))
    }
}

impl ObjectID {
    pub fn from_hex(hex_str: &str) -> Result<Self, ObjectIDError> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let mut buf = [0u8; IOTA_ADDRESS_LENGTH];
        hex::decode_to_slice(hex_str, &mut buf).map_err(|_| ObjectIDError::InvalidHex)?;
        Ok(ObjectID(buf))
    }

    pub fn as_bytes(&self) -> [u8; IOTA_ADDRESS_LENGTH] {
        self.0
    }

    pub fn as_tx_object_id(&self) -> transaction_types::ObjectID {
        transaction_types::ObjectID::new(self.as_bytes())
    }
}

impl From<ObjectID> for transaction_types::ObjectID {
    fn from(val: ObjectID) -> Self {
        val.as_tx_object_id()
    }
}

#[derive(Debug)]
pub enum DigestError {
    InvalidBase58,
    TooLong,
    WrongLength,
}

#[derive(Debug, Clone, PartialEq, Eq, Format)]
pub struct Digest([u8; IOTA_ADDRESS_LENGTH]);

impl Digest {
    pub fn from_base58(b58_str: &str) -> Result<Self, DigestError> {
        let mut buffer = [0u8; IOTA_ADDRESS_LENGTH];
        let length = bs58::decode(b58_str)
            .onto(&mut buffer)
            .map_err(|_| DigestError::InvalidBase58)?;

        if length != IOTA_ADDRESS_LENGTH {
            return Err(DigestError::WrongLength);
        }

        Ok(Self(buffer))
    }

    pub fn as_bytes(&self) -> [u8; IOTA_ADDRESS_LENGTH] {
        self.0
    }

    pub fn as_base58(&self) -> Result<heapless::String<128>, DigestError> {
        let mut buffer = heapless::Vec::<u8, 128>::new();
        bs58::encode(self.0)
            .onto(buffer.as_mut_slice())
            .map_err(|_| DigestError::InvalidBase58)?;
        let str = heapless::String::from_utf8(buffer).map_err(|_| DigestError::InvalidBase58)?;
        Ok(str)
    }
}

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str = self
            .as_base58()
            .map_err(|_| serde::ser::Error::custom("Invalid base58"))?;
        serializer.serialize_str(&str)
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = heapless::String::<128>::deserialize(deserializer)?;

        let mut buffer = [0u8; IOTA_ADDRESS_LENGTH];
        let length = bs58::decode(hex_str)
            .onto(&mut buffer)
            .map_err(|_| serde::de::Error::custom("Invalid hex string"))?;

        if length != IOTA_ADDRESS_LENGTH {
            return Err(serde::de::Error::custom(
                "Must be IOTA_ADDRESS_LENGTH bytes",
            ));
        }

        Ok(Digest(buffer))
    }
}

impl From<Digest> for transaction_types::Digest {
    fn from(val: Digest) -> Self {
        transaction_types::Digest::new(val.as_bytes())
    }
}

#[derive(Debug, Serialize)]
pub struct ExecuteTxRequest {
    pub reservation_id: u32,
    pub tx_bytes: BcsData<crate::transaction_types::TransactionData>,
    pub user_sig: Base64Signature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Effects<T> {
    pub effects: T,
    pub error: Option<String<1024>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub status: heapless::String<64>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteTxResponse {
    #[serde(rename = "transactionDigest")]
    pub transaction_digest: Digest,
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReserveGasRequest {
    pub gas_budget: u64,
    pub reserve_duration_secs: u32,
}

#[derive(Debug, Deserialize)]
pub struct ObjectRef {
    #[serde(rename = "objectId")]
    pub object_id: ObjectID,
    pub version: u32,
    pub digest: Digest,
}

impl ObjectRef {
    pub fn as_tx_object_ref(&self) -> transaction_types::ObjectRef {
        (
            self.object_id.clone().into(),
            transaction_types::SequenceNumber::new(self.version as u64),
            self.digest.clone().into(),
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct RequestGasResponse {
    pub sponsor_address: ObjectID,
    pub reservation_id: u32,
    pub gas_coins: Vec<ObjectRef, 4>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasStationResult<T> {
    pub result: T,
    pub error: Option<String<1024>>,
}

pub struct GasStationClient<'a, TCP, DNS>
where
    TCP: TcpConnect + 'a,
    DNS: Dns + 'a,
{
    client: JsonClient<'a, TCP, DNS, 8192, 4096>,
    bearer_token: &'a str,
    base_url: &'a str,
}

impl<'a, TCP, DNS> GasStationClient<'a, TCP, DNS>
where
    TCP: TcpConnect + 'a,
    DNS: Dns + 'a,
{
    pub fn new(
        http_client: HttpClient<'a, TCP, DNS>,
        base_url: &'a str,
        bearer_token: &'a str,
    ) -> Self {
        Self {
            client: JsonClient::new(http_client),
            bearer_token,
            base_url,
        }
    }

    pub async fn reserve_gas(
        &mut self,
        gas_budget: u64,
        reserve_duration_secs: u32,
    ) -> Result<RequestGasResponse, ClientError> {
        let request = ReserveGasRequest {
            gas_budget,
            reserve_duration_secs,
        };

        let mut url = String::<128>::new();
        write!(url, "{}/v1/reserve_gas", self.base_url).unwrap();

        let headers = [("Authorization", self.bearer_token)];

        let result: Result<GasStationResult<RequestGasResponse>, ClientError> =
            self.client.post_json(&url, &request, &headers).await;

        match result {
            Ok(response) => Ok(response.result),
            Err(e) => Err(e),
        }
    }

    pub async fn execute_tx(
        &mut self,
        reservation_id: u32,
        tx_bytes: BcsData<crate::transaction_types::TransactionData>,
        user_sig: Base64Signature,
    ) -> Result<ExecuteTxResponse, ClientError> {
        let request = ExecuteTxRequest {
            reservation_id,
            tx_bytes,
            user_sig,
        };

        let mut url = String::<128>::new();
        write!(url, "{}/v1/execute_tx", self.base_url).unwrap();

        let headers = [("Authorization", self.bearer_token)];

        let result: Result<Effects<ExecuteTxResponse>, ClientError> =
            self.client.post_json(&url, &request, &headers).await;

        match result {
            Ok(response) => Ok(response.effects),
            Err(e) => Err(e),
        }
    }
}
