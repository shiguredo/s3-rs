# CHANGES.md の shiguredo_http11 バージョン記述が実コードと不整合する

- Priority: Low
- Created: 2026-07-07
- Completed: 2026-07-07
- Model: hy3-free
- Branch: feature/fix-changes-md-shiguredo-http11-version

## 目的

`CHANGES.md` の `shiguredo_http11` バージョン記述を実コードの状態に合わせ、派生元ブランチとの最終差分のみを記載するという CHANGES 規約を守る。

## 優先度根拠

- AGENTS.md「変更履歴は派生元ブランチとの最終的な差分のみを記載する」「`## develop` セクションと現コードの整合性」を満たさない。
- リリース時に実態と異なる記述が残ると利用者が誤った前提を持つ。

## 現状

- `Cargo.toml:35`（本体 dev-dependency）: `shiguredo_http11 = "2026.6"`（Cargo.lock 上は `2026.6.1` に解決）。
- `examples/s3cli/Cargo.toml:10`: `shiguredo_http11 = "2026.5"`。
- `CHANGES.md:200,202,206,214`: 「2026.5 に更新・修正した」「`"2026.5"` に修正」と記載。

## 設計方針

- 実態（本体 2026.6 / examples 2026.5）を `CHANGES.md` に正しく反映する。本体を 2026.5 に戻すのか、記述を 2026.6 へ修正するのかを決定して統一する。
- 決定にあたっては直近のコミット履歴（`f9be039` shiguredo_http11 2026.6 への対応等）を確認し、意図したバージョンを特定する。

## 完了条件

- `CHANGES.md` の `shiguredo_http11` 記述が実コード（本体 / examples）と一致すること。
- 必要なら `examples/s3cli/Cargo.toml` のバージョンも実態に合わせて修正すること。
- `CHANGES.md` の `## develop` セクションに `[UPDATE]` または該当エントリの修正を記載すること。

## 解決方法

commit `b926444` で修正済み。

- `CHANGES.md` の `shiguredo_http11` バージョン記述を実コード（本体 2026.6 / examples 2026.6）と整合させた
