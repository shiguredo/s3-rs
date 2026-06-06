# CorsRule ID の XML serialize / parse が未実装

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-06-06
- Completed: 2026-06-06
- Branch: feature/fix-cors-rule-id-roundtrip

## 目的

`CorsRule.id` が PutBucketCors で XML 出力されず、GetBucketCors でパースされないため、CORS ルール ID のラウンドトリップが成立しない。aws-sdk-rust 互換のため ID 要素を serialize / parse する。

## 優先度根拠

CORS ルール ID は運用上ルール識別に使われる。PUT 後 GET で ID が消失すると、設定管理ツールや IaC が誤動作する。

## 現状

- `src/types.rs:1131`: `CorsRule.id: Option<String>` フィールドは存在する
- `src/types.rs:1162-1167`: `CorsRuleBuilder::id` は存在するがバリデーションなし（`self.id = Some(id.into())` のみ）
- `src/api/put_bucket_cors.rs:97-125`: `build_cors_xml` が `ID` 要素を出力しない
- `src/api/get_bucket_cors.rs:63-133`: `extract_cors_rules` が `"ID"` 分岐を持たず、常に `id: None`

## 設計方針

### ID 要素の XML シリアライズ

`put_bucket_cors.rs` の `build_cors_xml` で `rule.id` が `Some` のとき `<ID>` を出力する。AWS S3 API の XML スキーマでは `<ID>` は `<AllowedOrigin>` より前に配置されるため、`w.start("CORSRule")` の直後に出力する。

### ID 要素の XML パース

`get_bucket_cors.rs` の `extract_cors_rules` に `let mut id: Option<String> = None;` を宣言し、`CORSRule` 開始時に `None` にリセットする。`"ID"` 分岐を追加し `id = Some(current_text.clone())` を設定する。`CorsRule` 構築時に `id: id.take()` で設定する。

### ID のバリデーション

AWS S3 API Reference の「The value cannot be longer than 255 characters.」に基づき、`CorsRuleBuilder::id` で 255 文字超えの場合に `Error::InvalidInput` を返すバリデーションを追加する。空文字列は許容する（AWS S3 API 仕様で禁止されていないため）。

### 0075 との関係

issue 0075 で `extract_cors_rules` の戻り値を `Result<Vec<CorsRule>, Error>` に変更する。0076 は 0075 の完了後に実装する。`ID` パース失敗時（文字列取得に失敗した場合）は `Error::InvalidResponse` を返す。

## AWS S3 API Reference

- PutBucketCors: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketCors.html>
- GetBucketCors: <https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketCors.html>

> ID: Unique identifier for the rule. The value cannot be longer than 255 characters.

## 完了条件

- ID 付き CORS ルールを PUT し GET すると同一 ID が返る
- ID なしルールの既存挙動が維持される
- `CorsRuleBuilder::id` で 255 文字超えの入力が `Error::InvalidInput` になる
- 空文字列の ID は正常にラウンドトリップする
- MinIO / RustFS 統合テストで CORS ラウンドトリップを検証する
- 0075 で作成される `tests/test_get_bucket_cors.rs` に ID のパーステストを追加する

## 解決方法

1. `put_bucket_cors.rs`: `build_cors_xml` で `rule.id` が `Some` のとき `<ID>` 要素を `w.start("CORSRule")` の直後に出力するように修正
2. `get_bucket_cors.rs`: `extract_cors_rules` に `id` 変数を追加し、`"ID"` 分岐と `id.take()` による CorsRule 構築を実装
3. `types.rs`: `CorsRuleBuilder::id` に 255 文字制限バリデーションを追加（超過で panic）
4. `tests/test_get_bucket_cors.rs`: ID パーステスト、ID なし後方互換テスト、255 文字パニックテスト、空文字列許容テストを追加
