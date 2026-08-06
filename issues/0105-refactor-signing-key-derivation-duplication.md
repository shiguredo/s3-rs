# signing.rs の署名計算の重複ロジックを解消する

- Priority: Medium
- Created: 2026-07-12
- Completed: 2026-08-06
- Model: Composer 2.5 Fast
- Branch: feature/refactor-signing-duplication
- Polished: 2026-08-02

## 目的

`signing.rs` の `compute_authorization` と `compute_presigned_signature` で署名キー導出・カノニカルヘッダー構築・スコープ文字列構築のロジックが重複しており、修正漏れによるバグの温床となっている。共通関数として抽出する。

### 参照仕様

- https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-authenticating-requests.html

> Signing key derivation: DateKey = HMAC-SHA256("AWS4" + SecretAccessKey, Date); DateRegionKey = HMAC-SHA256(DateKey, Region); DateRegionServiceKey = HMAC-SHA256(DateRegionKey, Service); SigningKey = HMAC-SHA256(DateRegionServiceKey, "aws4_request")

- https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html

> The canonical request is the same as described in the previous section, except that the payload hash is UNSIGNED-PAYLOAD.

## 優先度根拠

SigV4 署名のコアロジックであり、片方だけ修正されてもう片方が取り残されると署名不一致による 403 エラーが発生する。技術的負債として早期解消が望ましい。

## 現状

`src/signing.rs` の `compute_authorization` 関数と `compute_presigned_signature` 関数で同一の 4 段階 HMAC 署名キー導出が重複:

```rust
let date_key = hmac_sha256(
    format!("AWS4{}", credentials.secret_access_key).as_bytes(),
    date_stamp.as_bytes(),
);
let date_region_key = hmac_sha256(&date_key, region.as_bytes());
let date_region_service_key = hmac_sha256(&date_region_key, b"s3");
let signing_key = hmac_sha256(&date_region_service_key, b"aws4_request");
```

`compute_authorization` 関数と `compute_presigned_signature` 関数でカノニカルヘッダー構築が重複:

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

さらに `compute_authorization` 内の `let scope = format!("{date_stamp}/{region}/s3/aws4_request")` と `compute_presigned_signature` 内の `let scope = format!("{date_stamp}/{}/s3/aws4_request", params.region)` でスコープ文字列の構築も重複している。加えて `src/api/mod.rs` の `build_presigned_url` 関数にも同一フォーマットのスコープ構築が存在する。

## 設計方針

1. `fn derive_signing_key(secret_key: &str, date_stamp: &str, region: &str) -> [u8; 32]` を private 関数として抽出する（S3 専用ライブラリであるため service は `s3` に固定する）
2. `fn build_canonical_headers(headers: &[(&str, &str)]) -> (String, String)` を private 関数として抽出する。戻り値は `(canonical_headers_str, signed_headers)` の順序とする
3. スコープ文字列の構築を `fn build_scope(date_stamp: &str, region: &str) -> String` として `pub(crate)` 関数に抽出する（`derive_signing_key` の戻り値は `[u8; 32]` のまま維持する）。`pub(crate)` とするのは `src/api/mod.rs` の `build_presigned_url` 関数にも同一フォーマットのスコープ構築が存在するため、そちらからも利用可能にするためである
4. `compute_authorization` と `compute_presigned_signature` からこれらの共通関数を呼び出すように変更する
5. `src/api/mod.rs` の `build_presigned_url` 内のスコープ構築も `build_scope` を利用するように変更する

なお `string_to_sign` の構築（`compute_authorization` 内と `compute_presigned_signature` 内の `let string_to_sign = format!("AWS4-HMAC-SHA256\n...")` ）は両関数で構造的に同一であり、共通化も可能である。しかし差異はその入力である `canonical_request` の構築（ペイロードハッシュ vs `UNSIGNED-PAYLOAD`）にあり、`canonical_request` の構築は入力パラメータの構成が異なるため、`string_to_sign` と `canonical_request` を合わせて本 issue のスコープ外とする。また最終署名計算 `hex_encode(&hmac_sha256(&signing_key, string_to_sign.as_bytes()))` も両関数で同一に重複しているが、1 行の式であり抽出の効果は限定的なため本 issue のスコープ外とする。

## 完了条件

- 署名キー導出とカノニカルヘッダー構築がそれぞれ 1 つの共通関数に集約されていること
- スコープ文字列の構築が `pub(crate)` の共通関数に集約され、`src/api/mod.rs` の `build_presigned_url` も利用していること
- `compute_authorization` と `compute_presigned_signature` の挙動が変更前後で同一であること
- リファクタリング前に `compute_presigned_signature` の回帰テスト（AWS 公式ドキュメント https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html の GET Object 例に基づくテストベクトル）を追加し、挙動同一性を検証可能にすること
- 既存の `test_compute_authorization` が Signature の完全一致を検証するよう強化されていること（現状は `starts_with` と `contains` のみで Signature 値を検証していない）
- 既存のテストが全て通過すること

## 解決方法

`src/signing.rs` に共通関数を抽出し、重複していた 3 つのロジックを集約した:

1. `derive_signing_key` 関数 (private) を追加し、`compute_authorization` と `compute_presigned_signature` の 4 段階 HMAC 署名キー導出を置き換えた
2. `build_canonical_headers` 関数 (private) を追加し、両関数のカノニカルヘッダー構築と署名対象ヘッダー名の構築を置き換えた
3. `build_scope` 関数 (`pub(crate)`) を追加し、`compute_authorization` / `compute_presigned_signature` / `src/api/mod.rs` の `build_presigned_url` のスコープ文字列構築を置き換えた

テストは以下のとおり:

- `test_compute_presigned_signature` を追加し、AWS 公式ドキュメント (sigv4-query-string-auth.html) の GET Object 例の署名 `aeeed9bb...` と完全一致することを検証する
- `test_compute_authorization` を強化し、AWS 公式ドキュメント (sig-v4-authenticating-requests.html) の GET Object 例の署名 `f0e8bdb8...` と完全一致することを検証する
- `test_build_canonical_headers_whitespace_folding` を追加し、ヘッダー値の空白正規化 (trim と連続空白の畳み込み) を検証する
- テストのセットアップを共通の `TEST_*` const と `test_credentials` / `test_datetime` ヘルパーに集約した

CHANGES.md の `## develop` の `### misc` にリファクタリングのエントリを追加した。
