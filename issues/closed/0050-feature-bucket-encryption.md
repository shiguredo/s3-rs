# GetBucketEncryption / PutBucketEncryption / DeleteBucketEncryption の実装

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 根拠

- コンプライアンス・セキュリティ要件で **バケット既定の SSE** をコードや IaC で管理する需要が高い。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットのデフォルト暗号化（サーバーサイド暗号化の既定）の取得・設定・削除を行う API を `S3Client` に追加する。

## 優先度

高

## 解決方法

- `GetBucketEncryptionFluentBuilder` / `PutBucketEncryptionFluentBuilder` / `DeleteBucketEncryptionFluentBuilder` を新規作成
- `ServerSideEncryptionRule`, `ServerSideEncryptionByDefault`, `GetBucketEncryptionOutput`, `PutBucketEncryptionOutput`, `DeleteBucketEncryptionOutput` 型を `types.rs` に追加
- XML 構造が `Rule > ApplyServerSideEncryptionByDefault > SSEAlgorithm` とネストされるため `EventReader` で直接パースする実装
- `S3Client` にファクトリメソッドを追加
- fuzz ターゲットに `GetBucketEncryptionFluentBuilder` を追加
