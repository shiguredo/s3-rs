# バケット暗号化 API を追加する

Created: 2026-03-27
Completed: 2026-05-04
Model: Opus 4.6

## 概要

GetBucketEncryption / PutBucketEncryption / DeleteBucketEncryption の 3 API を追加する。

## 根拠

バケットのデフォルト暗号化設定を管理する S3 標準 API。2023 年 1 月以降 S3 は全オブジェクトをデフォルトで SSE-S3 (AES256) 暗号化するようになったが、SSE-KMS への変更や S3 Bucket Key の有効化など、暗号化方式のカスタマイズに必要。

Cloudflare R2 では GetBucketEncryption のみサポート（読み取り専用、常に AES256）。

## 対象 API

### GetBucketEncryption

- `GET /{Bucket}?encryption`
- デフォルト暗号化設定を取得する
- レスポンス: `<ServerSideEncryptionConfiguration>` XML

### PutBucketEncryption

- `PUT /{Bucket}?encryption`
- デフォルト暗号化設定を作成・更新する
- リクエスト: `<ServerSideEncryptionConfiguration>` XML

### DeleteBucketEncryption

- `DELETE /{Bucket}?encryption`
- カスタム暗号化設定を削除し、デフォルト (SSE-S3) に戻す

## 追加する型

- `ServerSideEncryptionRule`: SSEAlgorithm, KMSMasterKeyID, BucketKeyEnabled
- `ServerSideEncryptionConfiguration`: Rule のリスト
- `GetBucketEncryptionOutput`
- `PutBucketEncryptionOutput`
- `DeleteBucketEncryptionOutput`

## 実装上の注意

- XML のネスト構造が深い: `Rule > ApplyServerSideEncryptionByDefault > SSEAlgorithm`
- `for_each_element` は直接の子要素 (depth=2) のみ取得するため、ネストした要素のパースに工夫が必要
- 対処案: `extract_element` でドキュメント全体からフラットに取得する、または XML リーダーで直接パースする専用関数を書く

## 参考

- https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketEncryption.html
- https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketEncryption.html
- https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketEncryption.html

## 優先度

中

## 解決方法

### 実装状況

本 issue が要求する 3 API はいずれも、過去のリリースで既に実装済みであり、本 issue の closed 化時点で `src/api/get_bucket_encryption.rs` / `src/api/put_bucket_encryption.rs` / `src/api/delete_bucket_encryption.rs` として動作している。

- `Client::get_bucket_encryption()` / `put_bucket_encryption()` / `delete_bucket_encryption()` を提供
- 関連型 `ServerSideEncryptionRule` / `ServerSideEncryptionConfiguration` / `ServerSideEncryptionByDefault` も `src/types.rs` に定義済み
- `ServerSideEncryptionRule::builder()` 経由で構造化入力可能
- ネスト XML (`Rule > ApplyServerSideEncryptionByDefault > SSEAlgorithm`) は issue 0065 で `for_each_element` を拡張した際の `get_nested(&[outer, inner])` パターンや、専用 XML パーサで対応済み

### 番号変更の経緯

本 issue は元々番号 0049 で作成されたが、過去の closed issue (`0049-feature-bucket-cors.md`) と番号が重複していたため、issue 台帳整理 (commit `b6437b3`) で `0070` に振り直した。

### 検証結果

- `cargo check --workspace --all-targets`: 成功
- `cargo test --test minio test_bucket_encryption`: passed (実装直後に追加された統合テスト)
- 既存 `Client::*_bucket_encryption()` の利用箇所はすべて build を通過
