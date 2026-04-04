# UploadPartCopy の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- 大容量オブジェクトをクライアントで再アップロードせず、**サーバー側で範囲コピーしてマルチパートを構成**できる。実運用の大ファイル効率化に直結する。
- aws-sdk-rust と同等のマルチパート操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

マルチパートアップロードの 1 パートとして、別オブジェクトのバイト範囲をコピーする API を `S3Client` に追加する。

## 対象 API

| 操作 | 説明 |
|------|------|
| UploadPartCopy | コピー元オブジェクトの範囲をパートデータとしてアップロードする |

## AWS 公式ドキュメント（API リファレンス）

- [UploadPartCopy](https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPartCopy.html)

## 補足

- コピー元は `x-amz-copy-source` ヘッダーで指定する。バージョニング有効バケットでは `?versionId=` を付与して特定バージョンを対象にできる。
- 範囲指定ヘッダー（`x-amz-copy-source-range`）でバイト範囲を限定できる。
- aws-sdk-rust 互換の builder にするため、以下のヘッダーも入力項目として含めること:
  - Conditional headers:
    - `copy_source_if_match`
    - `copy_source_if_none_match`
    - `copy_source_if_modified_since`
    - `copy_source_if_unmodified_since`
  - コピー元 SSE-C headers:
    - `copy_source_sse_customer_algorithm`
    - `copy_source_sse_customer_key`
    - `copy_source_sse_customer_key_md5`
  - コピー先 (destination) SSE-C headers:
    - `sse_customer_algorithm`
    - `sse_customer_key`
    - `sse_customer_key_md5`
  - その他:
    - `expected_bucket_owner`（コピー先バケットのオーナー検証）
    - `expected_source_bucket_owner`（コピー元バケットのオーナー検証）
    - `request_payer`
- レスポンスにコピー結果の ETag 等が含まれる。

## 優先度

高
