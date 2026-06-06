# proptest 導入と単体テスト配置規約の遵守

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Polished: 2026-06-06
- Branch: feature/add-proptest-and-test-layout

## 目的

AGENTS.md が要求する PBT（proptest）と `tests/test_<module>.rs` 配置が未整備。統合テストのみに依存するテスト戦略を PBT + 単体テスト + fuzzing の 3 層に移行する。

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

`pbt/Cargo.toml` を新規作成し、`[workspace] members` に追加する。proptest を dependency に記載する。

### PBT の対象モジュール

- `datetime`: `unix_timestamp_from_civil` → `civil_from_unix_timestamp` のラウンドトリップ
- `signing`: `uri_encode_path` と `uri_encode_component` の差異プロパティ、全 ASCII 文字エンコード結果検証
- `checksum`: Base64 デコード結果のバイト長がアルゴリズムの出力長と一致する

### inline テストの分類

AGENTS.md の「PBT でカバーできるものを単体テストで書かない」に従い、inline テストを分類する:

- PBT に移行: ラウンドトリップ可能なテスト
- 単体テストに移行: 固定値テスト（AWS 公式ベクトルとの一致検証等）
- `src/api/mod.rs` の inline テスト 4 件は `src/api/mod.rs` 内 `#[cfg(test)]` に留置

### `pub(crate)` 関数のテスト配置

`signing.rs` の `pub(crate)` 関数は、`pub` 昇格による API サーフェス拡大を避けるため `src/signing.rs` 内 `#[cfg(test)]` に留置する。

### 署名テストの強化

AWS Signature Version 4 Test Suite のベクトルで `Signature=` 完全一致を検証する。現在の `starts_with` / `contains` 検証を修正する。

### fuzz CI の smoke 実行

fuzz を CI に smoke 実行を追加する。対象 target: `fuzz_httpdate`, `fuzz_xml_parse`。各 target を 10 秒回す。nightly Rust と `cargo-fuzz` を CI で使用する。

### カバレッジ駆動のテスト作成

PBT 導入後、llvm-cov でカバレッジを取得し、未カバー行を分類する。

### サブ issue 分割

本 issue は複数の独立した作業を含むため、サブ issue に分割する:

1. proptest 導入（`pbt/Cargo.toml` 作成、`prop_datetime.rs` 作成）
2. 署名テスト強化（AWS 公式ベクトルでの完全一致検証）
3. fuzz CI の smoke 実行追加
4. カバレッジ計測と未カバー行の分類

## AWS S3 API Reference

- AWS Signature Version 4 Test Suite: <https://docs.aws.amazon.com/general/latest/gr/signature-v4-test-suite.html>

## 完了条件

- `pbt/tests/prop_datetime.rs` 等が存在し `cargo test -p pbt --test prop_datetime` が通る
- `src/signing.rs`, `src/datetime.rs`, `src/checksum.rs` 内の `#[cfg(test)]` に留置されたテストが整理される
- 署名テストが AWS 公式ベクトルで完全一致検証になる
- fuzz CI に smoke 実行が追加される
- `#[ignore]` 不使用を維持
- モック / スタブ不使用を維持
