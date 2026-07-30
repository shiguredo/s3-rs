# S3 Metadata Configuration API を追加する

- Priority: Low
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-bucket-metadata-configuration
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

S3 Metadata の bucket configuration、metadata table configuration、annotation / inventory / journal table configuration を aws-sdk-rust 互換に扱えるようにする。

## 現状

以下の 9 operation が未実装である。

- CreateBucketMetadataConfiguration
- CreateBucketMetadataTableConfiguration
- DeleteBucketMetadataConfiguration
- DeleteBucketMetadataTableConfiguration
- GetBucketMetadataConfiguration
- GetBucketMetadataTableConfiguration
- UpdateBucketMetadataAnnotationTableConfiguration
- UpdateBucketMetadataInventoryTableConfiguration
- UpdateBucketMetadataJournalTableConfiguration

src/api/ に対応する XML モデルと output 型も存在しない。

## 設計方針

- aws-sdk-rust の型名、nested structure、optional field をそのまま反映する。
- 作成、取得、削除、更新で異なる URI と body を混同しない。
- metadata journal / inventory / annotation の設定をそれぞれ独立した型として扱う。
- 実装対象の S3 互換サーバーで利用可能かを確認し、未対応の場合も Sans I/O のレスポンス検証を追加する。

## 完了条件

- 9 operation の builder と output が追加される。
- 正しい XML、query、header を構築・パースできる。
- AWS SDK の型名・フィールド名との比較テストが通る。

## AWS S3 API Reference

- CreateBucketMetadataConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucketMetadataConfiguration.html

> CreateBucketMetadataConfiguration

- CreateBucketMetadataTableConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateBucketMetadataTableConfiguration.html

> CreateBucketMetadataTableConfiguration

- DeleteBucketMetadataConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketMetadataConfiguration.html

> DeleteBucketMetadataConfiguration

- DeleteBucketMetadataTableConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketMetadataTableConfiguration.html

> DeleteBucketMetadataTableConfiguration

- GetBucketMetadataConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketMetadataConfiguration.html

> GetBucketMetadataConfiguration

- GetBucketMetadataTableConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketMetadataTableConfiguration.html

> GetBucketMetadataTableConfiguration

- UpdateBucketMetadataAnnotationTableConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UpdateBucketMetadataAnnotationTableConfiguration.html

> UpdateBucketMetadataAnnotationTableConfiguration

- UpdateBucketMetadataInventoryTableConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UpdateBucketMetadataInventoryTableConfiguration.html

> UpdateBucketMetadataInventoryTableConfiguration

- UpdateBucketMetadataJournalTableConfiguration: https://docs.aws.amazon.com/AmazonS3/latest/API/API_UpdateBucketMetadataJournalTableConfiguration.html

> UpdateBucketMetadataJournalTableConfiguration
