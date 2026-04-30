//! チェックサム計算モジュール
//!
//! S3 の Flexible Checksums で使用するチェックサムを計算する。
//! アルゴリズムに応じたヘッダー名と Base64 エンコード済みの値を返す。

use base64ct::{Base64, Encoding};

use crate::error::Error;

/// チェックサムアルゴリズム
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChecksumAlgorithm {
    Crc32,
    Crc32c,
    Crc64nvme,
    Sha1,
    Sha256,
}

impl std::str::FromStr for ChecksumAlgorithm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CRC32" => Ok(Self::Crc32),
            "CRC32C" => Ok(Self::Crc32c),
            "CRC64NVME" => Ok(Self::Crc64nvme),
            "SHA1" => Ok(Self::Sha1),
            "SHA256" => Ok(Self::Sha256),
            _ => Err(Error::InvalidInput(format!(
                "unsupported checksum algorithm: {s}"
            ))),
        }
    }
}

impl ChecksumAlgorithm {
    /// 対応する x-amz-checksum-* ヘッダー名を返す
    pub(crate) fn header_name(&self) -> &'static str {
        match self {
            Self::Crc32 => "x-amz-checksum-crc32",
            Self::Crc32c => "x-amz-checksum-crc32c",
            Self::Crc64nvme => "x-amz-checksum-crc64nvme",
            Self::Sha1 => "x-amz-checksum-sha1",
            Self::Sha256 => "x-amz-checksum-sha256",
        }
    }
}

/// ボディからチェックサムを計算して Base64 エンコード済み文字列を返す
pub(crate) fn compute_checksum(algorithm: ChecksumAlgorithm, data: &[u8]) -> String {
    match algorithm {
        ChecksumAlgorithm::Crc32 => {
            let hash = crc_fast::checksum(crc_fast::CrcAlgorithm::Crc32IsoHdlc, data);
            Base64::encode_string(&(hash as u32).to_be_bytes())
        }
        ChecksumAlgorithm::Crc32c => {
            let hash = crc_fast::checksum(crc_fast::CrcAlgorithm::Crc32Iscsi, data);
            Base64::encode_string(&(hash as u32).to_be_bytes())
        }
        ChecksumAlgorithm::Crc64nvme => {
            let hash = crc_fast::checksum(crc_fast::CrcAlgorithm::Crc64Nvme, data);
            Base64::encode_string(&hash.to_be_bytes())
        }
        ChecksumAlgorithm::Sha1 => Base64::encode_string(&sha1_digest(data)),
        ChecksumAlgorithm::Sha256 => Base64::encode_string(&crate::signing::sha256(data)),
    }
}

// -------------------------------------------------------
// SHA-1 (feature で切り替え)
// -------------------------------------------------------

/// SHA-1 ハッシュを計算する
///
/// 両方の feature が有効な場合は rust-crypto を優先する
#[cfg(feature = "rust-crypto")]
fn sha1_digest(data: &[u8]) -> [u8; 20] {
    use sha1::Digest;
    sha1::Sha1::digest(data).into()
}

#[cfg(all(feature = "aws_lc_rs", not(feature = "rust-crypto")))]
fn sha1_digest(data: &[u8]) -> [u8; 20] {
    let d = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA1_FOR_LEGACY_USE_ONLY, data);
    d.as_ref().try_into().expect("SHA-1 digest is 20 bytes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_checksum_algorithm_from_str() {
        assert_eq!(
            ChecksumAlgorithm::from_str("CRC32").unwrap(),
            ChecksumAlgorithm::Crc32
        );
        assert_eq!(
            ChecksumAlgorithm::from_str("CRC32C").unwrap(),
            ChecksumAlgorithm::Crc32c
        );
        assert_eq!(
            ChecksumAlgorithm::from_str("CRC64NVME").unwrap(),
            ChecksumAlgorithm::Crc64nvme
        );
        assert_eq!(
            ChecksumAlgorithm::from_str("SHA1").unwrap(),
            ChecksumAlgorithm::Sha1
        );
        assert_eq!(
            ChecksumAlgorithm::from_str("SHA256").unwrap(),
            ChecksumAlgorithm::Sha256
        );
        assert!(ChecksumAlgorithm::from_str("MD5").is_err());
    }

    #[test]
    fn test_checksum_algorithm_header_name() {
        assert_eq!(
            ChecksumAlgorithm::Crc32.header_name(),
            "x-amz-checksum-crc32"
        );
        assert_eq!(
            ChecksumAlgorithm::Sha256.header_name(),
            "x-amz-checksum-sha256"
        );
    }

    /// 空のデータに対する SHA-256 チェックサムを検証する
    #[test]
    fn test_compute_checksum_sha256_empty() {
        let result = compute_checksum(ChecksumAlgorithm::Sha256, b"");
        // SHA-256("") の Base64
        assert_eq!(result, "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=");
    }

    /// 既知の入力に対する CRC32 チェックサムを検証する
    #[test]
    fn test_compute_checksum_crc32() {
        let result = compute_checksum(ChecksumAlgorithm::Crc32, b"Hello, world!");
        // CRC32("Hello, world!") = 0xEBE6C6E6 → big-endian → Base64
        let expected = Base64::encode_string(&0xEBE6C6E6u32.to_be_bytes());
        assert_eq!(result, expected);
    }

    /// 空のデータに対する SHA-1 チェックサムを検証する
    #[test]
    fn test_compute_checksum_sha1_empty() {
        let result = compute_checksum(ChecksumAlgorithm::Sha1, b"");
        // SHA-1("") の Base64
        assert_eq!(result, "2jmj7l5rSw0yVb/vlWAYkK/YBwk=");
    }
}
