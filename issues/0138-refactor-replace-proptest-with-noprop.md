# proptest を noprop に置き換える

- Created: 2026-08-17
- Completed: {YYYY-MM-DD}
- Branch: feature/refactor-replace-proptest-with-noprop
- Polished: {YYYY-MM-DD}

## 目的

PBT (Property-Based Testing) 基盤を proptest から noprop に置き換える。noprop はマクロやコンビネータ DSL を持たず、素の Rust クロージャと関数でプロパティテストを書けるため、マクロベースの proptest より可読性が高く、依存も軽くなる。

## 現状

proptest を使っている箇所は以下に限られる:

- `pbt/Cargo.toml` の `[dev-dependencies]` に `proptest = "1.6"` を記載
- `pbt/tests/prop_datetime.rs` が `proptest!` マクロで `datetime_round_trip` (src/lib.rs の `datetime_round_trip` 関数) のラウンドトリップを検証

`pbt/tests/prop_datetime.rs` が唯一の PBT ファイルであり、ここだけを書き換えれば置き換えが完了する。

## 設計方針

- `pbt/Cargo.toml` の `proptest` 依存を `noprop` (バージョンは 0.2) に置き換える
- `pbt/tests/prop_datetime.rs` を noprop の命令型スタイルで書き直す
  - `noprop::Runner` と `noprop::sample_usize_in` などを使って年・月・日・時・分・秒を生成する
  - プロパティは現行と同じラウンドトリップ (`unix_timestamp_from_civil` → `civil_from_unix_timestamp` で元の日時が復元されること)
  - 存在しない暦日 (例: 2/31) が `Err` を返すパスを正しく扱うため、成功時のみ復元値を検証する
- 検証が空振りしないよう、ラウンドトリップが成功したケースをカウントするカバレッジゲートを付ける (noprop の流儀)

## 完了条件

- `pbt/Cargo.toml` から `proptest` 依存がなくなり、`noprop` に置き換わっていること
- `pbt/tests/prop_datetime.rs` が noprop で書き直され、ラウンドトリッププロパティがカバレッジゲート付きで検証されていること
- `cargo test -p pbt --test prop_datetime` が通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
