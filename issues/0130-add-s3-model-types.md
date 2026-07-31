# S3 model の enum と nested structure を aws-sdk-rust 互換にする

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-s3-model-types
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

aws-sdk-rust が typed enum または nested structure で表現している S3 model を String や flat field のままにせず、API 名・型名・フィールド構造を互換にする。

## 現状

src/types.rs では以下の model が不足、または String / flat structure になっている。

- ObjectLockMode、ObjectLockLegalHoldStatus、ObjectLockRetentionMode
- ReplicationStatus、ArchiveStatus、TransitionDefaultMinimumObjectSize
- BucketCannedAcl、BucketVersioningStatus、MfaDelete
- ObjectStorageClass、Initiator、RequestCharged
- GetBucketOwnershipControlsOutput の OwnershipControls nested structure
- GetPublicAccessBlockOutput / PutPublicAccessBlock の PublicAccessBlockConfiguration
- UploadPartCopyOutput の CopyPartResult nested structure

request_payer は既存の共通対応 issue、checksum type は Flexible Checksum issue と重複させない。

## 設計方針

- aws-sdk-rust の enum 名、variant、unknown value の扱いを確認して実装する。
- String から enum へ変更するフィールドは builder、output、XML parser の全経路を同時に更新する。
- flat な公開構造体は SDK の nested structure に合わせる。ただし既存 API からの移行方法を issue 内で明記する。
- 無効な値は InvalidInput または InvalidResponse として明示的に扱う。

## 完了条件

- 対象 model が aws-sdk-rust と同じ型名、variant、nested field を持つ。
- XML の往復、未知値、欠落値を検証するテストが通る。
- 外部公開型の変更内容が CHANGES.md と docs に記載される。

## AWS S3 API Reference

- GetBucketVersioning: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketVersioning.html

> Status: The versioning state of the bucket.

- GetBucketOwnershipControls: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketOwnershipControls.html

> Returns the OwnershipControls for an Amazon S3 bucket.

- GetPublicAccessBlock: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetPublicAccessBlock.html

> Retrieves the public access block configuration for an Amazon S3 bucket.

- UploadPartCopy: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UploadPartCopy.html

> Uploads a part by copying data from an existing object as the data source.
