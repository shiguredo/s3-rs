# 主要 8 種の S3 パラメータを型付き enum に置き換える

Created: 2026-05-04
Model: Opus 4.7

## 根拠

- 現状、`PutObject` / `CopyObject` / `DeleteObjects` 等の入力で `acl: Option<String>`、`storage_class: Option<String>`、`server_side_encryption: Option<String>`、`checksum_algorithm: Option<String>` のように、列挙的な値を文字列で受けている。
- aws-sdk-rust は対応する型を `ObjectCannedAcl`, `StorageClass`, `ServerSideEncryption`, `ChecksumAlgorithm` 等の `enum` で定義しており、利用者は綴り間違いをコンパイル時に検出できる。
- `AGENTS.md` / `CLAUDE.md` 「API 名、メソッド名、型名、フィールド名は aws-sdk-rust に合わせる」「推測で実装せず、仕様を確認してから実装する」の方針に整合させる。
- 内部には既に `crate::checksum::ChecksumAlgorithm` 個別 enum があり (`pub(crate)`)、これを `pub` に格上げして公開できる。
- 全 enum を一気に導入するのは Premature Optimization のため、利用頻度の高い 8 種に絞る。`RequestPayer` / `BucketLocationConstraint` / `ObjectLockMode` 系などは関連機能対応時に併せて導入する。

## 対象 enum (8 種)

| enum 型 | 関連 API | 値 |
|---|---|---|
| `ChecksumAlgorithm` | PutObject, UploadPart, CopyObject, DeleteObjects 等 | `Crc32`, `Crc32C`, `Crc64Nvme`, `Sha1`, `Sha256` |
| `ChecksumMode` | GetObject, HeadObject | `Enabled` |
| `ServerSideEncryption` | PutObject, CopyObject, CreateMultipartUpload | `Aes256`, `AwsKms`, `AwsKmsDsse` |
| `ObjectCannedAcl` | PutObject, CopyObject, CreateMultipartUpload | `Private`, `PublicRead`, `PublicReadWrite`, `AuthenticatedRead`, `AwsExecRead`, `BucketOwnerRead`, `BucketOwnerFullControl` |
| `StorageClass` | PutObject, CopyObject, CreateMultipartUpload | `Standard`, `ReducedRedundancy`, `StandardIa`, `OnezoneIa`, `IntelligentTiering`, `Glacier`, `DeepArchive`, `Outposts`, `GlacierIr`, `Snow`, `ExpressOnezone` |
| `MetadataDirective` | CopyObject | `Copy`, `Replace` |
| `TaggingDirective` | CopyObject | `Copy`, `Replace` |
| `EncodingType` | ListObjectsV2, ListObjectVersions, ListMultipartUploads | `Url` |

## 実装方針

### 構造

各 enum は以下の最小実装とする (aws-sdk-rust と互換の API)。

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChecksumAlgorithm {
    Crc32,
    Crc32C,
    Crc64Nvme,
    Sha1,
    Sha256,
    /// 未知の値を保持する (前方互換性のため)
    Unknown(String),
}

