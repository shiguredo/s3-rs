# docs/AWS_SDK_RUST.md の対応方針を再分類する

Created: 2026-05-04
Completed: 2026-05-04
Model: Opus 4.7

## 根拠

- `docs/AWS_SDK_RUST.md` には現状「対応予定無し」として 19 種のパラメータが列挙されている (例: `expected_bucket_owner`, `request_payer`, `grant_*`, `object_lock_*`, `bucket_key_enabled`, `ssekms_encryption_context`, `website_redirect_location`, `mfa`, `bypass_governance_retention` 等)。
- 一方、`AGENTS.md` および `CLAUDE.md` には「Amazon S3 API の仕様と aws-sdk-rust との互換性を最優先にする」と明記されており、`docs/AWS_SDK_RUST.md` の「対応予定無し」表記と矛盾している。
- 「対応予定無し」のうち、互換性のためにヘッダー渡しのみで実装が軽いもの (`expected_bucket_owner`, `request_payer`, `grant_*`, `website_redirect_location` 等) と、機能本格対応が必要なもの (`object_lock_*`, `mfa`, `bypass_governance_retention` 等) を **再分類** する必要がある。
- 既に issue 0057 (`pending/0057-feature-expected-bucket-owner-request-payer.md`) で `expected_bucket_owner` / `request_payer` の取り扱いが pending になっており、本 issue で方針整理した上で `pending/` から `issues/` に戻す。

## 変更内容

本 issue は **`docs/AWS_SDK_RUST.md` の対応方針再分類のみ** に絞る。実装側の出力フィールド補完は別 issue に分割した:

- `issues/0065-add-object-and-list-buckets-output-fields.md`: `Owner` / `RestoreStatus` 型新設、`Object` / `ObjectVersion` / `ListBucketsOutput` への追加
- `issues/0066-add-get-and-head-object-output-fields.md`: `GetObjectOutput` / `HeadObjectOutput` のフィールド補完
- `issues/0067-add-put-object-and-related-output-fields.md`: `PutObjectOutput` / `UploadPartOutput` / `CompleteMultipartUploadOutput` / `CopyObjectOutput` の SSE/checksum 補完
- `issues/0068-add-object-identifier-and-head-bucket-output-fields.md`: `ObjectIdentifier` / `CompletedPart` / `HeadBucketOutput` の補完

### 1. `docs/AWS_SDK_RUST.md` の方針再分類

「対応予定無し」を以下に再分類する。

#### 「未対応 (互換性のため対応予定)」に格上げ

ヘッダー渡しまたは値の追加のみで実装が軽量なもの:

| パラメータ | 該当 API | 理由 |
|---|---|---|
| `expected_bucket_owner` | 全 API | issue 0057 で対応 |
| `request_payer` | 全 API | issue 0057 で対応、enum 化は issue 0059 後続 |
| `website_redirect_location` | PutObject, CopyObject, CreateMultipartUpload | `x-amz-website-redirect-location` ヘッダー追加のみ |
| `grant_full_control` / `grant_read` / `grant_read_acp` / `grant_write` / `grant_write_acp` | PutObject, CopyObject, CreateBucket, CreateMultipartUpload | レガシー ACL だが API は単純 (文字列ヘッダー) |

#### 「入力は対応予定無し、出力は対応」(出力のみ対応)

入力側は機能本格対応とセットになるため保留するが、**出力側はレスポンスをパースして提供するだけで実装コストが軽量** であり、利用者が結果を確認できると有用なもの:

| パラメータ | 入力 (該当 API) | 出力 (該当 *Output) | 理由 |
|---|---|---|---|
| `bucket_key_enabled` | PutObject 等の入力で対応予定無し | `GetObjectOutput` / `HeadObjectOutput` / `PutObjectOutput` 等で対応 | 入力は KMS 機能本格対応とセット、出力は `x-amz-server-side-encryption-bucket-key-enabled` ヘッダーをパースするだけ |
| `ssekms_encryption_context` | PutObject 等の入力で対応予定無し | `CopyObjectOutput` 等で対応 | 入力は KMS 詳細機能、出力は `x-amz-server-side-encryption-context` ヘッダーをパースするだけ |
| `ssekms_key_id` | PutObject 等の入力で対応予定無し | `GetObjectOutput` / `PutObjectOutput` 等で対応 | 入力は KMS 詳細機能、出力は `x-amz-server-side-encryption-aws-kms-key-id` ヘッダーをパースするだけ |
| `sse_customer_algorithm` / `sse_customer_key_md5` | 既存の SSE-C 入力対応の延長 | 上記 Output で対応 | 出力ヘッダーをパースするだけ |

「入力は未対応、出力のみ対応」とする理由は、出力フィールドの追加は **互換ストレージの動作を確認できるようになる利点** がある一方、追加コストは XML/ヘッダーのパース処理拡張のみで小さいため。

