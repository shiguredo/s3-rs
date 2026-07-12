# DeleteObjects が x-amz-checksum-algorithm を送り x-amz-sdk-checksum-algorithm を使わない

- Priority: High
- Created: 2026-07-09
- Model: Grok 4.5
- Polished: 2026-07-12
- Branch: feature/fix-delete-objects-sdk-checksum-algorithm-header

## AWS S3 API Reference

- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjects.html>

Request Syntax より:

> ```
> POST /?delete HTTP/1.1
> Host: Bucket.s3.amazonaws.com
> ...
> x-amz-sdk-checksum-algorithm: ChecksumAlgorithm
> ```

URI Request Parameters より:

> **x-amz-sdk-checksum-algorithm** — Indicates the algorithm used to create the checksum for the object when you use the SDK.

## 目的

DeleteObjects でボディ整合性検証用のアルゴリズムヘッダーを、S3 / aws-sdk-rust が期待する `x-amz-sdk-checksum-algorithm` で送信する。

## 優先度根拠

- ヘッダー名が誤っていると SDK チェックサム検証パスがサーバ側で期待どおり動かない
- 同クレートの `PutObject` / `UploadPart` は既に `x-amz-sdk-checksum-algorithm` を使っており、DeleteObjects だけ不整合
- aws-sdk-s3 も DeleteObjects で `x-amz-sdk-checksum-algorithm` を送る

## 現状

```90:96:src/api/delete_objects.rs
        if let Some(ref algorithm) = self.checksum_algorithm {
            extra_headers.push(("x-amz-checksum-algorithm", algorithm.as_str()));
            let header_name = crate::checksum::header_name(algorithm)?;
            computed_checksum = crate::checksum::compute_checksum(algorithm, xml_body.as_bytes())?;
            extra_headers.push((header_name, &computed_checksum));
```

正しい例（PutObject）:

```563:563:src/api/put_object.rs
            extra_headers.push(("x-amz-sdk-checksum-algorithm", algorithm.as_str()));
```

**注意**: `CreateMultipartUpload` / `CopyObject` の `x-amz-checksum-algorithm` はオブジェクトに付与するチェックサムアルゴリズム指定であり、ボディ SDK チェックサム用の `x-amz-sdk-checksum-algorithm` とは別用途。本 issue は DeleteObjects（リクエストボディの整合性検証）のみを対象とする。他の Put 系設定 API で同様の誤りがある場合は別 issue で扱う。

## 設計方針

- `delete_objects.rs` のアルゴリズムヘッダー名を `x-amz-sdk-checksum-algorithm` に変更する
- 個別チェックサム値ヘッダー（`x-amz-checksum-crc32` 等）の付与ロジックは現状維持

## 完了条件

- `src/api/delete_objects.rs:92` のヘッダー名を `x-amz-checksum-algorithm` から `x-amz-sdk-checksum-algorithm` に変更すること
- `tests/test_delete_objects.rs` に `checksum_algorithm` 指定時のリクエストヘッダー検証テストを追加すること（`x-amz-sdk-checksum-algorithm` が含まれ、`x-amz-checksum-algorithm` が含まれないことを確認）
- `CHANGES.md` の `## develop` に `[FIX]` エントリを記載すること
