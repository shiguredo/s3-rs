# DeleteObjects で利用者入力の SystemTime によるパニック経路を修正する

- Priority: High
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Polished: 2026-07-23
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

1. `.unwrap_or(0)` を `?` によるエラー伝播に変更する: `t.duration_since(UNIX_EPOCH).map_err(|_| Error::InvalidInput("last_modified_time is before UNIX epoch"))?.as_secs()`
2. `.expect()` を `?` に変更する: `let c = crate::datetime::civil_from_unix_timestamp(secs)?;`
3. エラーメッセージを修正する（「S3 response」→「ユーザー入力」）

**注意**: 本 issue と issue 0097（DeleteObjects の LastModifiedTime 形式を ISO 8601 から IMF-fixdate に修正）は同一コードブロックを対象とする。0097 の修正（手動フォーマットブロック全体を `format_imf_fixdate(t)?` に置き換え）を実施すれば、本 issue の `.expect()` パニックと `.unwrap_or(0)` 黙殺の両方が自動的に解消される。0097 で一括修正し、本 issue は 0097 の完了後に close することを推奨する。

## 完了条件

- `ObjectIdentifier.last_modified_time` に UNIX_EPOCH 以前の `SystemTime` を設定した場合にパニックせず `Error::InvalidInput` が返ること
- `civil_from_unix_timestamp` が扱えない極端な値（オーバーフロー等）でも `Error::InvalidInput` が返ること
- `tests/test_delete_objects.rs` に上記エラーパスのテストを追加すること。ただし 0097 と同時に修正する場合は 0097 のテストでカバーされるため、別途追加不要
- `CHANGES.md` の `## develop` に `[FIX]` エントリを記載すること（0097 と同じコミットで修正する場合は 1 エントリに統合可能）

## 解決方法

1. `build_delete_objects_xml` の戻り値型は既に `Result<String, Error>` なので、`.expect()` を `?` に変更する
2. `.unwrap_or(0)` を `.map_err(|_| Error::InvalidInput("last_modified_time is before UNIX epoch"))?` に変更する
3. テストを追加する
4. CHANGES.md の `## develop` に `[FIX]` エントリを追加する