#### 「対応予定無し」維持

機能の本格対応とセットになるため保留:

| パラメータ | 理由 |
|---|---|
| `object_lock_legal_hold_status`, `object_lock_mode`, `object_lock_retain_until_date` | Object Lock 機能本格対応とセット (別 issue) |
| `object_lock_enabled_for_bucket` | 同上 |
| `mfa`, `bypass_governance_retention` | Object Lock / Versioning MFA 機能とセット |
| `if_match_initiated_time`, `if_match_last_modified_time`, `if_match_size` | S3 固有の条件付き機能、AWS SDK でも限定 |
| `mpu_object_size`, `write_offset_bytes` | S3 Express One Zone 専用機能 |
| `optional_object_attributes`, `fetch_owner` | AWS IAM ベース、S3 互換ストレージで意味が薄い |
| `confirm_remove_self_bucket_access` | AWS 固有の安全装置 |
| `object_ownership` | バケットオーナーシップコントロール (別 issue) |
| `checksum_type` | issue 0059 後続で型化検討 |

### 2. issue 0057 の pending 解除

`issues/pending/0057-feature-expected-bucket-owner-request-payer.md` を `issues/` に `git mv` で戻す。本 issue 完了後の別作業として処理する。

## AWS S3 API Reference

代表的な箇所を引用する (出力フィールドの API リファレンスは 0065-0068 の各 issue で詳細を引用する)。

- [PutObject](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html)

  > x-amz-server-side-encryption-bucket-key-enabled — Indicates whether the uploaded object uses an S3 Bucket Key for server-side encryption with Key Management Service (KMS) keys (SSE-KMS).

  > x-amz-server-side-encryption-context — Specifies the Amazon Web Services KMS Encryption Context to use for object encryption.

  上記は入力ヘッダーとしては KMS 機能本格対応時に扱うが、レスポンスヘッダーとしては受け取って `*Output` 構造体のフィールドに反映する方針。

## 影響範囲

- `docs/AWS_SDK_RUST.md` の対応表全面再分類のみ。
- 実装ファイルの変更は本 issue では行わない (0065-0068 で実施)。

## 依存関係

- 本 issue は文書修正のみのため、独立して実施可能。
- 0065-0068 の実装系 issue は、issue 0058 (リネーム), 0059 (enum 化), 0060 (時刻処理), 0063 (`CopyObjectResult`) の後に実施する。
- issue 0057 (`expected_bucket_owner` / `request_payer`) の pending 解除は本 issue の方針整理を前提とする。

## 優先度

中 (実装系 issue 0065-0068 の前提となる方針確定のため)

## CHANGES.md への記載

本 issue は `docs/AWS_SDK_RUST.md` の文書修正のみのため、`CHANGES.md` には機能変更として記載しない。`### misc` サブセクションにドキュメント整理として記録するに留める。

- `[UPDATE] docs/AWS_SDK_RUST.md の対応方針表を再分類する`
  - `### misc` サブセクションに記載

## 解決方法

### 実施した変更

1. **`docs/AWS_SDK_RUST.md` の対応方針表を再分類**
   - 凡例を更新 (`対応予定無し` の説明を「S3 固有機能 / S3 互換ストレージで意味が薄いため対応しない」に修正)
   - 旧「対応予定無しのパラメータ」セクションを「対応方針」セクションに置き換え、3 カテゴリに整理:
     - 「未対応 (互換性のため対応予定)」: `expected_bucket_owner` / `request_payer` / `website_redirect_location` / `grant_*`
     - 「入力は対応予定無し、出力は対応」: `bucket_key_enabled` / `ssekms_encryption_context` / `ssekms_key_id` / `sse_customer_algorithm` / `sse_customer_key_md5`
     - 「対応予定無し」維持: `object_lock_*` / `mfa` / `bypass_governance_retention` / `if_match_*` / `mpu_object_size` / `write_offset_bytes` / `optional_object_attributes` / `fetch_owner` / `confirm_remove_self_bucket_access` / `object_ownership` / `checksum_type`

2. **shiguredo_s3 独自パラメータ表を更新**
   - 旧 `checksum_value` を削除 (issue 0062 で個別 `checksum_*` フィールドに置換済み)
   - 「該当なし」とコメント

3. **メソッド名の差異表を更新**
   - `delete` (DeleteObjects) を削除 (issue 0061 で `Delete` 構造体経由に統一済み)

4. **`CHANGES.md` の `### misc` セクションに記載**
   - 機能変更ではなく文書整理のため `[UPDATE]` として misc サブセクションに追記

### issue 0057 の pending 解除について

`issues/pending/0057-feature-expected-bucket-owner-request-payer.md` を `issues/` に戻す作業は別 issue として残す (本 issue では文書修正のみに絞る)。
