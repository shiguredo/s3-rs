# Bucket / Object ACL API を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-acl-apis
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

GetBucketAcl、PutBucketAcl、GetObjectAcl、PutObjectAcl を追加し、ACL を利用する S3 互換ストレージとの API 互換性を高める。

## 現状

CreateBucket や PutObject の canned ACL は存在するが、bucket / object の ACL を取得・設定する operation と AccessControlPolicy、Grant、Grantee のモデルが存在しない。

## 設計方針

- aws-sdk-rust の AccessControlPolicy、Grant、Grantee、Owner、Permission、BucketCannedAcl に合わせる。
- bucket ACL と object ACL で異なる header、version_id、grant 項目を仕様どおりに扱う。
- XML の xsi:type、URI、EmailAddress、ID、Permission を失わずにパースする。
- ACL が無効化された bucket での AWS の挙動を統合テストで確認する。

## 完了条件

- 4 operation の builder、XML 入出力、output 型が実装される。
- canned ACL と grant 指定の両方を扱える。
- 実際の S3 互換サーバーを使う ACL の統合テストが通る。

## AWS S3 API Reference

- GetBucketAcl: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketAcl.html

> This implementation of the GET action uses the acl subresource to return the access control list (ACL) of a bucket.

- PutBucketAcl: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketAcl.html

> This implementation of the PUT action uses the acl subresource to set the access control list (ACL) permissions for a bucket.

- GetObjectAcl: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObjectAcl.html

> Returns the access control list (ACL) of an object.

- PutObjectAcl: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObjectAcl.html

> Uses the acl subresource to set the ACL permissions for an object.
