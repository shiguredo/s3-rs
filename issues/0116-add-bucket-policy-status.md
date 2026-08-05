# GetBucketPolicyStatus を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-policy-status
- Polished: 2026-08-06
- Model: GPT-5

## 目的

GetBucketPolicyStatus を追加し、bucket policy が public かどうかを aws-sdk-rust 互換に取得できるようにする。

## 現状

src/api/ に policy status 用の operation と PolicyStatus output が存在しない。

## 設計方針

- リクエスト URI は `GET /?policyStatus`。既存の `get_public_access_block.rs` と同様に、`build_signed_request` へ query param `("policyStatus", "")` を渡して構築する
- 型の構成（aws-sdk-rust 互換）:
  - `PolicyStatus` 構造体: `is_public: Option<bool>`（XML 要素名は `<IsPublic>`、aws-sdk-rust のフィールド名は `is_public`。要素名とフィールド名は混同しない）
  - `GetBucketPolicyStatusOutput`: `policy_status: Option<PolicyStatus>`（aws-sdk-rust と同じ 2 階層のネスト構造。既存の `GetPublicAccessBlockOutput` のようなフラット構造にしない）
- レスポンスのパース:
  - `<IsPublic>` の値は大文字 `TRUE` / `FALSE` と小文字 `true` / `false` の両方を受け付けて `Some(true)` / `Some(false)` に変換する（実 S3 / MinIO は大文字を返す。既存の `parse_xml_bool` は小文字のみのため、大文字対応は既存関数を変更せず本 issue 用に新規ヘルパー関数を `src/xml.rs` に実装する。実装時の統合テストで実サーバーの返す値を確認し、主張が誤っていた場合は本 issue の記述とテストを修正する）
  - `<PolicyStatus>` 要素は存在するが `<IsPublic>` が欠落している場合は `Some(PolicyStatus { is_public: None })` とする（aws-sdk-rust と同じ挙動。`extract_element` はネスト要素でエラーになるため使用せず、ルート要素の存在判定は既存の `for_each_element` を利用する。ルート要素が `<PolicyStatus>` 以外の場合は `Error::InvalidResponse` を返す）
  - 空ボディのレスポンスは XML パースを実行せず `policy_status: None` として扱う（aws-sdk-rust と同じ挙動）
  - 不正な bool 値は `Error::InvalidResponse` を返す（closed 0104 の方針）
- `expected_bucket_owner` は pending issue 0057 が全 API 横断で管理しているため、本 issue のスコープ外とする（0057 完了時に追加される）
- directory bucket（S3 Express One Zone）では GetBucketPolicyStatus は非サポートだが、directory bucket は issue 0125 で未実装のため、本 issue では directory bucket の制約をテストに反映しない。0125 完了時に制約の検証を追加する

### 変更対象ファイル

- `src/api/get_bucket_policy_status.rs` (新規作成)
- `src/api/mod.rs` (モジュール宣言 + FluentBuilder の pub use 追加)
- `src/types.rs` (`PolicyStatus`, `GetBucketPolicyStatusOutput` 追加)
- `src/xml.rs` (大文字 `TRUE` / `FALSE` 対応の新規 bool パースヘルパー関数追加。既存の `parse_xml_bool` は変更しない)
- `src/client.rs` (`get_bucket_policy_status()` メソッド追加)
- `src/lib.rs` (`PolicyStatus` の crate ルート再エクスポート追加。Output 型は既存パターンどおりルート再エクスポートしない)
- `tests/test_get_bucket_policy_status.rs` (新規作成)
- `tests/minio.rs` / `tests/rustfs.rs` (統合テストを追加。kikyo-local はバケットポリシー非対応のため `tests/kikyo.rs` には追加しない。`?policyStatus` サブリソースに未対応のサーバーは完了条件のとおり統合テスト対象外)

## 完了条件

- `GET /?policyStatus` のリクエストを正しい URI、ヘッダーで構築できる
- `<IsPublic>` の `TRUE` / `FALSE` / `true` / `false` を正しくパースし、`<IsPublic>` 欠落時は `is_public: None`、空ボディ時は `policy_status: None` になる（不正な値は `Error::InvalidResponse` になることも検証する）
- 実際の S3 互換サーバー（MinIO / RustFS）を `shiguredo_container` で起動した統合テストで検証する。kikyo-local はバケットポリシー非対応（`tests/kikyo.rs` に明記済み）のため、`tests/kikyo.rs` には統合テストを追加しない。ただし `?policyStatus` の対応有無はバケットポリシー非対応と同義ではないため、実装時に kikyo-local の対応状況を確認し、対応が確認できた場合は根拠を issue に追記してテスト追加を検討する。`?policyStatus` サブリソースに未対応のサーバーがある場合は、そのサーバー名と未対応の根拠を issue に追記した上で、対応しているサーバーでのみ統合テストを行う。TRUE / FALSE の両方を検証する（TRUE は匿名 ListBucket と `s3:PutObject` の両方を許可するバケットポリシーで行う。MinIO の IsPublic 判定は匿名 ListBucket + PutObject の両方の許可に依存し、RustFS は ListBucket または PutObject のいずれかで TRUE になる。FALSE はポリシー未設定で検証する。判定条件の主張が実サーバーの挙動と食い違う場合は、本 issue の記述とテストを修正する）
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
- CHANGES.md の `## develop` に `[ADD]` エントリを追加すること

## AWS S3 API Reference

- GetBucketPolicyStatus: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketPolicyStatus.html

> Retrieves the policy status for an Amazon S3 bucket, indicating whether the bucket is public.

> `TRUE` indicates that this bucket is public. `FALSE` indicates that the bucket is not public.

> This operation is not supported for directory buckets.

## 他 issue との依存関係

- 0057（pending: expected_bucket_owner / request_payer）は全 API 横断で管理。本 issue では対応しない（0057 完了時に追加される）
- 0125（add s3 express directory buckets）完了後に、GetBucketPolicyStatus の directory bucket 非対応の検証を追加する
- 0131（Fluent Builder の set_* 残件）は src/api/ 配下の全 Fluent Builder を対象とする。本 issue で新規追加する Fluent Builder の `set_*` メソッドは 0131 の管轄とする