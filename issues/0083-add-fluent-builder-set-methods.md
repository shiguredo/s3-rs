# Fluent Builder の set_* メソッド追加

- Priority: Medium
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/add-fluent-builder-set-methods

## 目的

aws-sdk-rust の Fluent Builder は全 `Option` フィールドに `set_*` バリアント（`Option` を直接受ける）を提供する。shiguredo_s3 では enum 化対象に限って `set_*` があり、String / SystemTime / bool 系が大量に欠落している。

## 優先度根拠

移行利用者は `builder.set_foo(None)` パターンを前提にコードを書く。欠落はコンパイルエラーではなく API 再学習コストになる。

## 現状

CHANGES.md L92「全 enum 化対象のビルダーに set_* バリアントを追加」は部分的にのみ達成。

欠落例:

- `PutObjectFluentBuilder`: `set_body`, `set_content_type`, `set_if_match` 等
- `GetObjectFluentBuilder`: `set_range`, `set_part_number`, `set_if_match` 等
- `CompleteMultipartUploadFluentBuilder`: `set_*` 0 件
- `ConfigBuilder`: `set_region`, `set_credentials_provider` 等 0 件
- PutBucket 系: `set_cors_configuration`, `set_rules` 等

## 設計方針

- 各 Fluent Builder の `Option<T>` フィールドごとに `set_*` を追加
- 既存 `foo(value)` メソッドは維持（後方互換）
- `ConfigBuilder` にも aws-sdk-rust 同等の `set_*` を追加
- `set_metadata`（HashMap 一括）を PutObject / CopyObject / CreateMultipartUpload に追加

## AWS S3 API Reference

- PutObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html>
- GetObject: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html>

## 完了条件

- 主要 Builder（Put/Get/Copy/CompleteMultipartUpload/Config）に aws-sdk-rust 相当の `set_*` が揃う
- 命名が aws-sdk-rust と一致する
- 既存テストが通る

## 解決方法

1. aws-sdk-rust ソースで各 operation builder の `set_*` 一覧を取得
2. 機械的に追加（マクロ検討可）
3. コンパイル確認
