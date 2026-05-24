# プロジェクト規約準拠（lint / CHANGES / 依存指定）

- Priority: Medium
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/fix-project-convention-compliance

## 目的

CLAUDE.md / AGENTS.md で定められた規約違反を解消する。機能バグではないが、リリース品質と CI 一貫性に影響する。

## 優先度根拠

規約違反の放置は新規コードの基準を曖昧にする。CHANGES.md の順序・内容不整合はリリースノートの信頼性を損なう。

## 現状

| 項目 | 場所 |
|------|------|
| `#[allow(...)]` 使用 | `src/api/mod.rs`, `examples/s3cli/` 等 8 箇所 |
| `compile_error!` が日本語 | `src/lib.rs:3` |
| CHANGES.md 種別順序 | ADD → CHANGE 混在 |
| CHANGES.md HttpDate 矛盾 | L52 ADD / L96 CHANGE 廃止 / L132 FIX 共存 |
| CHANGES.md misc 重複 | shiguredo_http11 中間更新記述 |
| `examples/s3cli/Cargo.toml` | `shiguredo_http11 = "2026.5.0"` パッチ指定 |
| `examples/s3cli/Cargo.toml` | `tokio = "1"` マイナー未指定 |
| CI clippy | `--all-targets` なし（`ci.yml:25`） |
| 解決済み issue 参照 TODO | `types.rs` 112, 457 等 |
| `docs/AWS_SDK_RUST.md` | (*) 凡例が未使用 |

## 設計方針

- `#[allow]` → `#[expect]`
- エラーメッセージは英語
- CHANGES.md ## develop を CHANGE → ADD → UPDATE → FIX 順に整理し、最終差分のみ残す
- 依存バージョンはマイナーまで
- CI を `prek.toml` / ローカルと揃える

## 完了条件

- 上記規約違反が解消される
- `cargo clippy --all-targets --all-features -- -D warnings` が CI でも実行される
- CHANGES.md と現コードが整合する

## 解決方法

1. 機械的置換（allow → expect, compile_error 英語化）
2. CHANGES.md 整理
3. Cargo.toml / ci.yml 修正
4. 陳腐化コメント削除
