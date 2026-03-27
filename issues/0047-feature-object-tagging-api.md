# オブジェクトタグ API (PutObjectTagging / GetObjectTagging / DeleteObjectTagging) を追加する

Created: 2026-03-27
Model: Opus 4.6

## 概要

オブジェクト単位のタグ操作 API を追加する。バケットレベルのタグ操作 (`PutBucketTagging` / `GetBucketTagging` / `DeleteBucketTagging`) は実装済みだが、オブジェクトレベルのタグ操作は未実装である。

## 根拠

S3 のオブジェクトタグはアクセス制御、ライフサイクルポリシー、コスト配分などに広く使われる基本機能である。バケットタグのみでオブジェクトタグが操作できないのは、利用者にとって不自然な欠落となる。

## 対象 API

### PutObjectTagging

- **HTTP**: `PUT /{Key}?tagging`
- **説明**: オブジェクトにタグセットを設定する。既存のタグは全て上書きされる
- **リクエストボディ**: XML 形式のタグ情報
- **レスポンス**: 200 OK、`version_id` を返す
- **参照**: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectTagging.html

FluentBuilder パラメータ (aws-sdk-rust 互換):

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| bucket | String | 必須 | バケット名 |
| key | String | 必須 | オブジェクトキー |
| version_id | String | - | オブジェクトのバージョン ID |
| checksum_algorithm | String | - | チェックサムアルゴリズム |
| tags | Vec\<Tag\> | 必須 | タグセット |

Output:

| フィールド | 型 | 説明 |
|---|---|---|
| version_id | Option\<String\> | オブジェクトバージョン ID |

### GetObjectTagging

- **HTTP**: `GET /{Key}?tagging`
- **説明**: オブジェクトに設定されているタグの一覧を取得する
- **レスポンス**: 200 OK、XML 形式のタグ情報
- **参照**: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectTagging.html

FluentBuilder パラメータ (aws-sdk-rust 互換):

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| bucket | String | 必須 | バケット名 |
| key | String | 必須 | オブジェクトキー |
| version_id | String | - | オブジェクトのバージョン ID |

Output:

| フィールド | 型 | 説明 |
|---|---|---|
| version_id | Option\<String\> | オブジェクトバージョン ID |
| tag_set | Vec\<Tag\> | タグセット |

### DeleteObjectTagging

- **HTTP**: `DELETE /{Key}?tagging`
- **説明**: オブジェクトのタグを全て削除する
- **レスポンス**: 204 No Content、`version_id` を返す
- **参照**: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjectTagging.html

FluentBuilder パラメータ (aws-sdk-rust 互換):

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| bucket | String | 必須 | バケット名 |
| key | String | 必須 | オブジェクトキー |
| version_id | String | - | オブジェクトのバージョン ID |

Output:

| フィールド | 型 | 説明 |
|---|---|---|
| version_id | Option\<String\> | オブジェクトバージョン ID |

## 実装方針

- バケットタグ API (`put_bucket_tagging.rs` / `get_bucket_tagging.rs` / `delete_bucket_tagging.rs`) の実装パターンを踏襲する
- バケットタグとの違いは、`key` パラメータが必須であること、`version_id` パラメータが追加されること
- `expected_bucket_owner` や `request_payer` は現時点では対応しない (他の API でも未対応のため)
- `Tag` 型と XML 構築・パース処理はバケットタグ API と共有する
- Output 型 (`PutObjectTaggingOutput` / `GetObjectTaggingOutput` / `DeleteObjectTaggingOutput`) を `types.rs` に追加する

## 優先度

高
