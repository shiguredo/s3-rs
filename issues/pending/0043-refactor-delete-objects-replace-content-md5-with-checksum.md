# DeleteObjects の Content-MD5 とチェックサムアルゴリズムの関係整理

## 概要

`src/api/delete_objects.rs` で Content-MD5 ヘッダーを常に付与している。
aws-sdk-rust では Content-MD5 を使わず CRC32 チェックサムのみを使用しているが、
S3 公式ドキュメントの記述とは異なる。

## S3 公式ドキュメントの記述

- **一般バケット**: Content-MD5 は**必須**
  > The Content-MD5 request header is required for all Multi-Object Delete requests.
  > Amazon S3 uses the header value to ensure that your request body has not been altered in transit.
- **ディレクトリバケット**: Content-MD5 **または** チェックサムヘッダーのいずれかが必須

## aws-sdk-rust の挙動

- Content-MD5 は付与していない
- デフォルトで `x-amz-sdk-checksum-algorithm: CRC32` と `x-amz-checksum-crc32` を付与
- ドキュメント上は一般バケットで Content-MD5 必須だが、実際にはチェックサムヘッダーがあれば受け付けている模様

## 現状の shiguredo_s3 の実装

- Content-MD5 を常に付与 (ドキュメント通りの正しい実装)
- `checksum_algorithm` を指定した場合はチェックサムヘッダーも追加で付与

## pending にした理由

- 現在の実装は S3 公式ドキュメントに準拠しており正しい
- aws-sdk-rust はドキュメントと異なる挙動をしており、どちらに合わせるか設計判断が必要
- S3 側の仕様が将来変更される可能性もあるため、経過観察が必要
