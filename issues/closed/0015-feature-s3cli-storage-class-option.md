# s3cli に --storage-class オプションを追加する

## 概要

cp / mv / sync サブコマンドに `--storage-class` オプションを追加する。

## 対応するストレージクラス

- `STANDARD`
- `STANDARD_IA`
- `ONEZONE_IA`
- `INTELLIGENT_TIERING`
- `GLACIER`
- `DEEP_ARCHIVE`
- `GLACIER_IR`
- `REDUCED_REDUNDANCY`

## 備考

- ライブラリ側の `PutObject` / `CopyObject` / `CreateMultipartUpload` に `x-amz-storage-class` ヘッダーの追加が必要

## 解決方法

ライブラリ側の PutObject / CopyObject / CreateMultipartUpload に storage_class フィールド・セッター・x-amz-storage-class ヘッダー生成を追加。s3cli の cp コマンドに --storage-class オプションを追加。
