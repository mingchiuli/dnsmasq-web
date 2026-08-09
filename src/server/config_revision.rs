use sha2::{Digest, Sha256};

use crate::api_types::ConfigRevision;

pub fn calculate(content: &str) -> ConfigRevision {
    let digest = Sha256::digest(content.as_bytes());
    ConfigRevision(encode_lower_hex(digest.as_ref()))
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        encoded.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::calculate;

    #[test]
    fn revision_is_stable_sha256_of_content() {
        assert_eq!(
            calculate("dnsmasq\n").0,
            "7404f2704d756a0985ce5a046ca1334c8e0e5d0753be50a11d5cd1a2d7e5a60d"
        );
    }
}
