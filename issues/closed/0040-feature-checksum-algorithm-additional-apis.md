# 追加 API への checksum_algorithm パラメータ対応

## 概要

checksum_algorithm パラメータが未対応の API に追加する。

## 対象 API

- DeleteObjects
- CopyObject
- CreateMultipartUpload
- PutBucketVersioning
- PutBucketTagging
- PutBucketPolicy
- PutPublicAccessBlock

## 説明

リクエストボディの整合性検証のために `x-amz-checksum-algorithm` ヘッダーと対応するチェックサムヘッダーを送信する。PutObject と UploadPart では既に対応済み。

## 優先度

中

## 解決方法

全 7 API に `checksum_algorithm` パラメータを追加した。

- ボディがある API (DeleteObjects, PutBucketVersioning, PutBucketTagging, PutBucketPolicy, PutPublicAccessBlock): `x-amz-checksum-algorithm` ヘッダーに加え、ボディからチェックサムを自動計算して対応するチェックサムヘッダーを付与する
- ボディがない API (CopyObject, CreateMultipartUpload): `x-amz-checksum-algorithm` ヘッダーのみ指定する (アルゴリズムのバリデーションは実施)
- 既存の `content-md5` ヘッダーはそのまま維持する (checksum_algorithm は追加の整合性検証手段)
