# ListObjectVersions の実装

Created: 2026-04-04
Model: Composer 2 Fast

## 根拠

- バージョニング運用では **全バージョンと削除マーカーの列挙**が必要で、`ListObjectsV2` では代替できない。
- aws-sdk-rust と同等の一覧操作を揃えるため（`AGENTS.md` の shiguredo_s3 方針）。

## 概要

バージョニング有効バケット内のオブジェクトバージョンおよび削除マーカーを一覧する API を `S3Client` に追加する。`ListObjectsV2` では代替できない。

## 対象 API

| 操作 | 説明 |
|------|------|
| ListObjectVersions | プレフィックス・区切り文字・キーマーカー・バージョン ID マーカーでバージョン一覧を取得する |

## AWS 公式ドキュメント（API リファレンス）

- [ListObjectVersions](https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectVersions.html)

## ページングパラメータ

aws-sdk-rust 互換の builder / output 設計のため、以下のパラメータを明示的にサポートすること。

### リクエスト（builder フィールド）

| パラメータ | 説明 |
|-----------|------|
| `prefix` | 指定プレフィックスで絞り込む |
| `delimiter` | 共通プレフィックスでグループ化する |
| `key_marker` | このキー以降のバージョンを返す |
| `version_id_marker` | `key_marker` と組み合わせて、この version ID 以降のバージョンを返す |
| `max_keys` | 返却する最大件数（デフォルト 1000） |
| `encoding_type` | キーのエンコーディング（`url`） |
| `optional_object_attributes` | 追加で返却するオブジェクト属性（`RestoreStatus` 等） |

### レスポンス（output フィールド）

| フィールド | 説明 |
|-----------|------|
| `is_truncated` | 結果が切り詰められたかどうか |
| `next_key_marker` | 次ページの `key_marker` に渡す値 |
| `next_version_id_marker` | 次ページの `version_id_marker` に渡す値 |
| `versions` | オブジェクトバージョンのリスト |
| `delete_markers` | 削除マーカーのリスト |
| `common_prefixes` | 共通プレフィックスのリスト |
| `name` | バケット名 |
| `prefix` | リクエストで指定されたプレフィックス |
| `delimiter` | リクエストで指定された区切り文字 |
| `max_keys` | リクエストで指定された最大件数 |
| `key_marker` | リクエストで指定されたキーマーカー |
| `version_id_marker` | リクエストで指定されたバージョン ID マーカー |
| `encoding_type` | キーのエンコーディング |

## 補足

- バケットのバージョニング状態は `GetBucketVersioning` / `PutBucketVersioning` で設定する（既存実装）。

## 優先度

高
