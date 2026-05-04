# 主要 8 種の S3 パラメータを型付き enum に置き換える

Created: 2026-05-04
Completed: 2026-05-04
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
| `ChecksumAlgorithm` | PutObject, UploadPart, CopyObject, DeleteObjects 等 | `Crc32`, `Crc32C`, `Crc64Nvme`, `Md5`, `Sha1`, `Sha256`, `Sha512`, `Xxhash128`, `Xxhash3`, `Xxhash64` |
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
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Xxhash128,
    Xxhash3,
    Xxhash64,
    /// 未知の値を保持する (前方互換性のため)
    Unknown(String),
}

impl ChecksumAlgorithm {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Crc32 => "CRC32",
            Self::Crc32C => "CRC32C",
            Self::Crc64Nvme => "CRC64NVME",
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA1",
            Self::Sha256 => "SHA256",
            Self::Sha512 => "SHA512",
            Self::Xxhash128 => "XXHASH128",
            Self::Xxhash3 => "XXHASH3",
            Self::Xxhash64 => "XXHASH64",
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
            "MD5" => Self::Md5,
            "SHA1" => Self::Sha1,
            "SHA256" => Self::Sha256,
            "SHA512" => Self::Sha512,
            "XXHASH128" => Self::Xxhash128,
            "XXHASH3" => Self::Xxhash3,
            "XXHASH64" => Self::Xxhash64,
            _ => Self::Unknown(s.to_string()),
        }
    }
}
```

- `#[non_exhaustive]` を付けることで、将来の値追加で `match` の網羅エラーを起こさない。
- `Unknown(String)` variant により、S3 互換ストレージが返す未知の値もパース可能にする (前方互換性)。
- aws-sdk-rust 互換の `as_str()` メソッドを提供する。
- `From<&str>` で文字列からの変換を許容する。
- variants は aws-sdk-rust の `ChecksumAlgorithm` (`/Users/voluntas/src/aws-sdk-rust/sdk/s3/src/types/_checksum_algorithm.rs:52`) と完全に揃える。S3 公式ドキュメントの「Valid values」では CRC32 / CRC32C / CRC64NVME / SHA1 / SHA256 のみ列挙されているが、aws-sdk-rust の Smithy モデルには 10 種が定義されているため、互換性最優先方針に従い shiguredo_s3 でも 10 種すべてを公開する。
- 他の enum (`ServerSideEncryption`, `ObjectCannedAcl`, `StorageClass`, `MetadataDirective`, `TaggingDirective`, `EncodingType`, `ChecksumMode`) も同様の構造で実装し、variants は aws-sdk-rust のものと完全に揃える。

### 入力フィールドの型変更

aws-sdk-rust と完全に揃えるため、ビルダーメソッドは型を直接受けるシグネチャにする (`impl Into<ChecksumAlgorithm>` は使わない)。

```rust
pub fn checksum_algorithm(mut self, input: ChecksumAlgorithm) -> Self {
    self.checksum_algorithm = Some(input);
    self
}

pub fn set_checksum_algorithm(mut self, input: Option<ChecksumAlgorithm>) -> Self {
    self.checksum_algorithm = input;
    self
}
```

- 利用者が `&str` を持っている場合は `ChecksumAlgorithm::from("CRC32C")` を明示的に呼ぶ形になる。
- `impl Into<T>` 方式は型推論失敗時のエラーメッセージが分かりにくくなりやすく、aws-sdk-rust と挙動も異なるため採用しない。
- aws-sdk-rust 互換の `set_*` バリアントを並列で提供する。

### 出力フィールドの型変更

`Object.storage_class`, `ObjectVersion.storage_class`, `ListObjectVersionsOutput.encoding_type` 等の出力側も対応 enum に変更する。XML/ヘッダーのパースは `ChecksumAlgorithm::from(s)` で行う。

### 既存 `crate::checksum::ChecksumAlgorithm` の扱い

- `src/checksum.rs:11-18` の `pub(crate) enum ChecksumAlgorithm` を `src/types/enums.rs` に移動して `pub` に格上げする。
- variant 名を AWS S3 API 仕様準拠に揃える (`Crc32` → `Crc32`、`Crc32c` → `Crc32C`、`Crc64nvme` → `Crc64Nvme` 等、aws-sdk-rust と一致)。
- 内部の署名計算側は新しい公開 enum を直接利用する。
- `Md5` / `Sha512` / `Xxhash*` は内部チェックサム計算には未対応のため、対象 variant が指定された場合は明確なエラー (`Error::UnsupportedChecksumAlgorithm` 等) を返す。「公開する型としては aws-sdk-rust と揃え、実装の進捗に応じて段階的にサポートする」スタイルにする。

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
- `examples/s3cli`, `tests/minio.rs`, `tests/rustfs.rs` の呼び出し箇所書き換え。文字列を直接渡している箇所はすべて `ChecksumAlgorithm::from("CRC32C")` 等の明示変換に置換する必要がある。

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

