use core::fmt::Write;
use ed25519_dalek::Signer;

const MAX_DATA_SIZE: usize = 5000;
const MESSAGE_SIZE: usize = 3 + MAX_DATA_SIZE;

const ED25519_SIGNATURE_SIZE: usize =
    ed25519_dalek::SIGNATURE_LENGTH + ed25519_dalek::PUBLIC_KEY_LENGTH + 1;

const KEY_SCHEME_ED25519: u8 = 0;

pub struct Crypto {
    verifying_key: ed25519_dalek::VerifyingKey,
    signing_key: ed25519_dalek::SigningKey,
}

impl Crypto {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let secret_key = ed25519_dalek::SecretKey::from(seed);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_key);
        let verifying_key = signing_key.verifying_key();

        Self {
            verifying_key,
            signing_key,
        }
    }

    /// This signs data with the default Intent (3x 0 bytes)
    pub fn sign(&self, data: &[u8]) -> [u8; ED25519_SIGNATURE_SIZE] {
        if data.len() > MAX_DATA_SIZE {
            core::panic!("Data too large");
        }

        let mut message = [0u8; MESSAGE_SIZE];
        message[3..3 + data.len()].copy_from_slice(data);

        let hash = blake2b_simd::Params::new()
            .hash_length(32)
            .hash(&message[..3 + data.len()]);

        let signature = self.signing_key.sign(hash.as_bytes());

        let mut result = [0u8; ED25519_SIGNATURE_SIZE];

        result[0] = KEY_SCHEME_ED25519;
        result[1..1 + ed25519_dalek::SIGNATURE_LENGTH].copy_from_slice(&signature.to_bytes());
        result[1 + ed25519_dalek::SIGNATURE_LENGTH..]
            .copy_from_slice(&self.verifying_key.to_bytes());

        result
    }

    pub fn public_address(&self) -> blake2b_simd::Hash {
        blake2b_simd::Params::new()
            .hash_length(32)
            .hash(self.verifying_key.as_bytes())
    }

    pub fn public_address_hex_string(&self) -> heapless::String<256> {
        let mut pub_address = heapless::String::<256>::new();
        write!(pub_address, "0x{}", self.public_address().to_hex().as_str())
            .expect("Can't write address to string");
        pub_address
    }

    pub fn verifying_key(&self) -> &ed25519_dalek::VerifyingKey {
        &self.verifying_key
    }

    pub fn signing_key(&self) -> &ed25519_dalek::SigningKey {
        &self.signing_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crypto_signing() {
        let seed = [0; 32];
        let kp = Crypto::from_seed(seed);

        let expected_tx_bytes = [
            0, 0, 3, 1, 0, 159, 16, 133, 122, 159, 135, 45, 23, 40, 140, 61, 243, 102, 240, 117,
            93, 121, 146, 55, 191, 145, 123, 49, 93, 215, 190, 5, 246, 186, 79, 101, 199, 2, 0, 0,
            0, 0, 0, 0, 0, 32, 186, 128, 247, 107, 53, 48, 75, 82, 7, 165, 148, 125, 118, 84, 34,
            198, 34, 208, 112, 146, 154, 199, 80, 118, 174, 0, 150, 135, 178, 57, 210, 81, 0, 5, 4,
            1, 2, 3, 4, 0, 32, 104, 157, 174, 47, 119, 176, 72, 220, 192, 142, 20, 215, 49, 4, 234,
            20, 34, 43, 91, 225, 76, 195, 31, 52, 161, 106, 18, 33, 249, 68, 193, 227, 3, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 6, 111, 112, 116, 105, 111, 110, 4, 115, 111, 109, 101, 1, 7, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 4, 99, 111,
            105, 110, 4, 67, 111, 105, 110, 1, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 4, 105, 111, 116, 97, 4, 73, 79, 84,
            65, 0, 1, 1, 0, 0, 0, 177, 169, 228, 154, 57, 202, 97, 67, 123, 93, 228, 169, 240, 88,
            167, 48, 111, 81, 207, 55, 123, 247, 55, 79, 250, 53, 254, 21, 217, 114, 9, 54, 6, 97,
            110, 99, 104, 111, 114, 15, 115, 116, 97, 114, 116, 95, 110, 101, 119, 95, 99, 104, 97,
            105, 110, 0, 2, 1, 1, 0, 2, 0, 0, 1, 1, 2, 1, 0, 1, 2, 0, 104, 157, 174, 47, 119, 176,
            72, 220, 192, 142, 20, 215, 49, 4, 234, 20, 34, 43, 91, 225, 76, 195, 31, 52, 161, 106,
            18, 33, 249, 68, 193, 227, 1, 79, 244, 153, 214, 156, 202, 32, 208, 123, 245, 9, 36,
            27, 29, 155, 99, 48, 165, 29, 95, 155, 153, 190, 175, 177, 55, 81, 147, 162, 56, 95,
            195, 2, 0, 0, 0, 0, 0, 0, 0, 32, 222, 21, 62, 84, 191, 178, 118, 179, 107, 202, 153,
            233, 66, 155, 176, 234, 127, 34, 26, 71, 52, 97, 163, 23, 136, 83, 149, 57, 44, 143,
            154, 225, 104, 157, 174, 47, 119, 176, 72, 220, 192, 142, 20, 215, 49, 4, 234, 20, 34,
            43, 91, 225, 76, 195, 31, 52, 161, 106, 18, 33, 249, 68, 193, 227, 232, 3, 0, 0, 0, 0,
            0, 0, 128, 150, 152, 0, 0, 0, 0, 0, 0,
        ];

        let expected_signature = [
            0, 212, 247, 154, 25, 207, 162, 141, 244, 59, 104, 172, 18, 64, 205, 66, 135, 116, 123,
            11, 223, 180, 187, 33, 242, 65, 34, 205, 228, 41, 57, 38, 46, 15, 88, 105, 72, 200, 23,
            76, 202, 121, 195, 242, 206, 234, 74, 40, 212, 235, 27, 231, 72, 148, 215, 174, 65,
            171, 110, 18, 59, 50, 123, 83, 5, 59, 106, 39, 188, 206, 182, 164, 45, 98, 163, 168,
            208, 42, 111, 13, 115, 101, 50, 21, 119, 29, 226, 67, 166, 58, 192, 72, 161, 139, 89,
            218, 41,
        ];

        /*
        let expected_hash = [ 102, 190, 162, 202, 227, 156, 171, 230, 94, 111, 99, 3, 219, 125, 36, 76, 227, 226, 62, 100, 148, 24, 216, 212, 101, 28, 33, 207, 106, 113, 229, 35 ];
        let expected_public_key = [ 59, 106, 39, 188, 206, 182, 164, 45, 98, 163, 168, 208, 42, 111, 13, 115, 101, 50, 21, 119, 29, 226, 67, 166, 58, 192, 72, 161, 139, 89, 218, 41 ];
        let expected_signed_msg = [ 212, 247, 154, 25, 207, 162, 141, 244, 59, 104, 172, 18, 64, 205, 66, 135, 116, 123, 11, 223, 180, 187, 33, 242, 65, 34, 205, 228, 41, 57, 38, 46, 15, 88, 105, 72, 200, 23, 76, 202, 121, 195, 242, 206, 234, 74, 40, 212, 235, 27, 231, 72, 148, 215, 174, 65, 171, 110, 18, 59, 50, 123, 83, 5 ];
        let expected_public_key = [ 59, 106, 39, 188, 206, 182, 164, 45, 98, 163, 168, 208, 42, 111, 13, 115, 101, 50, 21, 119, 29, 226, 67, 166, 58, 192, 72, 161, 139, 89, 218, 41 ];
        */

        let sig = kp.sign(&expected_tx_bytes);

        assert_eq!(sig.as_slice(), expected_signature);
        assert_eq!(
            kp.public_address().to_hex().as_str(),
            "689dae2f77b048dcc08e14d73104ea14222b5be14cc31f34a16a1221f944c1e3"
        );
    }
}
