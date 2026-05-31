# プロジェクト規約準拠（lint / CHANGES / 依存指定）

- Priority: Medium
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-05-31

## 目的

CLAUDE.md / AGENTS.md で定められた規約違反を解消する。機能バグではないが、リリース品質と CI 一貫性に影響する。

## 優先度根拠

規約違反の放置は新規コードの基準を曖昧にする。CHANGES.md の順序・内容不整合はリリースノートの信頼性を損なう。`#[allow]` の使用は lint 項目が不要になった時に気づけない。CI の `--all-targets` 欠如はローカルと CI の結果が一致しない。

## 現状

| 項目 | 場所 |
|------|------|
| `#[allow(...)]` 使用 | `src/api/mod.rs` (3 箇所), `examples/s3cli/` (6 箇所: `upload.rs` 1, `ops.rs` 4, `commands.rs` 1) |
| `compile_error!` が日本語 | `src/lib.rs:3` |
| CHANGES.md 種別順序 | ADD → CHANGE 混在 |
| CHANGES.md HttpDate 矛盾 | L52 ADD / L96 CHANGE 廃止 / L132 FIX 共存 |
| CHANGES.md misc 重複 | shiguredo_http11 中間更新記述 (L137, L139, L141) |
| `examples/s3cli/Cargo.toml` | `shiguredo_http11 = "2026.5.0"` パッチ指定 |
| `examples/s3cli/Cargo.toml` | `tokio = "1"` マイナー未指定 |
| CI clippy | `--all-targets` なし（`ci.yml:25`） |
| 解決済み issue 参照 TODO | `types.rs` 112, 457 等 |
| `docs/AWS_SDK_RUST.md` | (*) 凡例が未使用 |

## 設計方針

### lint 警告の抑制方法変更

`#[allow]` を `#[expect]` に変更する。対象: `src/api/mod.rs` (3 箇所), `examples/s3cli/` (6 箇所: `upload.rs` 1, `ops.rs` 4, `commands.rs` 1)。

### エラーメッセージの英語化

`src/lib.rs:3` の `compile_error!` を日本語から英語に変更する。

### CHANGES.md の整理

- 種別順序を CHANGE → ADD → UPDATE → FIX 順に整理する
- HttpDate の矛盾を整理する: AGENTS.md の「変更履歴は派生元ブランチとの最終的な差分のみを記載すること」に従い、廃止された L52 ADD（`HttpDate::try_from_imf_fixdate()` 追加）は削除し、L96 CHANGE（廃止）と L132 FIX（バグ修正）を残す
- shiguredo_http11 の中間更新記述を統合する: L137（2026.2 → 2026.5）と L141 の shiguredo_http11 部分（2026.1 → 2026.2）を「2026.1 → 2026.5」に統合する。L141 の `hmac` / `md-5` / `sha1` / `sha2` の更新記述は別途残す。L139（API 変更追従）は内容が異なるため別途残す

### 依存バージョンの修正

- `shiguredo_http11 = "2026.5.0"` を `shiguredo_http11 = "2026.5"` に変更する
- 注: `tokio = "1"` は Cargo のセマンティックバージョニングで `>=1.0.0, <2.0.0` を意味し、`"1.0"` と等価。変更不要

### CI の clippy コマンド修正

`cargo clippy --workspace -- -D warnings` を `cargo clippy --workspace --all-targets -- -D warnings` に変更する。注: `--all-features` は `Cargo.toml` の features が `rust-crypto` と `aws_lc_rs` の排他選択であるため使用不可。

### 陳腐化コメントの削除

`types.rs` 112, 457 行目の解決済み issue 参照 TODO を削除する。issue 0059 は解決済みであり、「後続で型化検討」というコメントは issue 0084 として登録済みのため、重複コメントは不要。

### docs/AWS_SDK_RUST.md の凡例削除

`(*)` 凡例が未使用のため削除する。

## AWS S3 API Reference

該当なし（インフラ整備 issue）。

## 完了条件

- `#[allow]` が全て `#[expect]` に変更される
- `compile_error!` が英語になる
- CHANGES.md の種別順序が CHANGE → ADD → UPDATE → FIX になる
- `shiguredo_http11` のバージョン指定が `"2026.5"` になる
- `cargo clippy --workspace --all-targets -- -D warnings` が CI でも実行される
- 解決済み issue 参照 TODO が削除される
- `docs/AWS_SDK_RUST.md` の `(*)` 凡例が削除される
