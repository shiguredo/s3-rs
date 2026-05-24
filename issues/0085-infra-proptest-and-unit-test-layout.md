# proptest 導入と単体テスト配置規約の遵守

- Priority: High
- Created: 2026-05-25
- Model: Composer 2.5
- Branch: feature/add-proptest-and-unit-test-layout

## 目的

CLAUDE.md / AGENTS.md が要求する PBT（proptest）と `tests/test_<module>.rs` 配置が未整備。統合テストのみに依存するテスト戦略を三層（PBT + 単体 + fuzzing + 統合）に移行する。

## 優先度根拠

signing / datetime / xml / checksum は PBT と単体テスト向きだが、`src/` 内 `#[cfg(test)]` のみ。オフラインでの欠陥検出力が弱く、リグレッションを統合テストだけではカバーしきれない。

## 現状

- `pbt/` ディレクトリなし
- `Cargo.toml` に `proptest` 依存なし
- `tests/test_*.rs` 0 件
- 単体テストは `src/signing.rs`, `src/datetime.rs`, `src/checksum.rs`, `src/api/mod.rs` 内 inline
- fuzz は CI 未実行、`fuzz_target_1.rs` が空

## 設計方針

- `proptest` を dev-dependency に追加
- `pbt/tests/prop_<module>.rs` を作成（datetime, signing, checksum, xml）
- `src/` 内 `#[cfg(test)]` を `tests/test_<module>.rs` へ移行
- 署名テスト: AWS 公式ベクトルで `Signature=` 完全一致 + presigned
- fuzz: CI に smoke 実行を追加（任意: nightly）

## 完了条件

- `pbt/tests/prop_datetime.rs` 等が存在し `cargo test -p shiguredo_s3 --test prop_datetime` が通る
- `tests/test_signing.rs` 等に inline テストが移行される
- `#[ignore]` 不使用を維持
- モック / スタブ不使用を維持

## 解決方法

1. `Cargo.toml` に proptest 追加
2. PBT でラウンドトリップ（datetime, uri_encode, format/parse IMF-fixdate）
3. inline テスト移行
4. `tests/minio.rs` / `tests/rustfs.rs` の共通ヘルパーを `tests/common/mod.rs` に抽出（別途小さく対応可）
