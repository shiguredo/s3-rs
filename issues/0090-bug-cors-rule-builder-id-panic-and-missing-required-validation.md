# CorsRuleBuilder::id の panic と build() の必須フィールド未検証を修正する

- Priority: Medium
- Created: 2026-07-07
- Model: hy3-free
- Branch: feature/fix-cors-rule-builder-validation

## AWS S3 API Reference

- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CORSRule.html>

> **ID** — Unique identifier for the rule. The value cannot be longer than 255 characters.

## 目的

本クレートの「すべてのバリデーションは `Result` を返す」方針と矛盾する `CorsRuleBuilder::id` の `panic!` を排除し、合わせて `build()` で必須フィールドの未指定を検証する。

## 優先度根拠

- 同一ファイル内の他バリデーションは `Result` を返す（`ConfigBuilder::build`、`required`、`validate_presign_expires`）のに、`id` だけ `panic!` しており、ライブラリ全体の方針と明確に矛盾する。
- バリデーションの厳格さが不揃い（ID 長だけ厳格、必須フィールドは未検証）であり、利用者が誤った設定を送信して S3 側で `MalformedXML` になるリスクがある。

## 現状

- `src/types.rs:1240-1250`（`CorsRuleBuilder::id`）は ID が 255 文字超の場合に `panic!` する。
- `src/types.rs:1311`（`CorsRuleBuilder::build`）はコメントで必須と明記の `allowed_methods` / `allowed_origins` の空チェックを行わない。
- 255 文字上限自体は仕様どおり（`API_CORSRule.md:32`）。

## 設計方針

- `id` の検証を `build()` 内で `Error::InvalidInput` を返す形に統一する（`id` メソッド自体はそのまま保持し、検証は `build()` で実施）。
- `build()` で `allowed_methods` / `allowed_origins` が空の場合も `Error::InvalidInput` を返すよう補強する。
- 他のビルダー（`ServerSideEncryptionConfigurationBuilder` 等）も同様の必須検証漏れがないか併せて確認する。

## 完了条件

- `CorsRuleBuilder` が `panic!` を含まず、検証失敗時に `Result::Err(Error::InvalidInput)` を返すこと。
- `allowed_methods` / `allowed_origins` 未指定時に `build()` がエラーを返すこと。
- `tests/` に `CorsRuleBuilder::build` のエラーパステストを追加すること。
- `CHANGES.md` の `## develop` セクションに `[FIX]` エントリを記載すること。

## 解決方法

（未着手）
