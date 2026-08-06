use sha2::{Digest, Sha256};

use crate::api_types::ConfigRevision;

pub fn calculate(content: &str) -> ConfigRevision {
    let digest = Sha256::digest(content.as_bytes());
    ConfigRevision(format!("{digest:x}"))
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
