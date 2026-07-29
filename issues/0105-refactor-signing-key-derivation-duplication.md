# signing.rs の署名計算の重複ロジックを解消する

- Priority: Medium
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/refactor-signing-duplication
- Polished: 2026-07-29

## 目的

`signing.rs` の `compute_authorization` と `compute_presigned_signature` で署名キー導出・カノニカルヘッダー構築・スコープ文字列構築のロジックが重複しており、修正漏れによるバグの温床となっている。共通関数として抽出する。

## 優先度根拠

SigV4 署名のコアロジックであり、片方だけ修正されてもう片方が取り残されると署名不一致による 403 エラーが発生する。技術的負債として早期解消が望ましい。

## 現状

`src/signing.rs:208-214` と `:275-281` で同一の 4 段階 HMAC 署名キー導出が重複:

```rust
let date_key = hmac_sha256(
    format!("AWS4{}", credentials.secret_access_key).as_bytes(),
    date_stamp.as_bytes(),
);
let date_region_key = hmac_sha256(&date_key, region.as_bytes());
let date_region_service_key = hmac_sha256(&date_region_key, b"s3");
let signing_key = hmac_sha256(&date_region_service_key, b"aws4_request");
```

`src/signing.rs:182-191` と `:247-258` でカノニカルヘッダー構築が重複:

```rust
let canonical_headers_str: String = headers
    .iter()
    .map(|(name, value)| format!("{name}:{}\n", normalize_header_value(value)))
    .collect();
let signed_headers: String = headers
    .iter()
    .map(|(name, _)| *name)
    .collect::<Vec<_>>()
    .join(";");
```

さらに `src/signing.rs:199` と `:243` でスコープ文字列の構築 (`format!("{date_stamp}/{region}/s3/aws4_request")`) も重複している。

## 設計方針

1. `fn derive_signing_key(secret_key: &str, date_stamp: &str, region: &str) -> [u8; 32]` を private 関数として抽出する（S3 専用ライブラリであるため service は `s3` に固定する）
2. `fn build_canonical_headers(headers: &[(&str, &str)]) -> (String, String)` を private 関数として抽出する。戻り値は `(canonical_headers_str, signed_headers)` の順序とする
3. スコープ文字列の構築を `fn build_scope(date_stamp: &str, region: &str) -> String` として private 関数に抽出する（`derive_signing_key` の戻り値は `[u8; 32]` のまま維持する）
4. `compute_authorization` と `compute_presigned_signature` からこれらの共通関数を呼び出すように変更する

なお `string_to_sign` の構築（`src/signing.rs:202-205` と `:269-273`）も構造的に類似しているが、入力（ペイロードハッシュ vs UNSIGNED-PAYLOAD）が異なるため本 issue のスコープ外とする。

## 完了条件

- 署名キー導出とカノニカルヘッダー構築がそれぞれ 1 つの共通関数に集約されていること
- `compute_authorization` と `compute_presigned_signature` の挙動が変更前後で同一であること
- リファクタリング前に `compute_presigned_signature` の回帰テスト（AWS 公式テストベクトル等）を追加し、挙動同一性を検証可能にすること
- 既存のテストが全て通過すること

## 解決方法

1. `compute_presigned_signature` の回帰テストを追加する（リファクタリング前の挙動を固定する。既存の `test_compute_authorization` も Signature の完全一致を検証するよう強化する）
2. `derive_signing_key` 関数を実装する
3. `build_canonical_headers` 関数を実装する
4. `build_scope` 関数を実装する
5. `compute_authorization` と `compute_presigned_signature` をリファクタリングする
6. CHANGES.md の `## develop` の `### misc` にエントリを追加する
