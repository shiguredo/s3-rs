# RustFS が DeleteBucketEncryption 後の GetBucketEncryption で 400 を返す

Created: 2026-03-27
Model: Opus 4.6

## 概要

RustFS で DeleteBucketEncryption 実行後に GetBucketEncryption を呼ぶと 400 (Bad Request) が返る。S3 の仕様では 404 (ServerSideEncryptionConfigurationNotFoundError) を返すべき。

## 再現手順

1. バケットを作成する
2. PutBucketEncryption で SSE-S3 (AES256) を設定する
3. DeleteBucketEncryption で暗号化設定を削除する
4. GetBucketEncryption を呼ぶ → **400 が返る** (期待値: 404)

## 期待する動作

- AWS S3: 404 `ServerSideEncryptionConfigurationNotFoundError`
- MinIO: 404 `ServerSideEncryptionConfigurationNotFoundError`

## 実際の動作

- RustFS: 400 (Bad Request)

## 影響

shiguredo_s3 側では `400 || 404` で許容するテストにしている (`tests/rustfs.rs` の `test_bucket_encryption`)。

## pending の理由

RustFS 側の不具合であり、shiguredo_s3 側では対応不要。RustFS が修正されたらテストの許容条件を 404 のみに変更する。
