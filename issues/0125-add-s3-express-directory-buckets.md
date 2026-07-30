# S3 Express Directory Bucket API を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-s3-express-directory-buckets
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

CreateSession と ListDirectoryBuckets を追加し、S3 Express One Zone の directory bucket を aws-sdk-rust 互換に扱えるようにする。

## 現状

src/api/ に CreateSession、ListDirectoryBuckets、session credential の output、directory bucket の一覧 output が存在しない。

## 設計方針

- CreateSession の session token、credentials、expiration を aws-sdk-rust と同じ型で返す。
- directory bucket の zone、region、bucket ARN、creation date を一覧 output に反映する。
- endpoint の virtual-hosted-style、zone id、session token header を既存の endpoint / signing 実装と整合させる。
- 長期 credential を session credential に置き換える境界を明確にする。

## 完了条件

- 2 operation の request / response が実装される。
- directory bucket 用 host と session token を署名に正しく反映できる。
- 実際の S3 Express 対応環境、または公式形式の実レスポンスを使うテストが通る。

## AWS S3 API Reference

- CreateSession: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateSession.html

> Creates a session that establishes temporary security credentials to access a directory bucket.

- ListDirectoryBuckets: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListDirectoryBuckets.html

> Returns a list of all directory buckets owned by the authenticated sender of the request.
