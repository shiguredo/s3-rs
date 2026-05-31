# CorsRule ID の XML serialize / parse が未実装

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-05-31

## 目的

`CorsRule.id` が PutBucketCors で XML 出力されず、GetBucketCors でパースされないため、CORS ルール ID のラウンドトリップが成立しない。aws-sdk-rust 互換のため ID 要素を serialize / parse する。

## 優先度根拠

CORS ルール ID は運用上ルール識別に使われる。PUT 後 GET で ID が消失すると、設定管理ツールや IaC が誤動作する。

## 現状

- `src/types.rs`: `CorsRule.id` フィールドと `CorsRuleBuilder::id` は存在する
- `src/api/put_bucket_cors.rs:97-125`: `build_cors_xml` が `ID` 要素を出力しない
- `src/api/get_bucket_cors.rs:63-133`: `extract_cors_rules` が `"ID"` 分岐を持たず、常に `id: None`

## 設計方針

### ID 要素の XML シリアライズ

`build_cors_xml` で `rule.id` が `Some` のとき `<ID>` を出力する。AWS S3 API の XML スキーマでは `<ID>` は `<AllowedOrigin>` より前に配置されるため、`w.start("CORSRule")` の直後に出力する。

### ID 要素の XML パース

`extract_cors_rules` に `let mut id: Option<String> = None;` を宣言し、`CORSRule` スタート時に `None` にリセットする。`"ID"` 分岐を追加し、`id = Some(current_text.clone())` を設定する。`CorsRule` 構築時に `id` フィールドに設定する。

### ID のバリデーション

AWS S3 API Reference の「The value cannot be longer than 255 characters.」に基づき、`CorsRuleBuilder::id` で 255 文字超えの場合に `Error::InvalidInput` を返すバリデーションを追加する。空文字列は許容する（AWS S3 API 仕様で禁止されていないため）。

### 0075 との関係

issue 0075 で `extract_cors_rules` の戻り値を `Result<Vec<CorsRule>, Error>` に変更する。0076 は 0075 の完了後に実装し、`ID` パース失敗時は `Error::InvalidResponse` を返す。

## AWS S3 API Reference

- PutBucketCors: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketCors.html>
- GetBucketCors: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketCors.html>

> ID: Unique identifier for the rule. The value cannot be longer than 255 characters.

## 完了条件

- ID 付き CORS ルールを PUT し GET すると同一 ID が返る
- ID なしルールの既存挙動が維持される
- 255 文字超えの ID で `Error::InvalidResponse` が返る
- MinIO / RustFS 統合テストで CORS ラウンドトリップを検証する

## 解決方法

1. `types.rs` の `CorsRuleBuilder::id` に 255 文字制限バリデーションを追加
2. `put_bucket_cors.rs` の `build_cors_xml` で `w.start("CORSRule")` の直後に `ID` を出力
3. `get_bucket_cors.rs` の `extract_cors_rules` に `id` 変数と `"ID"` 分岐を追加
4. 統合テストで ID 付きルールの PUT / GET を追加
