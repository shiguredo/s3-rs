# signing.rs の署名キー導出とカノニカルヘッダー構築の重複を解消する

- Priority: Medium
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/refactor-signing-key-derivation-duplication

## 目的

`signing.rs` の `compute_authorization` と `compute_presigned_signature` で署名キー導出とカノニカルヘッダー構築ロジックが重複しており、修正漏れによるバグの温床となっている。共通関数として抽出する。

## 優先度根拠

SigV4 署名のコアロジックであり、片方だけ修正されてもう片方が取り残されると署名不一致による 403 エラーが発生する。技術的負債として早期解消が望ましい。

## 現状

`src/signing.rs:208-214` と `:275-281` で同一の 4 段階 HMAC 署名キー導出が重複:

```rust
let date_key = hmac_sha256(format!("AWS4{}", secret).as_bytes(), date_stamp.as_bytes());
let date_region_key = hmac_sha256(&date_key, region.as_bytes());
let date_region_service_key = hmac_sha256(&date_region_key, b"s3");
let signing_key = hmac_sha256(&date_region_service_key, b"aws4_request");
```

`src/signing.rs:182-190` と `:247-256` でカノニカルヘッダー構築が重複:

```rust
let canonical_headers_str: String = headers
    .iter()
    .map(|(name, value)| format!("{name}:{}\n", normalize_header_value(value)))
    .collect();
let signed_headers: String = headers.iter().map(|(name, _)| *name).collect::<Vec<_>>().join(";");
```

## 設計方針

1. `fn derive_signing_key(secret_key: &str, date_stamp: &str, region: &str) -> [u8; 32]` を抽出する
2. `fn build_canonical_headers(headers: &[(&str, &str)]) -> (String, String)` を抽出する
3. `compute_authorization` と `compute_presigned_signature` からこれらの共通関数を呼び出すように変更する

## 完了条件

- 署名キー導出とカノニカルヘッダー構築がそれぞれ 1 つの共通関数に集約されていること
- `compute_authorization` と `compute_presigned_signature` の挙動が変更前後で同一であること
- 既存のテストが全て通過すること

## 解決方法

1. `derive_signing_key` 関数を実装する
2. `build_canonical_headers` 関数を実装する
3. `compute_authorization` と `compute_presigned_signature` をリファクタリングする
4. CHANGES.md の `## develop` の `### misc` にエントリを追加する
