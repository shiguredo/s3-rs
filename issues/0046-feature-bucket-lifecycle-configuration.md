# バケットライフサイクル設定 API を追加する

Created: 2026-03-27
Model: Opus 4.6

## 概要

S3 のバケットライフサイクル設定 API（PutBucketLifecycleConfiguration / GetBucketLifecycleConfiguration / DeleteBucketLifecycleConfiguration）を実装する。

## 根拠

ライフサイクル設定はオブジェクトの自動削除やストレージクラス移行を制御する基本的な S3 機能であり、本番運用で頻繁に利用される。この API がないと利用者は aws-sdk-rust に頼る必要があり、shiguredo_s3 単体での運用が制限される。

## 対象 API

### PutBucketLifecycleConfiguration

- メソッド: `PUT /{Bucket}?lifecycle`
- リクエストボディ: `<LifecycleConfiguration>` XML
- 主要パラメータ:
  - `bucket` (必須)
  - `lifecycle_configuration` (必須): ライフサイクルルールのコンテナ
  - `checksum_algorithm` (任意)

### GetBucketLifecycleConfiguration

- メソッド: `GET /{Bucket}?lifecycle`
- レスポンスボディ: `<LifecycleConfiguration>` XML
- 主要パラメータ:
  - `bucket` (必須)
- 出力:
  - `rules`: `Vec<LifecycleRule>`

### DeleteBucketLifecycleConfiguration

- メソッド: `DELETE /{Bucket}?lifecycle`
- レスポンスボディ: なし (204)
- 主要パラメータ:
  - `bucket` (必須)

## 必要な型定義

- `LifecycleRule`: ルールのコンテナ（id, status, filter, expiration, transitions 等）
- `LifecycleExpiration`: オブジェクト失効設定（date / days / expired_object_delete_marker）
- `LifecycleRuleFilter`: ルール適用フィルタ（prefix, tag, object_size 等）
- `LifecycleRuleAndOperator`: 複数条件の AND 結合
- `Transition`: ストレージクラス移行設定
- `TransitionStorageClass`: 移行先ストレージクラス（enum）
- `NoncurrentVersionTransition`: 非カレントバージョンの移行設定
- `NoncurrentVersionExpiration`: 非カレントバージョンの失効設定
- `AbortIncompleteMultipartUpload`: 不完全マルチパートアップロードの自動中止設定
- `ExpirationStatus`: ルールの有効/無効（enum）

## 実装方針

- aws-sdk-rust 互換の API を提供する
- 既存の tagging API (put/get/delete_bucket_tagging) と同様のパターンで実装する
- XML の構築は `XmlWriter`、パースは `for_each_element` / `extract_element` を使う
- `expected_bucket_owner` や `transition_default_minimum_object_size` は初期実装では対応しない（MinIO/RustFS が未対応のため）

## 実装順序

1. 型定義を `src/types.rs` に追加する
2. `DeleteBucketLifecycleConfiguration` を実装する（最もシンプル）
3. `PutBucketLifecycleConfiguration` を実装する（XML 構築が必要）
4. `GetBucketLifecycleConfiguration` を実装する（XML パースが必要）
5. 統合テストを追加する
