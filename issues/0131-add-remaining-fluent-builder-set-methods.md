# Fluent Builder の set_* 残件を追加する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/add-remaining-fluent-builder-set-methods
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

aws-sdk-rust の Fluent Builder が提供する Option 直接指定用の set_* method を全 builder で揃え、builder の移行互換性を完成させる。

## 現状

過去の Fluent Builder 対応後も、共通 56 operation の比較で 265 項目中 167 項目に set_* が不足している。特に単純な bucket operation、multipart operation、設定 operation に不足が多い。

## 設計方針

- src/api/ 配下の全 Fluent Builder を aws-sdk-rust の input fields と機械的に照合する。
- Option<T> の全フィールドに set_field(Option<T>) を追加する。
- 既存の field(value) method と既存 set_* method は維持する。
- ConfigBuilder の set_* と、非 Fluent Builder の対象範囲は既存 issue の境界を確認して混在させない。

## 完了条件

- 全 operation の入力フィールドに対応する set_* が存在する。
- set_field(Some(value)) と field(value) の request が一致する。
- set_field(None) が値をクリアすることを検証するテストが通る。
- aws-sdk-rust の generated builder との API surface 比較が通る。

## AWS S3 API Reference

- PutObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html

> The PutObject action adds an object to a bucket.

- GetObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html

> Retrieves an object from Amazon S3.

- ListObjectsV2: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjectsV2.html

> Returns some or all (up to 1,000) of the objects in a bucket.
