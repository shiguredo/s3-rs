# Bucket / Object ACL API を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-acl-apis
- Polished: 2026-08-03
- Model: GPT-5

## 目的

GetBucketAcl、PutBucketAcl、GetObjectAcl、PutObjectAcl を追加し、ACL を利用する S3 互換ストレージとの API 互換性を高める。

## 現状

CreateBucket / PutObject / CopyObject / CreateMultipartUpload に canned ACL（`x-amz-acl` ヘッダー、`ObjectCannedAcl` enum）が存在するが、bucket / object の ACL を取得・設定する operation と AccessControlPolicy、Grant、Grantee のモデルが存在しない。

## 設計方針

- GetBucketAcl / PutBucketAcl / GetObjectAcl / PutObjectAcl の builder を追加する。リクエスト URI は bucket が `GET /?acl` と `PUT /?acl`、object が `GET /{Key}?acl` と `PUT /{Key}?acl`（object のみ `versionId` クエリパラメータを追加で扱う）
- 型の構成（aws-sdk-rust 互換）:
  - `Type` enum: `AmazonCustomerByEmail` / `CanonicalUser` / `Group` / `Unknown(String)` の 4 variant。Grantee の種別（XML の `xsi:type` 属性に対応）を表す
  - `Permission` enum: `FullControl` / `Read` / `ReadAcp` / `Write` / `WriteAcp` / `Unknown(String)` の 6 variant
  - `Grantee` 構造体: `display_name` / `email_address` / `id` / `uri` は `Option<String>`、`r#type: Type`（aws-sdk-rust と同じく必須）
  - `Grant` 構造体: `grantee: Option<Grantee>` / `permission: Option<Permission>`
  - `AccessControlPolicy` 構造体: `owner: Option<Owner>`（`src/types.rs` の既存 `Owner` 型を再利用する）/ `grants: Option<Vec<Grant>>`
  - `BucketCannedAcl` enum: `AuthenticatedRead` / `Private` / `PublicRead` / `PublicReadWrite` / `Unknown(String)` の 5 variant（PutBucketAcl の `acl` 用。aws-sdk-rust と同じ 4 値 + Unknown）
  - `GetBucketAclOutput`: `owner: Option<Owner>` / `grants: Option<Vec<Grant>>`
  - `PutBucketAclOutput`: フィールドなし（空構造体）
  - `GetObjectAclOutput`: `owner: Option<Owner>` / `grants: Option<Vec<Grant>>` / `request_charged: Option<String>`
  - `PutObjectAclOutput`: `request_charged: Option<String>`
  - 全 enum に `#[non_exhaustive]`、`as_str()` / `From<&str>` / `Display` を実装（`src/types.rs` の enum 方針に準拠）
- `Grantee` / `Grant` / `AccessControlPolicy` は既存の `CorsRuleBuilder` / `TaggingBuilder` と同じ builder パターンで構築する。`GranteeBuilder::build()` は `r#type` 未指定時に `Error::InvalidInput` を返す（aws-sdk-rust の GranteeBuilder と同じく `type` は必須）
- PutObjectAcl の `acl` は既存の `ObjectCannedAcl` を再利用する（aws-sdk-rust は PutObjectAcl に ObjectCannedAcl、PutBucketAcl に BucketCannedAcl を使う）
- PutBucketAcl / PutObjectAcl の入力は 3 モード（AWS 仕様により三者択一）:
  1. `acl`（canned ACL、`x-amz-acl` ヘッダー）
  2. `grant_full_control` / `grant_read` / `grant_read_acp` / `grant_write` / `grant_write_acp`（`x-amz-grant-*` ヘッダー、`String` 型）
  3. `access_control_policy`（AccessControlPolicy の XML ボディ）
  - ボディとヘッダーの同時指定、および `acl` と `grant_*` の同時指定は `Error::InvalidInput` を返す（AWS 仕様が同時指定を禁止している。既存の PutBucketWebsite の排他検証と同じ方針）
  - 3 モードの全てが未指定の場合も `Error::InvalidInput` を返す（既存の PutBucketEncryption / PutBucketWebsite の空入力検証と同じ方針）
- XML の `xsi:type` 属性を失わずに扱うため、`src/xml.rs` に属性サポートを追加する:
  - パース側: `for_each_element` / `ChildElements` は現在属性を全て破棄しているため、属性にアクセスできる機構を追加する
  - 生成側: `XmlWriter` に属性出力を追加する（`<Grantee xmlns:xsi="..." xsi:type="CanonicalUser">` 形式の出力が必要）