impl ChecksumAlgorithm {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Crc32 => "CRC32",
            Self::Crc32C => "CRC32C",
            Self::Crc64Nvme => "CRC64NVME",
            Self::Sha1 => "SHA1",
            Self::Sha256 => "SHA256",
            Self::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for ChecksumAlgorithm {
    fn from(s: &str) -> Self {
        match s {
            "CRC32" => Self::Crc32,
            "CRC32C" => Self::Crc32C,
            "CRC64NVME" => Self::Crc64Nvme,
            "SHA1" => Self::Sha1,
            "SHA256" => Self::Sha256,
            _ => Self::Unknown(s.to_string()),
        }
    }
}
```

- `#[non_exhaustive]` を付けることで、将来の値追加で `match` の網羅エラーを起こさない。
- `Unknown(String)` variant により、S3 互換ストレージが返す未知の値もパース可能にする (前方互換性)。
- aws-sdk-rust 互換の `as_str()` メソッドを提供する。
- `From<&str>` で文字列からの変換を許容する。

### 入力フィールドの型変更

`PutObject` 等のビルダーメソッドは `impl Into<ChecksumAlgorithm>` を受けるシグネチャに変更し、利用者が `ChecksumAlgorithm::Crc32C` も `"CRC32C"` も渡せるようにする。

```rust
pub fn checksum_algorithm(mut self, algorithm: impl Into<ChecksumAlgorithm>) -> Self {
    self.checksum_algorithm = Some(algorithm.into());
    self
}
```

### 出力フィールドの型変更

`Object.storage_class`, `ObjectVersion.storage_class`, `ListObjectVersionsOutput.encoding_type` 等の出力側も対応 enum に変更する。XML/ヘッダーのパースは `ChecksumAlgorithm::from(s)` で行う。

### 既存 `crate::checksum::ChecksumAlgorithm` の扱い

- `src/checksum.rs:11-18` の `pub(crate) enum ChecksumAlgorithm` を `src/types/enums.rs` に移動して `pub` に格上げする。
- variant 名を AWS S3 API 仕様準拠に揃える (`Crc32` → `Crc32`、`Crc32c` → `Crc32C`、`Crc64nvme` → `Crc64Nvme` 等、aws-sdk-rust と一致)。
- 内部の署名計算側は新しい公開 enum を直接利用する。

### `lib.rs` の `pub use`

`pub use types::{ChecksumAlgorithm, ChecksumMode, ServerSideEncryption, ObjectCannedAcl, StorageClass, MetadataDirective, TaggingDirective, EncodingType};` を追加する。

## 後回しにする enum (本 issue 対象外)

以下は関連機能対応時に併せて導入する。

- `RequestPayer`, `RequestCharged` (issue 0057 で扱う)
- `BucketLocationConstraint` (Directory Bucket 対応時)
- `ObjectLockMode`, `ObjectLockLegalHoldStatus`, `ObjectLockRetentionMode` (Object Lock 本格対応時)
- `ReplicationStatus` (レプリケーション機能時)
- `ObjectStorageClass`, `ObjectVersionStorageClass` (Output 細分化時、または `StorageClass` で代用)
- `ChecksumType` (`CRC64NVME` 関連で必要になったとき)

## AWS S3 API Reference

複数 API で使用するため、代表的な箇所を引用する。

- [PutObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html)

  > x-amz-checksum-algorithm — Indicates the algorithm used to create the checksum for the object when you use the SDK. Valid values: CRC32 | CRC32C | CRC64NVME | SHA1 | SHA256.

  > x-amz-acl — The canned ACL to apply to the object. Valid values: private | public-read | public-read-write | authenticated-read | aws-exec-read | bucket-owner-read | bucket-owner-full-control.

  > x-amz-server-side-encryption — Valid values: AES256 | aws:kms | aws:kms:dsse.

  > x-amz-storage-class — Valid values: STANDARD | REDUCED_REDUNDANCY | STANDARD_IA | ONEZONE_IA | INTELLIGENT_TIERING | GLACIER | DEEP_ARCHIVE | OUTPOSTS | GLACIER_IR | SNOW | EXPRESS_ONEZONE.

- [CopyObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html)

  > x-amz-metadata-directive — Specifies whether the metadata is copied from the source object or replaced with metadata that's provided in the request. Valid values: COPY | REPLACE.

  > x-amz-tagging-directive — Specifies whether the object tag-set is copied from the source object or replaced with the tag-set that's provided in the request. Valid values: COPY | REPLACE.

- [GetObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html)

  > x-amz-checksum-mode — To retrieve the checksum, this mode must be enabled. Valid values: ENABLED.

- [ListObjectsV2](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html)

  > encoding-type — Encoding type used by Amazon S3 to encode object keys in the response. Valid values: url.

## 影響範囲

- `src/api/put_object.rs:22-50`、`src/api/copy_object.rs:14-58`、`src/api/delete_objects.rs:14-19`、`src/api/get_object.rs:36`、`src/api/head_object.rs`、`src/api/list_objects_v2.rs:22`、`src/api/upload_part.rs:24` 等のフィールド型変更とビルダーメソッド更新。
- `src/types.rs:330,342,361` 等の出力フィールド型変更。
- `examples/s3cli`, `tests/minio.rs`, `tests/rustfs.rs` の呼び出し箇所書き換え (現状文字列を渡しているので大半はそのまま `impl Into<...>` で動くが、出力受け取り側は要変更)。

## 優先度

高 (他の issue で「型付き enum を引数にとる」前提を作るため)

## CHANGES.md への記載

- `[CHANGE] PutObject 等の checksum_algorithm を ChecksumAlgorithm enum に変更する`
- `[CHANGE] PutObject 等の acl を ObjectCannedAcl enum に変更する`
- `[CHANGE] PutObject 等の storage_class を StorageClass enum に変更する`
- `[CHANGE] PutObject 等の server_side_encryption を ServerSideEncryption enum に変更する`
- `[CHANGE] CopyObject の metadata_directive / tagging_directive を MetadataDirective / TaggingDirective enum に変更する`
- `[CHANGE] GetObject / HeadObject の checksum_mode を ChecksumMode enum に変更する`
- `[CHANGE] ListObjectsV2 等の encoding_type を EncodingType enum に変更する`
- `[ADD] ChecksumAlgorithm を public 型として公開する`
