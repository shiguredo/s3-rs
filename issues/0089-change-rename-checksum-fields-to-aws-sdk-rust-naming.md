# GetObjectOutput / HeadObjectOutput の checksum フィールド名を aws-sdk-rust 互換に統一する

- Priority: Medium
- Created: 2026-07-07
- Model: hy3-free
- Branch: feature/change-checksum-field-naming

## AWS S3 API Reference

- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>

> The GET operation retrieves objects from Amazon S3.

レスポンスヘッダ（同ページ）より、checksum は以下のように公開される:

> `x-amz-checksum-crc32` / `x-amz-checksum-crc32c` / `x-amz-checksum-crc64nvme` / `x-amz-checksum-sha1` / `x-amz-checksum-sha256`

aws-sdk-rust の `GetObjectOutput` / `HeadObjectOutput` は対応する Rust フィールドを `checksum_crc32_c` / `checksum_crc64_nvme` とする（HTTP ヘッダ名の `crc32c` / `crc64nvme` をスネークケースにした際のアンダースコア位置）。

## 目的

本クレートが謳う aws-sdk-rust 互換 API を満たすため、レスポンス出力型の checksum フィールド名を aws-sdk-rust と一致させる。

## 優先度根拠

- 無線（HTTP ヘッダ）マッピングは正しいが、Rust 構造体フィールド名が aws-sdk-rust と異なるため、aws-sdk-rust から移行する利用者が期待するフィールド名でアクセスできず違和感・移行コストが生じる（AGENTS.md の「aws-sdk-rust 互換をできるだけ維持する」方針に抵触）。
- リクエスト側（`PutObjectOutput` 等）は `checksum_crc32_c` / `checksum_crc64_nvme` を使用しており、レスポンス側と内部でも不揃い。

## 現状

- `src/types.rs:137,139,141`（`GetObjectOutput`）は `checksum_crc32c` / `checksum_crc64nvme`。
- `src/types.rs:195,197,199`（`HeadObjectOutput`）は `checksum_crc32c` / `checksum_crc64nvme`。
- 利用側 `src/api/get_object.rs:385,388`、`src/api/head_object.rs:289,292` は上記フィールド名で構築している。
- HTTP ヘッダ名 `x-amz-checksum-crc32c`（GetObject レスポンス）の取得自体は正しい。

## 設計方針

- `checksum_crc32c` → `checksum_crc32_c`、`checksum_crc64nvme` → `checksum_crc64_nvme` にリネーム。
- `get_object.rs` / `head_object.rs` の構築コードも併せて修正。
- フィールド追加ではなくリネームのため、後方互換のない変更（`[CHANGE]`）として扱う。aws-sdk-rust にある `checksum_type` フィールドの有無も併せて確認し、欠落していれば追加を検討する。

## 完了条件

- `GetObjectOutput` / `HeadObjectOutput` の checksum フィールド名が aws-sdk-rust と一致すること。
- 利用側の構築コードが修正され、コンパイルが通ること。
- `CHANGES.md` の `## develop` セクションに `[CHANGE]` エントリを記載すること。

## 解決方法

（未着手）
