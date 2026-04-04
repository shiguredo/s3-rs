# GetBucketCors / PutBucketCors / DeleteBucketCors の実装

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 根拠

- ブラウザから直接 S3 互換エンドポイントにアクセスする構成では **CORS 設定が必須**になりやすい。
- aws-sdk-rust と同等のバケットサブリソース操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バケットの CORS（クロスオリジン）設定の取得・設定・削除を行う API を `S3Client` に追加する。

## 優先度

高

## 解決方法

- `GetBucketCorsFluentBuilder` / `PutBucketCorsFluentBuilder` / `DeleteBucketCorsFluentBuilder` を新規作成
- `CorsRule`, `GetBucketCorsOutput`, `PutBucketCorsOutput`, `DeleteBucketCorsOutput` 型を `types.rs` に追加
- CORSRule の XML パースは同名タグが複数出現するため `EventReader` で直接パースする実装
- `S3Client` にファクトリメソッドを追加
