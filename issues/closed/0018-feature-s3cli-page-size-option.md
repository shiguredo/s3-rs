# s3cli に --page-size オプションを追加する

## 概要

ls / rm サブコマンドに `--page-size` オプションを追加する。

## 動作

- ListObjectsV2 の `max-keys` パラメータに対応する
- デフォルトは 1000（S3 の既定値）
- ページネーション時の 1 ページあたりの結果数を制御する

## 備考

- ライブラリ側の `ListObjectsV2` に `max_keys` パラメータは既に存在する

## 解決方法

s3cli の `cmd_ls` に `--page-size` オプションを追加し、`ListObjectsV2` の `max_keys` パラメータに渡すようにした。
