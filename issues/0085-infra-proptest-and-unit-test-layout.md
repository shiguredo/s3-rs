# proptest 導入と単体テスト配置規約の遵守

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-05-31

## 目的

CLAUDE.md / AGENTS.md が要求する PBT（proptest）と `tests/test_<module>.rs` 配置が未整備。統合テストのみに依存するテスト戦略を PBT + 単体テスト + fuzzing の 3 層に移行する。

## 優先度根拠

signing / datetime / checksum は PBT と単体テスト向きだが、`src/` 内 `#[cfg(test)]` のみ。オフラインでの欠陥検出力が弱く、リグレッションを統合テストだけではカバーしきれない。

## 現状

- `pbt/` ディレクトリなし
- `Cargo.toml` に `proptest` 依存なし
- `tests/test_*.rs` 0 件
- 単体テストは `src/signing.rs`, `src/datetime.rs`, `src/checksum.rs`, `src/api/mod.rs` 内 inline
- fuzz は `fuzz_httpdate.rs` と `fuzz_xml_parse.rs` が実装済み。CI 未実行

## 設計方針

### proptest の導入

`pbt/Cargo.toml` を新規作成し、`[workspace] members` に追加する。proptest を `pbt/Cargo.toml` の dependency に記載する。

### PBT の対象モジュール

- `datetime`: `unix_timestamp_from_civil` → `civil_from_unix_timestamp` のラウンドトリップ（注: `second` は `0..=59` の範囲で生成する。`second == 60`（うるう秒）は `second.min(59)` で 59 に丸めるため、round-trip が成立しない）
- `signing`: `uri_encode_path` と `uri_encode_component` の差異プロパティ（path は `/` を残し component は `%2F` にする）、全 ASCII 文字に対するエンコード結果が `%XX` 形式か安全文字のみであることの検証（注: signing.rs にデコード関数が存在しないため、ラウンドトリップ PBT は不可能）
- `checksum`: Base64 デコード結果のバイト長がアルゴリズムの出力長と一致する

注: `xml.rs` には `#[cfg(test)]` が存在しないため、PBT は新規作成となる。検証すべきプロパティ: 有効な XML パース結果の妥当性、エラー耐性。

### inline テストの分類

AGENTS.md の「PBT でカバーできるものを単体テストで書かない」に従い、inline テストを分類する:

- PBT に移行: ラウンドトリップ可能なテスト（`test_uri_encode_path_simple`, `test_uri_encode_component` 等）
- 単体テストに移行: 固定値テスト（`test_compute_authorization` 等、AWS 公式ベクトルとの一致検証）
- `src/api/mod.rs` の inline テスト（deterministic, varies_with_now, presigned, reject_pre_epoch）は `tests/test_api_mod.rs` に移行

### `pub(crate)` 関数のテスト配置

`signing.rs` の `uri_encode_path`, `compute_authorization` 等は `pub(crate)`。`tests/test_signing.rs` に移行する場合、`pub` に昇格すると crate の公開 API サーフェスが拡大する。AGENTS.md の「依存は最小限にすること」に従い、`pub(crate)` のまま `src/signing.rs` 内 `#[cfg(test)]` に留置する方針を採用する。

### 署名テストの強化

AWS Signature Version 4 Test Suite のベクトルで `Signature=` 完全一致を検証する。現在の `starts_with` / `contains` 検証を修正し、期待する完全な署名値をベクトルから取得して一致検証する。具体的なベクトル: `get-vanilla`、`get-vanilla-query-unreserved`、`get-utf8` 等。

### fuzz CI の smoke 実行

fuzz を CI に smoke 実行を追加する。対象 target: `fuzz_httpdate`, `fuzz_xml_parse`（注: `fuzz_target_1` はスケルトンのため対象外）。各 target を 10 秒回す。`Cargo.toml` で fuzz は `exclude` されているため、nightly Rust と `cargo-fuzz` を使用して CI でビルド・実行する。CI プラットフォーム: GitHub Actions。nightly のインストール: `rustup install nightly`。cargo-fuzz のインストール: `cargo install cargo-fuzz`。

### カバレッジ駆動のテスト作成

PBT 導入後、llvm-cov でカバレッジを取得し、未カバー行を分類する手順を完了条件に含める。

## AWS S3 API Reference

- AWS Signature Version 4 Test Suite: <https://docs.aws.amazon.com/general/latest/gr/signature-v4-test-suite.html>

## 完了条件

- `pbt/tests/prop_datetime.rs` 等が存在し `cargo test -p pbt --test prop_datetime` が通る
- `src/signing.rs`, `src/datetime.rs`, `src/checksum.rs` 内の `#[cfg(test)]` に留置されたテストが整理される
- `src/api/mod.rs` の inline テスト 4 件が `#[cfg(test)]` 内に留置される
- 署名テストが AWS 公式ベクトルで完全一致検証になる
- fuzz CI に smoke 実行が追加される
- `#[ignore]` 不使用を維持
- モック / スタブ不使用を維持
- llvm-cov でカバレッジを取得し、未カバー行を分類する

注: 本 issue は複数の独立した作業（proptest 導入、テスト整理、署名テスト強化、fuzz CI、カバレッジ計測）を含む。AGENTS.md の「1 issue 完了ごとに 1 コミットすること」に従い、サブ issue に分割する。分割先:

1. proptest 導入（`pbt/Cargo.toml` 作成、`prop_datetime.rs` 作成）
2. 署名テスト強化（AWS 公式ベクトルでの完全一致検証）
3. fuzz CI の smoke 実行追加
4. カバレッジ計測と未カバー行の分類
