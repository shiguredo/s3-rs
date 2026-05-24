# CorsRule ID の XML serialize / parse が未実装

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/fix-cors-rule-id-roundtrip

## 目的

`CorsRule.id` が PutBucketCors で XML 出力されず、GetBucketCors でパースされないため、CORS ルール ID のラウンドトリップが成立しない。aws-sdk-rust 互換のため ID 要素を serialize / parse する。

## 優先度根拠

CORS ルール ID は運用上ルール識別に使われる。PUT 後 GET で ID が消失すると、設定管理ツールや IaC が誤動作する。

## 現状

- `src/types.rs`: `CorsRule.id` フィールドと `CorsRuleBuilder::id` は存在する
- `src/api/put_bucket_cors.rs:97-125`: `build_cors_xml` が `ID` 要素を出力しない
- `src/api/get_bucket_cors.rs:63-133`: `extract_cors_rules` が `"ID"` 分岐を持たず、常に `id: None`

## 設計方針

- `build_cors_xml` で `rule.id` が `Some` のとき `<ID>` を出力する
- `extract_cors_rules` に `"ID"` 分岐を追加する
- aws-sdk-rust の `CorsRule.id` と同じ意味論とする

## AWS S3 API Reference

- PutBucketCors: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketCors.html>
- GetBucketCors: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketCors.html>

> ID: Unique identifier for the rule. The value cannot be longer than 255 characters.

## 完了条件

- ID 付き CORS ルールを PUT し GET すると同一 ID が返る
- ID なしルールの既存挙動が維持される
- MinIO / RustFS 統合テストで CORS ラウンドトリップを検証する

## 解決方法

1. `build_cors_xml` に `if let Some(ref id) = rule.id { w.element("ID", id); }` を追加
2. `extract_cors_rules` に `"ID" => id = Some(current_text.clone())` を追加
3. 統合テストで ID 付きルールの PUT / GET を追加
