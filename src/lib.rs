pub use puid_macros::puid;

/// SAFETY: Buf must be at least 22 bytes long.
#[doc(hidden)]
pub fn encode_suffix(buf: &mut [u8]) {
    base62::encode_bytes(rand::random::<u128>(), buf).unwrap();
}

#[doc(hidden)]
pub fn is_valid_suffix_byte(byte: u8) -> bool {
    matches!(byte, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid length")]
    InvalidLength,
    #[error("Invalid prefix")]
    InvalidPrefix,
    #[error("Invalid format")]
    InvalidFormat,
    #[error("Invalid suffix character: {0}")]
    InvalidSuffixChar(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_suffix() {
        let mut buf = [b'0'; 22];
        encode_suffix(&mut buf);
        assert_eq!(buf.len(), 22);
        // Check that the buffer contains valid base62 characters
        for &byte in &buf {
            assert!(byte.is_ascii_alphanumeric());
        }
    }

    #[test]
    fn test_encode_suffix_with_prefix() {
        let mut buf = [b'0'; 25];
        buf[0..3].copy_from_slice(b"te_");
        encode_suffix(&mut buf[3..]);
        assert_eq!(buf.len(), 25);
        assert_eq!(&buf[0..3], b"te_");
        // Check that the suffix is valid base62 characters
        for &byte in &buf[3..] {
            assert!(byte.is_ascii_alphanumeric());
        }
    }
}