- AccessControlPolicy の XML は `<Grant>` 要素を `<AccessControlList>` 要素が包む構造（`<Grants>` ではない）。生成・パースの両方で `<AccessControlList>` ラッパー要素を扱う
- レスポンスの `<Grantee>` に `xsi:type` 属性が欠落している場合は `Error::InvalidResponse` を返す（aws-sdk-rust が必須フィールド欠落をデシリアライズエラーにするのと同等の方針）
- PutBucketAcl / PutObjectAcl の XML ボディに対する Content-MD5 と x-amz-sdk-checksum-algorithm は既存の XML API（`put_bucket_encryption.rs` 等）と同じ方針で処理する
- GetObjectAclOutput / PutObjectAclOutput の `request_charged` は `x-amz-request-charged` レスポンスヘッダーから取得し、既存 output と同じく `Option<String>` 型にする（`RequestCharged` enum への型化は 0130 の管轄。型化時にこれらも含めて更新される）
- `expected_bucket_owner` / `request_payer` は pending issue 0057 が全 API 横断で管理しているため、本 issue のスコープ外とする（0057 完了時に追加される）
- 他 operation（PutObject / CopyObject / CreateBucket / CreateMultipartUpload）の `grant_*` ヘッダーは issue 0126 の管轄。PutBucketAcl / PutObjectAcl 自身の `grant_*` ヘッダーは本 issue のスコープ
- CreateBucket の `acl` が `ObjectCannedAcl` を使っている点（aws-sdk-rust は `BucketCannedAcl`）は本 issue では変更しない（変更は破壊的変更を伴う change カテゴリの作業であり、必要なら別 issue で扱う）
- ACL が無効化された bucket（Object Ownership の BucketOwnerEnforced）では、AWS は Put 系を `AccessControlListNotSupported` エラーで失敗させ、Get 系は ACL の読み取りを引き続きサポートする。これは Object Ownership に依存する AWS 固有機能であり、S3 互換サーバーが再現するかどうかは実装時に AWS 公式ドキュメントまたは実レスポンスで確認し、本 issue のブロッカーにしない
- 4 operation いずれも directory bucket（S3 Express One Zone）では利用できないが、directory bucket は issue 0125 で未実装のため、本 issue では directory bucket の制約をテストに反映しない。0125 完了時に制約の検証を追加する
- AWS は 2025-10-01 に Email Grantee ACL を廃止済み（該当リージョンで HTTP 405）だが、互換性のため `Grantee` の `email_address` フィールドは保持する。統合テストでは EmailAddress 指定の grant を検証しない

### 変更対象ファイル

- `src/api/get_bucket_acl.rs` (新規作成)
- `src/api/put_bucket_acl.rs` (新規作成)
- `src/api/get_object_acl.rs` (新規作成)
- `src/api/put_object_acl.rs` (新規作成)
- `src/api/mod.rs` (モジュール宣言 + FluentBuilder の pub use 追加)
- `src/types.rs` (`Type`, `Permission`, `Grantee`, `Grant`, `AccessControlPolicy`, `BucketCannedAcl`, `GetBucketAclOutput`, `PutBucketAclOutput`, `GetObjectAclOutput`, `PutObjectAclOutput` 追加)
- `src/xml.rs` (パース・生成の属性サポート追加)
- `src/client.rs` (`get_bucket_acl()` / `put_bucket_acl()` / `get_object_acl()` / `put_object_acl()` メソッド追加)
- `src/lib.rs` (新しいモデル型の crate ルート再エクスポート追加。Output 型は既存パターンどおりルート再エクスポートしない)
- `tests/test_acl.rs` (新規作成。4 operation のリクエスト構築・レスポンスパースのテスト)
- `tests/minio.rs` / `tests/rustfs.rs` (ACL の統合テストを追加。kikyo-local は ACL 非対応のため `tests/kikyo.rs` には追加しない)

## 完了条件

- `GET /?acl` / `PUT /?acl`（bucket）と `GET /{Key}?acl` / `PUT /{Key}?acl`（object、`versionId` 付きを含む）のリクエストを正しい URI、ヘッダー、XML で構築できる
- `xsi:type` 属性付きの AccessControlPolicy レスポンス（CanonicalUser / Group / AmazonCustomerByEmail）を正しくパースできる（未知の Permission / Type の値は `Unknown(String)` としてパースされることも検証する）
- canned ACL、`grant_*` ヘッダー、AccessControlPolicy XML ボディの 3 モードの入力を扱え、同時指定は `Error::InvalidInput` となる
- 実際の S3 互換サーバー（MinIO / RustFS / kikyo-local）を `shiguredo_container` で起動した統合テストで検証する。kikyo-local は ACL 非対応（`tests/kikyo.rs` に明記済み）のため、対応しているサーバーでのみ統合テストを行う。その他のサーバーで `?acl` サブリソースに未対応がある場合は、そのサーバー名と未対応の根拠を issue に追記した上で、対応しているサーバーでのみ統合テストを行う
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
- CHANGES.md の `## develop` に `[ADD]` エントリを追加すること

## AWS S3 API Reference

- GetBucketAcl: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketAcl.html

> This implementation of the GET action uses the acl subresource to return the access control list (ACL) of a bucket.

- PutBucketAcl: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketAcl.html

> Sets the permissions on an existing bucket using access control lists (ACL).

- GetObjectAcl: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectAcl.html

> Returns the access control list (ACL) of an object.

- PutObjectAcl: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectAcl.html

> Uses the acl subresource to set the access control list (ACL) permissions for a new or existing object in an S3 bucket.

## 他 issue との依存関係

- 0057（pending: expected_bucket_owner / request_payer）は全 API 横断で管理。本 issue では対応しない
- 0126（operation 固有入力項目）は PutObject / CopyObject / CreateBucket / CreateMultipartUpload の `grant_*` ヘッダーを管理。PutBucketAcl / PutObjectAcl 自身の `grant_*` ヘッダーは本 issue で対応する
- 0130（S3 model の enum / nested structure 化）の対象一覧に `BucketCannedAcl` が含まれているが、`BucketCannedAcl` は PutBucketAcl の実装に必須のため本 issue が追加する（関連機能を実装する際にその一部として enum 化を導入するという closed 0059 / 0084 の方針に従う）。`BucketCannedAcl` は 0130 の対象一覧から削除する（重複導入を防ぐため、0130 の実装着手時までに対応する）
- `RequestCharged` enum の型化は 0130 の管轄。本 issue は既存 output と整合する `Option<String>` で追加する
