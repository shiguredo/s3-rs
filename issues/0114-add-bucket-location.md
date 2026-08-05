# GetBucketLocation を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-location
- Polished: 2026-08-06
- Model: GPT-5

## 目的

GetBucketLocation を追加し、bucket の region / location constraint を aws-sdk-rust 互換に取得できるようにする。

## 現状

src/api/ に GetBucketLocation がなく、LocationConstraint のレスポンスを表す型も存在しない。`CreateBucketConfiguration.location_constraint` は `Option<String>` で入力側のみ扱われている。

## 設計方針

- リクエスト URI は `GET /?location`。`src/api/get_bucket_versioning.rs` の query param `("versioning", "")` と同様に、`build_signed_request` へ `("location", "")` を渡して構築する
- aws-sdk-rust の `GetBucketLocationOutput`（`location_constraint: Option<BucketLocationConstraint>`）と `BucketLocationConstraint` に合わせる。`BucketLocationConstraint` の variant は aws-sdk-rust と一致させ、`UsEast1` variant は存在しない（us-east-1 は空レスポンスとして扱う）。`EU` は aws-sdk-rust の `Eu` variant に対応する。enum の実装は `src/types.rs` の enum 方針（`#[non_exhaustive]`、`as_str()` / `From<&str>` / `Display`、`Unknown(String)` 前方互換）に準拠する
- レスポンスの空ボディ（`response.body` が空）は XML パースを実行せず `location_constraint: None` として扱う。空要素 / 空文字列は `extract_element` が `Some("")` を返すため、空文字列を `None` に変換する。それ以外の LocationConstraint 値（リージョン名、`EU` 値等）は対応する `BucketLocationConstraint` に変換する（`us-east-1` は variant が存在しないため、aws-sdk-rust と同じく `Unknown("us-east-1")` になる）
- `expected_bucket_owner` は pending issue 0057 が全 API 横断で管理しているため、本 issue のスコープ外とする（0057 完了時に追加される）
- GetBucketLocation は AWS が HeadBucket の利用を推奨しているが、aws-sdk-rust の API 表面互換のために実装する（CODEBASE.md の方針に従う）
- directory bucket（S3 Express One Zone）では GetBucketLocation は非サポートだが、directory bucket は issue 0125 で未実装のため、本 issue では directory bucket の制約をテストに反映しない。0125 完了時に制約の検証を追加する
- closed issue 0059 で後回しにされた `BucketLocationConstraint` を、関連機能である GetBucketLocation の実装に合わせて導入する
- `CreateBucketConfiguration.location_constraint` の `Option<String>` から `Option<BucketLocationConstraint>` への変更は本 issue のスコープ外とする（0130 にも含まれないため、必要なら別 issue で扱う）

### 変更対象ファイル

- `src/api/get_bucket_location.rs` (新規作成)
- `src/api/mod.rs` (モジュール宣言 + FluentBuilder の pub use 追加)
- `src/types.rs` (`BucketLocationConstraint`, `GetBucketLocationOutput` 追加)
- `src/client.rs` (`get_bucket_location()` メソッド追加)
- `src/lib.rs` (`BucketLocationConstraint` enum の crate ルート再エクスポート追加。`GetBucketLocationOutput` は既存パターンどおりルート再エクスポートしない)
- `tests/test_get_bucket_location.rs` (新規作成)
- `tests/minio.rs` / `tests/rustfs.rs` / `tests/kikyo.rs` (統合テストを追加。`?location` サブリソースに未対応のサーバーは完了条件のとおり統合テスト対象外)

## 完了条件

- `GET /?location` のリクエストを正しい URI、ヘッダーで構築できる
- XML の LocationConstraint を正しくパースし、空ボディ / 空 XML は `None`、リージョン名・`EU` 値は対応する `BucketLocationConstraint` に変換できる（未知の値は `Unknown(String)` になるパースも検証する）
- 実際の S3 互換サーバー（MinIO / RustFS / kikyo-local）を `shiguredo_container` で起動した統合テストで検証する。`?location` サブリソースに未対応のサーバーがある場合は、そのサーバー名と未対応の根拠を issue に追記した上で、対応しているサーバーでのみ統合テストを行う
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
- CHANGES.md の `## develop` に `[ADD]` エントリを追加すること

## AWS S3 API Reference

- GetBucketLocation: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLocation.html

> Returns the Region the bucket resides in.

> Buckets in Region `us-east-1` have a LocationConstraint of `null`. Buckets with a LocationConstraint of `EU` reside in `eu-west-1`.

> Using the `GetBucketLocation` operation is no longer a best practice. To return the Region that a bucket resides in, we recommend that you use the `HeadBucket` operation instead.

> This operation is not supported for directory buckets.

## 他 issue との依存関係

- 0057（pending: expected_bucket_owner / request_payer）は全 API 横断で管理。本 issue では対応しない
- 0125（add s3 express directory buckets）完了後に、GetBucketLocation の directory bucket 非対応の検証を追加する