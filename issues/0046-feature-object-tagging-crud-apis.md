# GetObjectTagging / PutObjectTagging / DeleteObjectTagging の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- オブジェクトタグはライフサイクルルールやコスト配分・運用ラベルで広く使われ、**アップロード時の `x-amz-tagging` だけでは既存オブジェクトの取得・変更・削除に対応できない**。
- aws-sdk-rust と同等の操作セットを揃え、利用者が移行しやすくするため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

既存オブジェクトに対するタグの取得・設定・削除を行う API を `S3Client` に追加する。バケットタグ（`GetBucketTagging` 等）とは別操作である。

3 API とも `versionId` クエリパラメータを受け取り、バージョニング有効バケットでは特定バージョンを対象にできる。aws-sdk-rust 互換の API 形状にするため、builder に `version_id` フィールドを含めること。

## 対象 API

| 操作 | 説明 |
|------|------|
| GetObjectTagging | オブジェクトに付いたタグ（キー・値）を取得する |
| PutObjectTagging | オブジェクトのタグを置き換える |
| DeleteObjectTagging | オブジェクトからタグをすべて削除する |

## AWS 公式ドキュメント（API リファレンス）

- [GetObjectTagging](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectTagging.html)
- [PutObjectTagging](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectTagging.html)
- [DeleteObjectTagging](https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjectTagging.html)

## 関連

- アップロード時にタグを付ける `x-amz-tagging` ヘッダーは [#0039](0039-feature-tagging-params.md) で扱う。本 issue は **オブジェクト単位のタグ CRUD** である。

## 優先度

高
