# PutObject のデフォルト CRC32 自動計算の明文化

## 概要

`src/api/put_object.rs` で `checksum_algorithm` 未指定時にデフォルトで CRC32 を自動計算している。

## 調査結果

aws-sdk-rust も `RequestChecksumCalculation::WhenSupported` のデフォルト設定で、
チェックサムアルゴリズム未指定時に CRC32 を自動設定している。
つまり shiguredo_s3 の挙動は aws-sdk-rust と一致している。

## 解決方法

- `checksum_algorithm()` メソッドの doc コメントに「未指定の場合はデフォルトで CRC32 が使用される」と明記した
- `build_request()` 内のコメントに aws-sdk-rust と同じデフォルト動作である旨を追記した
