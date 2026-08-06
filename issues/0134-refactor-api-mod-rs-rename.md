# src/api/mod.rs を src/api.rs にリネームする

- Priority: Medium
- Created: 2026-08-06
- Completed: {YYYY-MM-DD}
- Branch: feature/refactor-api-mod-rs
- Polished: {YYYY-MM-DD}

## 目的

`src/api/mod.rs` は `shiguredo-rust` の「`mod.rs` を使わないこと。モジュールは `<module>.rs` で書くこと」という規約に違反している。`src/api.rs` にリネームして規約に準拠する。

## 現状

`src/api/mod.rs` (477 行) が `api` モジュールのルートファイルとして存在し、58 個のサブモジュール (`src/api/*.rs`) を宣言している。`shiguredo-rust` 規約は `<module>.rs` + `<module>/<submodule>.rs` の構成を要求しており、`mod.rs` は禁止されている。

## 設計方針

1. `git mv src/api/mod.rs src/api.rs` でリネームする
2. `src/lib.rs` の `pub mod api;` は変更不要（Rust は `src/api.rs` + `src/api/` ディレクトリの構成で `api` モジュールを解決する）
3. サブモジュールの `use super::{...}` 参照と、外部クレートからの `shiguredo_s3::api::*` パスはリネームの影響を受けない

## 完了条件

- `src/api/mod.rs` が存在せず、`src/api.rs` に置き換わっていること
- 公開 API パス (`shiguredo_s3::api::*`) が変更されていないこと
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること
