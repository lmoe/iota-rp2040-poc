use core::fmt::{Debug, Write};
use embedded_nal_async::{Dns, TcpConnect};
use heapless::String;
use reqwless::client::HttpClient;
use reqwless::headers::ContentType;
use reqwless::request::{Method, RequestBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ClientError {
    HttpError(String<512>),
    ParseError(String<512>),
    JsonRpcError(String<512>),
    SerializationError(String<512>),
}

impl ClientError {
    fn from_serde_error<T: core::fmt::Display>(error: T) -> Self {
        let mut message = String::<512>::new();
        let _ = write!(message, "Serialization error: {}", error);
        ClientError::SerializationError(message)
    }

    fn from_http_error<T: core::fmt::Debug>(error: T, context: &str) -> Self {
        let mut message = String::<512>::new();
        let _ = write!(message, "{}: {:?}", context, error);
        ClientError::HttpError(message)
    }

    fn from_parse_error<T: core::fmt::Display>(error: T, context: &str) -> Self {
        let mut message = String::<512>::new();
        let _ = write!(message, "{}: {}", context, error);
        ClientError::ParseError(message)
    }
}

fn log_bytes_as_string(bytes: &[u8], prefix: &str) {
    if let Ok(s) = core::str::from_utf8(bytes) {
        #[cfg(feature = "defmt")]
        {
            defmt::info!("UTF-8 conversion OK, string length: {}", s.len());

            if s.len() > 10240 {
                let truncated = &s[..10240];
                defmt::info!("\n\n{} (truncated): {}\n\n", prefix, truncated);
            } else {
                defmt::info!("\n\n{}: {}\n\n", prefix, s);
            }
        }
        #[cfg(feature = "std")]
        print!("\n\n{}: {}\n\n", prefix, s);
    } else {
        #[cfg(feature = "defmt")]
        defmt::warn!("{}: <invalid UTF-8>", prefix);
        #[cfg(feature = "std")]
        print!("{}: <invalid UTF-8>\n\n", prefix);
    }
}

pub struct JsonClient<'a, TCP, DNS, const TX_BUFFER_SIZE: usize, const RX_BUFFER_SIZE: usize>
where
    TCP: TcpConnect + 'a,
    DNS: Dns + 'a,
{
    http_client: HttpClient<'a, TCP, DNS>,
}

impl<'a, TCP, DNS, const TX_BUFFER_SIZE: usize, const RX_BUFFER_SIZE: usize>
    JsonClient<'a, TCP, DNS, TX_BUFFER_SIZE, RX_BUFFER_SIZE>
where
    TCP: TcpConnect + 'a,
    DNS: Dns + 'a,
{
    pub fn new(http_client: HttpClient<'a, TCP, DNS>) -> Self {
        Self { http_client }
    }

    pub async fn post_json<Req, Resp>(
        &mut self,
        url: &str,
        request_body: &Req,
        headers: &[(&'a str, &'a str)],
    ) -> Result<Resp, ClientError>
    where
        Req: Serialize,
        Resp: for<'de> Deserialize<'de>,
    {
        let mut tx_buffer = [0u8; TX_BUFFER_SIZE];
        let mut rx_buffer = [0u8; RX_BUFFER_SIZE];

        let json_length = serde_json_core::ser::to_slice(request_body, &mut tx_buffer)
            .map_err(ClientError::from_serde_error)?;

        log_bytes_as_string(&tx_buffer, "Request");

        let mut request = self
            .http_client
            .request(Method::POST, url)
            .await
            .unwrap()
            .headers(headers)
            .body(&tx_buffer[..json_length])
            .content_type(ContentType::ApplicationJson);

        let response = request.send(&mut rx_buffer).await.unwrap();
        let response_length = response
            .content_length
            .expect("could not determine response length");

        let body_bytes = response
            .body()
            .read_to_end()
            .await
            .map_err(|e| ClientError::from_http_error(e, "Failed to read response body"))?;

        let body_bytes = &body_bytes[..response_length];

        log_bytes_as_string(body_bytes, "Response");

        let parsed_response: Resp = serde_json_core::de::from_slice(body_bytes)
            .map_err(|e| ClientError::from_parse_error(e, "Failed to parse response"))
            .map(|(response, _)| response)?;

        Ok(parsed_response)
    }
}
