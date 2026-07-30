# Advanced Object Operation を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-advanced-object-operations
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

aws-sdk-rust に存在する高度な object operation を追加し、S3 API の主要な object 操作を網羅する。

## 現状

以下の operation が src/api/ に存在しない。

- GetObjectAttributes
- GetObjectTorrent
- RestoreObject
- SelectObjectContent
- RenameObject
- UpdateObjectEncryption
- WriteGetObjectResponse

## 設計方針

- 各 operation の builder、入力型、出力型、URI、query、header を aws-sdk-rust と公式 API に合わせる。
- GetObjectAttributes は checksum、object parts、storage class、object size を失わずに返す。
- RestoreObject は restore request と optional tier / description / days を扱う。
- SelectObjectContent は event stream を Sans I/O で段階的にパースできる設計にする。
- RenameObject と UpdateObjectEncryption は S3 Express の endpoint / session 前提を明確にする。
- WriteGetObjectResponse は Object Lambda 用の response header と body を扱う。

## 完了条件

- 7 operation が client から利用できる。
- 通常レスポンスだけでなく、SelectObjectContent の event stream とエラー終端を処理できる。
- 各 operation の実レスポンスを使った Sans I/O テストが通る。

## AWS S3 API Reference

- GetObjectAttributes: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectAttributes.html

> GetObjectAttributes

- GetObjectTorrent: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectTorrent.html

> GetObjectTorrent

- RestoreObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_RestoreObject.html

> RestoreObject

- SelectObjectContent: https://docs.aws.amazon.com/AmazonS3/latest/API/API_SelectObjectContent.html

> SelectObjectContent

- RenameObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_RenameObject.html

> RenameObject

- UpdateObjectEncryption: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UpdateObjectEncryption.html

> UpdateObjectEncryption

- WriteGetObjectResponse: https://docs.aws.amazon.com/AmazonS3/latest/API/API_WriteGetObjectResponse.html

> WriteGetObjectResponse
