# aws-sdk-rust 互換性ドキュメントを現状に同期する

- Priority: Medium
- Created: 2026-07-31
- Completed: {YYYY-MM-DD}
- Branch: feature/update-aws-sdk-rust-compatibility-docs
- Polished: {YYYY-MM-DD}
- Model: GPT-5

## 目的

実装済み項目を未実装として記載している docs/AWS_SDK_RUST.md と、全 operation 対応と誤解できる README の presigned 記載を現状に同期する。

## 現状

docs/AWS_SDK_RUST.md には GetObject の checksum_mode、PutObject の content_length / conditional field、CopyObject の conditional field など、現在は実装済みの項目が未対応として残っている。また output 対応表も過去の issue 対応後の構造と一致していない。

README.md は Presigned リクエスト対応を全オペレーションと記載しているが、実装されている presigned method は一部 operation のみである。Sans I/O による意図的な差分と、未実装の API 差分も区別されていない。

## 設計方針

- aws-sdk-rust の現行ソースと s3-rs の現行ソースを比較して対応表を作り直す。
- 実装済み、未実装、意図的な Sans I/O 差分、既存 pending issue を区別する。
- README の presigned 記載を実装事実に合わせ、docs の未実装一覧を issue と矛盾しない内容にする。
- API Reference URL と aws-sdk-rust の参照先を各項目に残す。

## 完了条件

- docs/AWS_SDK_RUST.md の operation、input、output、presigned 対応表が現状と一致する。
- README.md が実装範囲を過大に表現しない。
- 文書中の未実装項目が今回の issue 一覧または既存 pending issue から追跡できる。

## AWS S3 API Reference

- ListObjects: https://docs.aws.amazon.com/AmazonS3/latest/API/API_ListObjects.html

> For backward compatibility, Amazon S3 continues to support ListObjects.

- GetObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html

> Retrieves an object from Amazon S3.

- HeadObject: https://docs.aws.amazon.com/AmazonS3/latest/API/API_HeadObject.html

> The HEAD operation retrieves metadata from an object without returning the object itself.
