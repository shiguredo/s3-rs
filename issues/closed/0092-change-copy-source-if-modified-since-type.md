# CopyObject の copy_source_if_*_since を SystemTime 型に統一する

- Priority: Low
- Created: 2026-07-07
- Completed: 2026-07-29
- Model: hy3-free
- Branch: feature/change-copy-source-if-since-type

## AWS S3 API Reference

- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CopyObject.html>

> `x-amz-copy-source-if-modified-since` / `x-amz-copy-source-if-unmodified-since` — 条件付きコピーのための日付ヘッダ。

同ページのリクエスト構文（行 83, 85）で `{{CopySourceIfModifiedSince}}` / `{{CopySourceIfUnmodifiedSince}}` として参照される。aws-sdk-rust では `CopySourceIfModifiedSince` / `CopySourceIfUnmodifiedSince` は `DateTime` 型であり、SDK が自動的に IMF-fixdate へ整形する。

## 目的

`copy_object` および `upload_part_copy` の `copy_source_if_modified_since` / `copy_source_if_unmodified_since` を `String` から `SystemTime` に変更し、日付フォーマットを利用者に丸投げしないようにする。

## 優先度根拠

- 同一クレート内の `get_object` は `if_modified_since: SystemTime` を `format_imf_fixdate` で整形しているのに、copy 系だけ `String` で非対称。
- `String` のままだと利用者が ISO 8601 等の誤った書式を渡してもコンパイルが通り、S3 側で暗黙に 400 になる。型で強制すればコンパイル時に防げる。

## 現状

- `src/api/copy_object.rs:273-282`、`src/api/upload_part_copy.rs:99-109` は `Option<String>`。
- `get_object.rs:280-288` は `SystemTime` + `crate::datetime::format_imf_fixdate`。

## 設計方針

- 両フィールドを `SystemTime`（または `DateTime`）型に変更し、`format_imf_fixdate` で IMF-fixdate に整形して `x-amz-copy-source-if-*-since` ヘッダーに設定する。
- 変更は後方互換のない型変更のため `[CHANGE]` として扱う。

## 完了条件

- `copy_object` / `upload_part_copy` の `copy_source_if_*_since` が `SystemTime` 型になり、IMF-fixdate に整形されること。
- `CHANGES.md` の `## develop` セクションに `[CHANGE]` エントリを記載すること。

## 解決方法

コミット `d118f50` で実装済み。`copy_source_if_modified_since` / `copy_source_if_unmodified_since` を `Option<String>` から `Option<SystemTime>` に変更し、`format_imf_fixdate` で IMF-fixdate に整形。`copy_object.rs` / `upload_part_copy.rs` の両方で対応。CHANGES.md に `[CHANGE]` エントリ記載済み。
