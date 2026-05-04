# GetObjectTagging / PutObjectTagging / DeleteObjectTagging の実装

Created: 2026-04-04
Completed: 2026-05-04
Model: Composer 2 Fast

## 根拠

- オブジェクトタグはライフサイクルルールやコスト配分・運用ラベルで広く使われ、**アップロード時の `x-amz-tagging` だけでは既存オブジェクトの取得・変更・削除に対応できない**。
- aws-sdk-rust と同等の操作セットを揃え、利用者が移行しやすくするため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

既存オブジェクトに対するタグの取得・設定・削除を行う API を `Client` に追加する。バケットタグ（`GetBucketTagging` 等）とは別操作である。

3 API とも `versionId` クエリパラメータを受け取り、バージョニング有効バケットでは特定バージョンを対象にできる。aws-sdk-rust 互換の API 形状にするため、builder に `version_id` フィールドを含めること。

## 対象 API

| 操作 | 説明 |
|------|------|
| GetObjectTagging | オブジェクトに付いたタグ（キー・値）を取得する |
| PutObjectTagging | オブジェクトのタグを置き換える |
| DeleteObjectTagging | オブジェクトからタグをすべて削除する |

## AWS 公式ドキュメント（API リファレンス）

- [GetObjectTagging](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectTagging.html)
- [PutObjectTagging](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectTagging.html)
- [DeleteObjectTagging](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjectTagging.html)

## 関連

- アップロード時にタグを付ける `x-amz-tagging` ヘッダーは [#0039](0039-feature-tagging-params.md) で扱う。本 issue は **オブジェクト単位のタグ CRUD** である。

## 優先度

高

## 解決方法

### 実装状況

本 issue が要求する 3 API はいずれも、過去のリリースで既に実装済みであり、本 issue の closed 化時点で `src/api/get_object_tagging.rs` / `src/api/put_object_tagging.rs` / `src/api/delete_object_tagging.rs` として動作している。

- `Client::get_object_tagging()` / `put_object_tagging()` / `delete_object_tagging()` を提供
- 3 API とも `version_id(impl Into<String>)` ビルダーメソッド経由で `versionId` クエリパラメータを送信可能
- `GetObjectTaggingFluentBuilder::parse_response` で XML レスポンスから `Tagging.TagSet` を `Vec<Tag>` に変換
- `PutObjectTaggingFluentBuilder` は `tagging(Tagging)` 経由で構造化入力を受ける (issue 0061 で `DeleteObjects` を `Delete` 構造体経由に変更したのと同じ方針)
- `DeleteObjectTaggingFluentBuilder` は `bucket` / `key` / `version_id` のみで動作

### 番号変更の経緯

本 issue は元々番号 0046 で作成されたが、過去の closed issue (`0046-feature-bucket-lifecycle-configuration.md`) と番号が重複していたため、issue 台帳整理 (commit `b6437b3`) で `0069` に振り直した。

### 検証結果

- `cargo check --workspace --all-targets`: 成功
- `cargo test --test minio test_object_tagging`: passed (実装直後に追加された統合テスト)
- 既存 `Client::*_object_tagging()` の利用箇所はすべて build を通過