## 解決方法

### 実施した変更

1. **8 種の型付き enum を新設 (`src/types.rs`)**
   - `ChecksumAlgorithm` (10 variants: `Crc32, Crc32C, Crc64Nvme, Md5, Sha1, Sha256, Sha512, Xxhash128, Xxhash3, Xxhash64`)
   - `ChecksumMode` (`Enabled`)
   - `ServerSideEncryption` (`Aes256, AwsFsx, AwsKms, AwsKmsDsse`)
   - `ObjectCannedAcl` (7 variants)
   - `StorageClass` (13 variants)
   - `MetadataDirective` (`Copy, Replace`)
   - `TaggingDirective` (`Copy, Replace`)
   - `EncodingType` (`Url`)
   - 各 enum は `#[non_exhaustive]`、`Unknown(String)` variant、`as_str()` メソッド、`From<&str>`、`Display` を持つ
   - variants は aws-sdk-rust の対応 enum と完全一致 (issue spec の variant 数より多い場合も含む)

2. **`src/checksum.rs` のリファクタリング**
   - `pub(crate) enum ChecksumAlgorithm` を削除
   - `header_name` / `compute_checksum` を public な `ChecksumAlgorithm` を引数に取り、`Result<_, Error>` を返す関数に変更
   - 内部計算未対応の variant (`Md5`, `Sha512`, `Xxhash*`, `Unknown`) は `Error::InvalidInput` を返す
   - `test_header_name_unsupported` / `test_compute_checksum_unsupported` を追加

3. **入力フィールドを enum 型に変更**
   - `src/api/put_object.rs`, `src/api/copy_object.rs`, `src/api/create_multipart_upload.rs`, `src/api/create_bucket.rs`, `src/api/upload_part.rs`, `src/api/delete_objects.rs` の `acl` / `storage_class` / `server_side_encryption` / `checksum_algorithm` / `metadata_directive` / `tagging_directive` を対応 enum に変更
   - `src/api/get_object.rs`, `src/api/head_object.rs` の `checksum_mode` を `ChecksumMode` に変更
   - `src/api/list_objects_v2.rs`, `src/api/list_object_versions.rs`, `src/api/list_multipart_uploads.rs` の `encoding_type` を `EncodingType` に変更
   - `src/api/put_bucket_*.rs` (8 ファイル) の `checksum_algorithm` を `ChecksumAlgorithm` に変更
   - すべての enum 化対象ビルダーに aws-sdk-rust 互換の `set_*` バリアントを追加 (Option を直接受ける)

4. **出力フィールドを enum 型に変更 (`src/types.rs`)**
   - `Object.storage_class`, `ObjectVersion.storage_class`, `ListPartsOutput.storage_class`, `MultipartUpload.storage_class`, `HeadObjectOutput.storage_class` を `Option<StorageClass>` に変更
   - `Transition.storage_class`, `NoncurrentVersionTransition.storage_class` を `Option<StorageClass>` に変更
   - `ListObjectVersionsOutput.encoding_type` を `Option<EncodingType>` に変更
   - 各 `parse_response` で `From<&str>` を使ってパースする

5. **examples / tests の追従**
   - `examples/s3cli/src/params.rs`: builder 適用箇所を `Enum::from(v.as_str())` 変換に書き換え
   - `examples/s3cli/src/commands.rs`: `Object.storage_class.as_deref()` を `as_ref().map(|sc| sc.as_str())` に変更
   - `tests/minio.rs`: `.checksum_algorithm("CRC32C")` 等の文字列を enum バリアントに置換

6. **`src/lib.rs` の `pub use` 拡張**
   - 新設した 8 種の enum を `pub use types::{...}` で公開

### 検証結果

- `cargo check --workspace --all-targets`: 成功
- `cargo clippy --workspace --all-targets`: 警告ゼロ
- `cargo test --lib`: 14 tests passed (新規追加 2 件含む)
- `cargo test --test minio test_object_put_get_head_delete test_checksum_algorithm test_put_object_acl test_put_object_storage_class test_copy_object_metadata_replace`: 5 tests passed
- pre-commit hook (cargo fmt / clippy / test) すべて pass
