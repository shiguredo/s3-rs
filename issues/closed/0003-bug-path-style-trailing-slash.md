# use_path_style 時に bucket-level リクエストのパスに末尾スラッシュが付く

## 優先度

P2

## 概要

`path_for_key` で key が空文字列の場合、path_style だと `/{bucket}/` (末尾スラッシュあり) になる。
仕様的には `/{bucket}` が正しい。

## 影響

- 署名の canonical URI に末尾スラッシュが含まれるため、S3 互換実装で署名不一致やルーティング誤りを起こす可能性がある
- HeadBucket, CreateBucket, DeleteBucket, ListObjectsV2, DeleteObjects が該当する

## 該当箇所

- `src/api/mod.rs:374-381` - `path_for_key` 関数

## 修正方針

key が空文字列の場合は末尾スラッシュを付けないようにする。

## 完了

- `path_for_key` で `encoded_key` が空の場合に path_style では `/{bucket}` (末尾スラッシュなし)、virtual-hosted では `/` を返すよう修正
