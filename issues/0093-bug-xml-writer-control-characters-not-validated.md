# XmlWriter が XML 1.0 で禁止された制御文字を検証せず素通しする

- Priority: Medium
- Created: 2026-07-07
- Model: hy3-free
- Branch: feature/fix-xml-writer-control-characters

## AWS S3 API Reference

- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteObjects.html>

> The `DELETE` operation removes multiple objects from a bucket.

`Delete` リクエストボディの `<Key>` にはオブジェクトキー（制御文字を含み得る）が入る。S3 は正しい XML のみを受け付け、不正な場合は `MalformedXML` を返す。

参考（XML 1.0 整形式）: <https://www.w3.org/TR/xml/#charsets> — 文字データには `0x00`〜`0x1F` の制御文字（改行・タブ・キャリッジリターンを除く）を含めてはならない。

## 目的

`XmlWriter` がユーザー由来の文字列をエスケープ/検証せずに出力し、生成 XML が XML 1.0 非準拠になって S3 から `MalformedXML` を招く問題を防ぐ。

## 優先度根拠

- xml-rs 1.3 の `characters` は制御文字をエスケープも拒否もせず生バイトで出力する（実挙動確認済み）。
- クライアント側で明確なエラーにならず、原因追跡が困難になる。

## 現状

- `src/xml.rs:269-285`（`XmlWriter::text` / `element`）は引数をそのまま `xml::writer::XmlEvent::characters` に渡す。
- 呼び出し側: `src/api/delete_objects.rs:145`（`w.element("Key", &obj.key)`）、`src/api/put_object_tagging.rs:135-136`（タグ key/value）等がユーザー入力を書く。

## 設計方針

- `text()` で XML 1.0 の許可範囲（`#x9 | #xA | #xD | #x20-#xD7FF | #xE000-#xFFFD | #x10000-#x10FFFF`）外の制御文字を検出したら `Error::InvalidInput` を返す（`XmlWriter` のメソッドを `Result` 化するか、書き込み前に検証）。
- `mod.rs` 等の呼び出し側で `?` で伝播するよう修正。

## 完了条件

- `XmlWriter` が制御文字を含む文字列で `Error::InvalidInput` を返すこと。
- `tests/` に制御文字を含むキー/タグで `Error::InvalidInput` になることを確認するテストを追加すること。
- `CHANGES.md` の `## develop` セクションに `[FIX]` エントリを記載すること。

## 解決方法

（未着手）
