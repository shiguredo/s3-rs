# Object Lock 関連 API の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- 改ざん防止・保持義務のあるデータでは **リテンションとリーガルホールド**が必須になり、バケット既定ロックとオブジェクト単位 API の両方が必要になる。
- aws-sdk-rust と同等の操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

WORM 運用のための Object Lock 関連 API を `S3Client` に追加する。

### 前提条件

- **Object Lock が有効なバケット**であること（バケット作成時に有効化、または `PutObjectLockConfiguration` で設定）
- **バージョニングが有効**であること（Object Lock 有効化時に自動的に有効になる）

`GetObjectLegalHold` / `PutObjectLegalHold` / `GetObjectRetention` / `PutObjectRetention` は `versionId` を受け取り、オブジェクトの特定バージョンを対象にする API である。aws-sdk-rust 互換の API 形状にするため、builder に `version_id` フィールドを含めること。

### 追加の必須パラメータ

- **`PutObjectRetention`**: `bypass_governance_retention` (bool) — GOVERNANCE モードのリテンションを上書きする際に必要。builder に含めること。
  - 参考: [PutObjectRetention - x-amz-bypass-governance-retention](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectRetention.html)
- **`PutObjectLockConfiguration`**: 既存バケットに対して Object Lock を有効化する場合、`x-amz-bucket-object-lock-token` ヘッダーが必要。builder に `token` フィールド（aws-sdk-rust の builder メソッド名と同名）を含めること。HTTP ヘッダー名は `x-amz-bucket-object-lock-token` だが、公開 API 名は aws-sdk-rust 互換で `token` とする。
  - 参考: [PutObjectLockConfiguration - x-amz-bucket-object-lock-token](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectLockConfiguration.html)

## 対象 API

| 操作 | 説明 |
|------|------|
| GetObjectLegalHold | オブジェクトのリーガルホールド状態を取得する |
| PutObjectLegalHold | オブジェクトのリーガルホールドを設定する |
| GetObjectRetention | オブジェクトのリテンション（保持期限）を取得する |
| PutObjectRetention | オブジェクトのリテンションを設定する |
| GetObjectLockConfiguration | バケットの Object Lock 既定設定を取得する |
| PutObjectLockConfiguration | バケットの Object Lock 既定設定を行う |

## AWS 公式ドキュメント（API リファレンス）

- [GetObjectLegalHold](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectLegalHold.html)
- [PutObjectLegalHold](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectLegalHold.html)
- [GetObjectRetention](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectRetention.html)
- [PutObjectRetention](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectRetention.html)
- [GetObjectLockConfiguration](https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectLockConfiguration.html)
- [PutObjectLockConfiguration](https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectLockConfiguration.html)

## ユーザーガイド（概念）

- [S3 Object Lock の仕組み](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lock.html)

## 補足

- バケットの Object Lock 有効化は作成時または `PutObjectLockConfiguration` で行う。
- テストでは「Object Lock 有効かつバージョニング有効」なバケットを前提とし、特定バージョンに対する LegalHold / Retention の設定・取得を検証すること。

## 優先度

高
