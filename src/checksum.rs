//! チェックサム計算モジュール
//!
//! S3 の Flexible Checksums で使用するチェックサムを計算する。
//! アルゴリズムに応じたヘッダー名と Base64 エンコード済みの値を返す。
//!
//! 公開型は `crate::types::ChecksumAlgorithm` を使う (aws-sdk-rust 互換)。
//! shiguredo_s3 内部の実装は CRC32 / CRC32C / CRC64NVME / SHA1 / SHA256 のみ
//! 対応しており、それ以外の variant が指定された場合は `Error::InvalidInput`
//! を返す。

use base64ct::{Base64, Encoding};

use crate::error::Error;
use crate::types::ChecksumAlgorithm;

/// 対応する `x-amz-checksum-*` ヘッダー名を返す
///
/// shiguredo_s3 が内部で計算可能な 5 種のみ対応する。それ以外は
/// `Error::InvalidInput` を返す。
pub(crate) fn header_name(algorithm: &ChecksumAlgorithm) -> Result<&'static str, Error> {
    match algorithm {
        ChecksumAlgorithm::Crc32 => Ok("x-amz-checksum-crc32"),
        ChecksumAlgorithm::Crc32C => Ok("x-amz-checksum-crc32c"),
        ChecksumAlgorithm::Crc64Nvme => Ok("x-amz-checksum-crc64nvme"),
        ChecksumAlgorithm::Sha1 => Ok("x-amz-checksum-sha1"),
        ChecksumAlgorithm::Sha256 => Ok("x-amz-checksum-sha256"),
        other => Err(Error::InvalidInput(format!(
            "unsupported checksum algorithm: {}",
            other.as_str()
        ))),
    }
}

/// ボディからチェックサムを計算して Base64 エンコード済み文字列を返す
///
/// shiguredo_s3 が内部で計算可能な 5 種のみ対応する。それ以外は
/// `Error::InvalidInput` を返す。
pub(crate) fn compute_checksum(
    algorithm: &ChecksumAlgorithm,
    data: &[u8],
) -> Result<String, Error> {
    let value = match algorithm {
        ChecksumAlgorithm::Crc32 => {
            let hash = crc_fast::checksum(crc_fast::CrcAlgorithm::Crc32IsoHdlc, data);
            Base64::encode_string(&(hash as u32).to_be_bytes())
        }
        ChecksumAlgorithm::Crc32C => {
            let hash = crc_fast::checksum(crc_fast::CrcAlgorithm::Crc32Iscsi, data);
            Base64::encode_string(&(hash as u32).to_be_bytes())
        }
        ChecksumAlgorithm::Crc64Nvme => {
            let hash = crc_fast::checksum(crc_fast::CrcAlgorithm::Crc64Nvme, data);
            Base64::encode_string(&hash.to_be_bytes())
        }
        ChecksumAlgorithm::Sha1 => Base64::encode_string(&sha1_digest(data)),
        ChecksumAlgorithm::Sha256 => Base64::encode_string(&crate::signing::sha256(data)),
        other => {
            return Err(Error::InvalidInput(format!(
                "unsupported checksum algorithm: {}",
                other.as_str()
            )));
        }
    };
    Ok(value)
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

    #[test]
    fn test_header_name_supported() {
        assert_eq!(
            header_name(&ChecksumAlgorithm::Crc32).unwrap(),
            "x-amz-checksum-crc32"
        );
        assert_eq!(
            header_name(&ChecksumAlgorithm::Sha256).unwrap(),
            "x-amz-checksum-sha256"
        );
    }

    #[test]
    fn test_header_name_unsupported() {
        assert!(header_name(&ChecksumAlgorithm::Md5).is_err());
        assert!(header_name(&ChecksumAlgorithm::Sha512).is_err());
        assert!(header_name(&ChecksumAlgorithm::Xxhash64).is_err());
        assert!(header_name(&ChecksumAlgorithm::Unknown("FOO".to_string())).is_err());
    }

    /// 空のデータに対する SHA-256 チェックサムを検証する
    #[test]
    fn test_compute_checksum_sha256_empty() {
        let result = compute_checksum(&ChecksumAlgorithm::Sha256, b"").unwrap();
        // SHA-256("") の Base64
        assert_eq!(result, "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=");
    }

    /// 既知の入力に対する CRC32 チェックサムを検証する
    #[test]
    fn test_compute_checksum_crc32() {
        let result = compute_checksum(&ChecksumAlgorithm::Crc32, b"Hello, world!").unwrap();
        // CRC32("Hello, world!") = 0xEBE6C6E6 → big-endian → Base64
        let expected = Base64::encode_string(&0xEBE6C6E6u32.to_be_bytes());
        assert_eq!(result, expected);
    }

    /// 空のデータに対する SHA-1 チェックサムを検証する
    #[test]
    fn test_compute_checksum_sha1_empty() {
        let result = compute_checksum(&ChecksumAlgorithm::Sha1, b"").unwrap();
        // SHA-1("") の Base64
        assert_eq!(result, "2jmj7l5rSw0yVb/vlWAYkK/YBwk=");
    }

    #[test]
    fn test_compute_checksum_unsupported() {
        assert!(compute_checksum(&ChecksumAlgorithm::Md5, b"").is_err());
        assert!(compute_checksum(&ChecksumAlgorithm::Sha512, b"").is_err());
    }

    /// `From<&str>` の挙動を検証する (types.rs に書く方が筋がよいが、内部 API への
    /// 影響を確認するため checksum.rs 側の単体テストとして残す)
    #[test]
    fn test_checksum_algorithm_from_str() {
        assert_eq!(ChecksumAlgorithm::from("CRC32"), ChecksumAlgorithm::Crc32);
        assert_eq!(ChecksumAlgorithm::from("CRC32C"), ChecksumAlgorithm::Crc32C);
        assert_eq!(
            ChecksumAlgorithm::from("CRC64NVME"),
            ChecksumAlgorithm::Crc64Nvme
        );
        assert_eq!(ChecksumAlgorithm::from("MD5"), ChecksumAlgorithm::Md5);
        assert_eq!(ChecksumAlgorithm::from("SHA1"), ChecksumAlgorithm::Sha1);
        assert_eq!(ChecksumAlgorithm::from("SHA256"), ChecksumAlgorithm::Sha256);
        assert_eq!(ChecksumAlgorithm::from("SHA512"), ChecksumAlgorithm::Sha512);
        assert_eq!(
            ChecksumAlgorithm::from("XXHASH128"),
            ChecksumAlgorithm::Xxhash128
        );
        assert_eq!(
            ChecksumAlgorithm::from("FOO"),
            ChecksumAlgorithm::Unknown("FOO".to_string())
        );
    }
}
