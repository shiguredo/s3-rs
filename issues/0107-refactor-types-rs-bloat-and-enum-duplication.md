# types.rs の肥大化解消と enum 実装の重複排除

- Priority: Medium
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/refactor-types-rs-bloat-and-enum-duplication

## 目的

`src/types.rs` (1938 行) に全 API の出力型・リクエスト型・enum 型・Builder 型が詰め込まれている。責務別にサブモジュール分割し、8 つの enum で手動コピペされている `as_str()`/`From<&str>`/`Display` 実装をマクロで一括生成する。

## 優先度根拠

1938 行の単一ファイルは変更時の影響範囲把握が困難。8 enum の手動コピペ実装は variant と文字列の不一致バグのリスクがある。AGENTS.md の「テストが長くなるのはモジュール自体が大きすぎるサインなので `src/<module>.rs` 側の分割を検討すること」に合致。

## 現状

`types.rs` に以下の型が混在:
- 出力型 (~30 struct): `GetObjectOutput`, `PutObjectOutput`, `HeadObjectOutput` 等
- enum 型 (8 enum): `ChecksumAlgorithm`, `ChecksumMode`, `ServerSideEncryption`, `ObjectCannedAcl`, `StorageClass`, `MetadataDirective`, `TaggingDirective`, `EncodingType`
- Builder 型: `DeleteBuilder`, `TaggingBuilder`, `ServerSideEncryptionByDefaultBuilder`, `CorsRuleBuilder` 等
- バリデーション: `validate_imf_fixdate`

8 enum すべてで `as_str()` + `From<&str>` + `fmt::Display` の 3 実装が同型パターンで手動コピペされている。

## 設計方針

1. `src/types/` ディレクトリモジュールに分割する
2. `pub use` で後方互換を維持する
3. enum の `as_str()`/`From<&str>`/`Display` 実装を宣言的マクロで一括生成する

## 完了条件

- `src/types/` ディレクトリが作成され、責務別にファイルが分割されていること
- 8 enum の実装がマクロで一括生成されていること
- 公開 API の import パスが変更されていないこと（後方互換）
- 既存のテストが全て通過すること

## 解決方法

1. `src/types/` ディレクトリを作成
2. `src/types/output.rs` に出力型を移動
3. `src/types/enums.rs` に enum 型を移動
4. `src/types/builders.rs` に Builder 型を移動
5. enum の実装をマクロに置き換え
6. `src/types.rs` を `pub use` の再エクスポートのみに縮小
7. CHANGES.md の `## develop` の `### misc` にエントリを追加する
