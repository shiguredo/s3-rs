# DeleteObjects の LastModifiedTime が IMF-fixdate ではなく ISO 8601 で送られる

- Priority: High
- Created: 2026-07-09
- Completed: 2026-07-23
- Model: Grok 4.5
- Polished: 2026-07-23
- Branch: feature/fix-delete-objects-last-modified-time-format

## AWS S3 API Reference

- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjects.html>
- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_ObjectIdentifier.html>

Conditional deletes のサンプルより:

> ```
> <LastModifiedTime>Tue, 15 Oct 2024 15:04:05 GMT</LastModifiedTime>
> ```

## 目的

条件付き削除で使う `ObjectIdentifier.last_modified_time` を、S3 / aws-sdk-rust が期待する IMF-fixdate（HTTP-date）形式で送信する。

## 優先度根拠

- 条件付き削除の前提条件がサーバ側で一致せず、削除が意図どおり成功しない・失敗しない可能性がある
- aws-sdk-s3 は `Format::HttpDate` でシリアライズしており、互換方針に反する
- 本クレートには既に `format_imf_fixdate` がある

## 現状

`src/api/delete_objects.rs` の `build_delete_objects_xml` で:

```152:165:src/api/delete_objects.rs
        if let Some(t) = obj.last_modified_time {
            // S3 仕様では ISO 8601 (RFC 3339) フォーマットで送る
            // ここではミリ秒を含めない秒精度
            let secs = t
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let c = crate::datetime::civil_from_unix_timestamp(secs)
                .expect("SystemTime from S3 response should be convertible to CivilDateTime");
            let formatted = format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                c.year, c.month, c.day, c.hour, c.minute, c.second
            );
            w.element("LastModifiedTime", &formatted)?;
```

問題点:

1. 形式が `YYYY-MM-DDTHH:MM:SSZ` であり、公式例・aws-sdk の HTTP-date と異なる
2. epoch 前の `SystemTime` は `unwrap_or(0)` で 1970-01-01 に化け、条件付き削除の意味が壊れる
3. `expect` のコメントは「S3 response」とあるが、実際は利用者入力の `SystemTime`

aws-sdk-s3 1.135 の `shape_object_identifier.rs`:

```rust
inner_writer.data(var_3.fmt(::aws_smithy_types::date_time::Format::HttpDate)?.as_ref());
```

## 設計方針

- `build_delete_objects_xml` の `LastModifiedTime` 生成部分を、手動の ISO 8601 フォーマットから `crate::datetime::format_imf_fixdate(t)?` に置き換える
- `format_imf_fixdate` は epoch 前の `SystemTime` に対して `Error::InvalidInput` を返すため、`unwrap_or(0)` で黙殺されていた問題も同時に解消される
- `duration_since` / `civil_from_unix_timestamp` / `expect` のブロック全体を削除し、`format_imf_fixdate` 一発で置き換える
- 本修正により issue 0102（`expect` パニック・`unwrap_or(0)` 黙殺）も同時に解消される。0102 と同一コードブロックを対象とするため、本 issue で一括修正する

## 完了条件

- `build_delete_objects_xml` の `LastModifiedTime` が `format_imf_fixdate` 経由で `Day, DD Mon YYYY HH:MM:SS GMT` 形式で送信されること
- epoch 前の `SystemTime` が `Error::InvalidInput` で拒否されること（`unwrap_or(0)` の黙殺が解消されること）
- `tests/test_delete_objects.rs` に `build_request` で `last_modified_time` を指定した場合のフォーマット検証テストと、epoch 前に `Error::InvalidInput` になることを確認するエラーパステストを追加すること
- `CHANGES.md` の `## develop` に `[FIX]` エントリを記載すること

## 解決方法

1. `build_delete_objects_xml` の `last_modified_time` 処理ブロック（`duration_since` / `unwrap_or(0)` / `civil_from_unix_timestamp` / `expect` / `format!`）を `crate::datetime::format_imf_fixdate(t)?` 一発に置き換える
2. `tests/test_delete_objects.rs` に `last_modified_time` 指定時のフォーマット検証テストと epoch 前エラーパステストを追加する（issue 0102 のテストも兼ねる）
3. `CHANGES.md` の `## develop` に `[FIX]` エントリを追加する（0102 と統合して 1 エントリで可）
