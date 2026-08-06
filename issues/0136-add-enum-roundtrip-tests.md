# enum 型の往復変換テストを追加する

- Priority: Low
- Created: 2026-08-06
- Completed: {YYYY-MM-DD}
- Branch: feature/add-enum-roundtrip-tests
- Polished: {YYYY-MM-DD}

## 目的

`src/types/enums.rs` の enum 型の変換ロジック (`as_str()` / `From<&str>` / `FromStr` / `Unknown(String)` 保持) に単体テストがなく、変換が壊れても検出できない。往復変換を単体テストで固定する。

## 現状

`src/types/enums.rs` の以下の enum にテストがない:

- 8 enum (`ChecksumAlgorithm` / `ChecksumMode` / `ServerSideEncryption` / `ObjectCannedAcl` / `StorageClass` / `MetadataDirective` / `TaggingDirective` / `EncodingType`): `as_str()` / `From<&str>` / `Display` を実装し、未知の値は `Unknown(String)` に保持する
- `ExpirationStatus`: 方針の例外として `FromStr` を実装し (Unknown variant なし)、不明な値は `Error::InvalidResponse` を返す

分割前の `src/types.rs` にもテストは存在しなかったため回帰ではないが、`#[non_exhaustive]` + `Unknown(String)` を持つ複雑な変換のため、回帰検知の価値が高い。

## 設計方針

`src/types/enums.rs` の `#[cfg(test)] mod tests` に以下を追加する:

1. 各 enum の `as_str()` → `From<&str>` のラウンドトリップ (文字列 → enum → 文字列が同一値に戻ること)
2. 未知の文字列が `From<&str>` で `Unknown(String)` になり、`as_str()` で元の文字列が返ること
3. `ExpirationStatus` の `FromStr` (`Enabled` / `Disabled` の通過、未知の値の `Error::InvalidResponse` 拒否)

## 完了条件

- 上記 3 種の変換が全 enum に対して単体テストで検証されていること
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
