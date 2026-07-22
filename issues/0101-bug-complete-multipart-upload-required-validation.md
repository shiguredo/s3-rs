# CompleteMultipartUpload で multipart_upload の必須チェックを追加する

- Priority: High
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Polished: 2026-07-23
- Branch: feature/fix-complete-multipart-upload-required-validation

## 目的

`complete_multipart_upload.rs` の `build_request()` および `presigned()` で、`multipart_upload` が `None` の場合に必須パラメータのバリデーションがスキップされ、空の `<CompleteMultipartUpload>` XML が生成される問題を修正する。

## 優先度根拠

利用者が誤って `multipart_upload` を設定せずにリクエストを構築した場合、空 XML が S3 に送信され、S3 側のエラー待ちになる。aws-sdk-rust ではビルド時点でエラーになるため互換性の問題でもある。

## 現状

`src/api/complete_multipart_upload.rs:98-118`:

```rust
if let Some(ref upload) = self.multipart_upload
    && let Some(ref parts) = upload.parts
{
    // パート番号重複チェック、昇順チェック
    // e_tag 必須チェック
}
```

`multipart_upload` が `None` の場合、上記ブロック全体がスキップされる。`build_complete_multipart_xml` は `multipart_upload` が `None` の場合に空の `<CompleteMultipartUpload>` XML を生成する。

## 設計方針

- `build_request` と `presigned` の先頭（`required` 呼び出し直後、parts 検証ブロックの前）に `multipart_upload` の必須チェックを追加する
- `multipart_upload` が `None` の場合: `Error::InvalidInput("multipart_upload is required")` を返す
- `parts` が `None` または空 `Vec` の場合: `Error::InvalidInput("at least one part is required")` を返す
- `build_complete_multipart_xml` 側でも空 parts チェックを追加し、XML 生成前に早期エラーを返す
- issue 0099（presigned の parts 検証）と同一ファイルを対象とする。本 issue を先に実装すれば 0099 の `validate_completed_parts` は `None` を扱う必要がなくなるため、本 issue → 0099 の順で実装することを推奨する

## 完了条件

- `build_request` と `presigned` の両方で `multipart_upload` が `None` の場合に `Error::InvalidInput` が返ること
- `parts` が `None` または空 `Vec` の場合に `Error::InvalidInput` が返ること
- `tests/test_complete_multipart_upload.rs` に上記 2 ケースのエラーパステストを追加すること
- 既存のテストが全て通過すること
- `CHANGES.md` の `## develop` に `[FIX]` エントリを記載すること

## 解決方法

1. `build_request()` と `presigned()` の先頭付近に `multipart_upload` の必須チェックを追加する
2. `build_complete_multipart_xml` にも空 parts のチェックを追加する
3. テストを追加する（`tests/test_complete_multipart_upload.rs` を作成、または既存の統合テストにケースを追加）
4. CHANGES.md の `## develop` に `[FIX]` エントリを追加する
