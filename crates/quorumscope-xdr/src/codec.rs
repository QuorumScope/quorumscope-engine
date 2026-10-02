use crate::error::XdrError;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use stellar_xdr::{Limits, ReadXdr, WriteXdr};

pub fn default_limits() -> Limits {
    Limits {
        depth: 500,
        len: 1048576, // 1MB
    }
}

pub fn decode_base64(input: &str) -> Result<Vec<u8>, XdrError> {
    STANDARD.decode(input).map_err(XdrError::Base64)
}

pub fn encode_base64(input: &[u8]) -> String {
    STANDARD.encode(input)
}

pub fn decode_xdr_base64<T: ReadXdr>(input: &str) -> Result<T, XdrError> {
    let bytes = decode_base64(input)?;
    T::from_xdr(&bytes, default_limits()).map_err(XdrError::Decode)
}

pub fn encode_xdr_base64<T: WriteXdr>(value: &T) -> Result<String, XdrError> {
    let bytes = value.to_xdr(default_limits()).map_err(XdrError::Decode)?;
    Ok(encode_base64(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::LedgerKey;

    #[test]
    fn test_base64_decode_malformed() {
        assert!(decode_base64("not base 64!").is_err());
    }

    #[test]
    fn test_xdr_decode_malformed_bytes() {
        let malformed_base64 = encode_base64(&[0, 1, 2, 3]);
        assert!(decode_xdr_base64::<LedgerKey>(&malformed_base64).is_err());
    }
}
