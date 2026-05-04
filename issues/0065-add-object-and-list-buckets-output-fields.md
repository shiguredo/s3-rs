# Owner / RestoreStatus 型を新設し Object / ObjectVersion / ListBucketsOutput にフィールド追加する

Created: 2026-05-04
Model: Opus 4.7

## 根拠

- issue 0064 で `docs/AWS_SDK_RUST.md` の方針を再分類した結果、`Object` / `ObjectVersion` / `ListBucketsOutput` 等の出力構造体が aws-sdk-rust と比べてフィールド不足となっていることが判明した。
- aws-sdk-rust の `Object` (`/Users/voluntas/src/aws-sdk-rust/sdk/s3/src/types/_object.rs`) には `owner: Option<Owner>`, `restore_status: Option<RestoreStatus>`, `checksum_algorithm: Option<Vec<ChecksumAlgorithm>>`, `checksum_type: Option<ChecksumType>` が定義されている。
- ListObjectsV2 / ListObjectVersions の XML レスポンスにも `<Owner>`, `<RestoreStatus>`, `<ChecksumAlgorithm>` 要素が含まれており、互換ストレージから返却される可能性がある。
- ListBuckets の XML レスポンスにも `<Owner>` 要素が含まれる。
- 出力フィールドの追加のため後方互換ではあるが、`Object` 構造体への変更は構造体パターンマッチに影響する可能性がある (`#[non_exhaustive]` 化と合わせて検討)。

## 変更内容

### 1. `Owner` / `RestoreStatus` 型の新設

`src/types.rs` に以下を追加する。

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Owner {
    pub display_name: Option<String>,
    pub id: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RestoreStatus {
    pub is_restore_in_progress: Option<bool>,
    pub restore_expiry_date: Option<SystemTime>,  // issue 0060 の方針に従う
}
```

`#[non_exhaustive]` を付けて将来のフィールド追加に対応する。

### 2. `Object` (`src/types.rs:354-362`) への追加

```rust
pub owner: Option<Owner>,
pub restore_status: Option<RestoreStatus>,
pub checksum_algorithm: Option<Vec<ChecksumAlgorithm>>,  // issue 0059 で型化
pub checksum_type: Option<String>,  // issue 0059 後続で enum 化検討
```

### 3. `ObjectVersion` (`src/types.rs:333-343`) への追加

```rust
pub owner: Option<Owner>,
pub restore_status: Option<RestoreStatus>,
pub checksum_algorithm: Option<Vec<ChecksumAlgorithm>>,
pub checksum_type: Option<String>,
```

### 4. `ListBucketsOutput` (`src/types.rs:399-406`) への追加

```rust
pub owner: Option<Owner>,
```

### 5. パース処理の追加

各 `parse_response` 内で XML 要素 (`<Owner>`, `<RestoreStatus>`, `<ChecksumAlgorithm>`, `<ChecksumType>`) からフィールドを抽出する処理を追加する。

具体的には:
- `src/api/list_objects_v2.rs` の XML パース処理
- `src/api/list_object_versions.rs` の XML パース処理
- `src/api/list_buckets.rs` の XML パース処理

### 6. `lib.rs` の `pub use` 更新

```rust
pub use types::{Owner, RestoreStatus};
```

## AWS S3 API Reference

- [ListObjectsV2](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html)

  XML レスポンス内 `<Contents>` 要素:

  > ```xml
  > <Contents>
  >    <Key>string</Key>
  >    <LastModified>timestamp</LastModified>
  >    <ETag>string</ETag>
  >    <ChecksumAlgorithm>string</ChecksumAlgorithm>
  >    <ChecksumType>string</ChecksumType>
  >    <Size>long</Size>
  >    <StorageClass>string</StorageClass>
  >    <Owner>
  >       <DisplayName>string</DisplayName>
  >       <ID>string</ID>
  >    </Owner>
  >    <RestoreStatus>
  >       <IsRestoreInProgress>boolean</IsRestoreInProgress>
  >       <RestoreExpiryDate>timestamp</RestoreExpiryDate>
  >    </RestoreStatus>
  > </Contents>
  > ```

- [ListObjectVersions](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectVersions.html)

  XML レスポンス内 `<Version>` 要素は `<Contents>` と類似の構造で `<Owner>`, `<RestoreStatus>`, `<ChecksumAlgorithm>`, `<ChecksumType>` を含む。

- [ListBuckets](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListBuckets.html)

  XML レスポンス:

  > ```xml
  > <ListAllMyBucketsResult>
  >    <Buckets>...</Buckets>
  >    <Owner>
  >       <DisplayName>string</DisplayName>
  >       <ID>string</ID>
  >    </Owner>
  >    <ContinuationToken>string</ContinuationToken>
  >    <Prefix>string</Prefix>
  > </ListAllMyBucketsResult>
  > ```

## 影響範囲

- `src/types.rs` への `Owner` / `RestoreStatus` 型新設、`Object` / `ObjectVersion` / `ListBucketsOutput` へのフィールド追加。
- `src/api/list_objects_v2.rs` / `src/api/list_object_versions.rs` / `src/api/list_buckets.rs` のパース処理拡張。
- `src/lib.rs` の `pub use` 更新。
- `examples/s3cli`, `tests/` の既存出力アクセスは影響を受けない (フィールド追加のみのため)。

## 依存関係

- 本 issue は issue 0059 (enum 化) および issue 0060 (時刻処理) の後に実施することで型整合が取りやすい。

## 優先度

中

## CHANGES.md への記載

- `[ADD] Owner / RestoreStatus 型を追加する`
- `[ADD] Object / ObjectVersion に owner / restore_status / checksum_algorithm / checksum_type を追加する`
- `[ADD] ListBucketsOutput に owner を追加する`
