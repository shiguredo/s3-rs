# ConfigBuilder の set_* メソッドのセマンティクスを統一する

- Priority: Low
- Created: 2026-07-07
- Completed: 2026-07-29
- Model: hy3-free
- Branch: feature/change-config-builder-set-semantics

## 目的

`ConfigBuilder` の `set_*` メソッドで `None` の扱いが場所によって異なる（クリア vs no-op）非対称を解消し、aws-sdk-rust 互換の「`Option` で丸ごと置換」に統一する。

## 優先度根拠

- 同一構造体内で `set_*` の意味論が二通り存在し、利用者が期待する挙動が場所によって変わる。
- aws-sdk-rust の `set_*` は「渡した `Option` で丸ごと置換（None ならクリア）」が標準であり、本クレートの `region` / `endpoint` 側が aws-sdk-rust 互換、`force_path_style` / `ignore_cert_check` 側が独自挙動になっている。

## 現状

- `src/client.rs:96,108,120`（`set_region` / `set_endpoint` / `set_credentials_provider`）は `self.x = x;`（None を渡すとクリア）。
- `src/client.rs:132-137,146-151`（`set_force_path_style` / `set_ignore_cert_check`）は `if let Some(v) = x { self.x = v; }`（None は既存値を維持 = no-op）。

## 設計方針

- すべての `set_*` を「`Option` で丸ごと置換」に統一する（aws-sdk-rust 互換）。
- 変更は後方互換のない挙動変更となるため `[CHANGE]` として扱う。
- `ConfigBuilder` 以外のビルダー（`*Builder` 各種）でも同様の不統一がないか併せて確認する。

## 完了条件

- `ConfigBuilder` の全 `set_*` が `Option` で丸ごと置換する挙動になること。
- `CHANGES.md` の `## develop` セクションに `[CHANGE]` エントリを記載すること。

## 解決方法

コミット `b85ba28` で実装済み。`set_force_path_style` / `set_ignore_cert_check` を `self.x = x;`（Option で丸ごと置換）に統一。全ビルダーの `set_*` も統一済みであることを確認。CHANGES.md に `[CHANGE]` エントリ記載済み。
