# DeleteObjects で利用者入力の SystemTime によるパニック経路を修正する

- Priority: High
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/fix-delete-objects-panic-via-systemtime

## 目的

`delete_objects.rs` の `build_delete_objects_xml` 関数内で、利用者が `ObjectIdentifier.last_modified_time` に巨大な `SystemTime` を設定した場合に `.expect()` でパニックする経路を修正する。

## 優先度根拠

Sans I/O ライブラリとして、利用者入力でパニックが発生することは許容されない。エラーは `Result` 型で伝播されるべき。

## 現状

`src/api/delete_objects.rs:155-160`:

```rust
let secs = t
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or(0);
let c = crate::datetime::civil_from_unix_timestamp(secs)
    .expect("SystemTime from S3 response should be convertible to CivilDateTime");
```

2 つの問題がある:
1. `.duration_since(UNIX_EPOCH)` が `Err` の場合（UNIX_EPOCH 以前）、`.unwrap_or(0)` で暗黙的に epoch として扱われる
2. `.expect()` が利用者入力由来の値に対してパニックする。またメッセージが「S3 response」とあるが実際は利用者入力

## 設計方針

1. `.unwrap_or(0)` を `Result` 型のエラーハンドリングに変更する
2. `.expect()` を `?` によるエラー伝播に変更する
3. エラーメッセージを適切なものに修正する

## 完了条件

- `ObjectIdentifier.last_modified_time` に UNIX_EPOCH 以前または極端に遠い未来の `SystemTime` を設定した場合にパニックせず `Error::InvalidInput` が返ること
- 既存のテストが全て通過すること
- 単体テストが追加されていること

## 解決方法

1. `build_delete_objects_xml` の戻り値型は既に `Result<String, Error>` なので、`.expect()` を `?` に変更する
2. `.unwrap_or(0)` を `.map_err(|_| Error::InvalidInput("last_modified_time is before UNIX epoch"))?` に変更する
3. テストを追加する
4. CHANGES.md の `## develop` に `[FIX]` エントリを追加する
