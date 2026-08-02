# types.rs の肥大化解消

- Priority: Medium
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/refactor-types-rs-split
- Polished: 2026-07-29

## 目的

`src/types.rs` (1938 行) に全 API の出力型・入力型・設定型・enum 型・Builder 型・バリデーション関数が詰め込まれている。責務別に `src/types/` ディレクトリモジュールに分割する。

## 優先度根拠

1938 行の単一ファイルは変更時の影響範囲把握が困難。shiguredo-rust スキルの「テストが長くなるのはモジュール自体が大きすぎるサインなので `src/<module>.rs` 側の分割を検討すること」に合致する規模である。

## 現状

`types.rs` に以下の型が混在:

| カテゴリ | 例 | 備考 |
|----------|-----|------|
| 出力型 (~56 struct) | `GetObjectOutput`, `PutObjectOutput`, `HeadObjectOutput`, `CopyObjectOutput` 等 | 全 API のレスポンス型 |
| 入力・リクエストボディ型 | `ObjectIdentifier`, `Delete`, `CompletedPart`, `CompletedMultipartUpload`, `CreateBucketConfiguration`, `Tagging`, `ObjectLockConfiguration` 等 | XML シリアライズを持つ型 |
| 設定型 | `ServerSideEncryptionByDefault`, `CorsRule`, `LifecycleRule`, `TopicConfiguration`, `RoutingRule`, `OwnershipControlsRule` 等 | バケット設定関連 |
| Builder 型 | `DeleteBuilder`, `TaggingBuilder`, `CorsRuleBuilder`, `ServerSideEncryptionByDefaultBuilder` 等 | 設定型のビルダー |
| enum 型 (9 enum) | `ChecksumAlgorithm`, `ChecksumMode`, `ServerSideEncryption`, `ObjectCannedAcl`, `StorageClass`, `MetadataDirective`, `TaggingDirective`, `EncodingType`, `ExpirationStatus` | `ExpirationStatus` は他 8 個と異なり `FromStr` を実装し `Unknown(String)` variant を持たない |
| バリデーション | `validate_imf_fixdate` | 日付検証関数 |

## 設計方針

1. `src/types/` ディレクトリモジュールに分割する。分割先のファイル分類は以下を目安とする（実装時に調整可）:
   - `output.rs`: 出力型
   - `model.rs`: 入力・リクエストボディ型・設定型・Builder 型
   - `enums.rs`: enum 型（9 enum すべて。`ExpirationStatus` を含む）
   - `mod.rs`: 再エクスポート + `validate_imf_fixdate`
2. `src/types/mod.rs` に `pub use` で再エクスポートし、`crate::types::XxxOutput` パスを維持する。tests/examples が `shiguredo_s3::types::{...}` パスで import しているため（`tests/test_xml.rs:6`, `examples/s3cli/src/upload.rs:12` 等）、後方互換の維持のために再エクスポートが必要。shiguredo-rust の「re-export は基本的にやらないこと」規約に対する例外として正当化する
3. enum の手動実装はそのまま維持する（8 enum: `as_str()` / `From<&str>` / `Display`、`ExpirationStatus`: `as_str()` / `FromStr`。shiguredo-rust の「マクロを作らないこと」規約に従う。重複排除は本 issue のスコープ外とする）
4. 既存の `#[non_exhaustive]` 属性は維持する（除去は別 issue で扱う）

## 完了条件

- `src/types/` ディレクトリが作成され、責務別にファイルが分割されていること
- 外部クレートからの公開 API パス（`shiguredo_s3::types::GetObjectOutput` 等）およびクレート内部パス（`crate::types::GetObjectOutput` 等）が変更されていないこと
- 既存のテストが全て通過すること
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること

## 解決方法

1. `src/types/` ディレクトリを作成する
2. `src/types/output.rs` に出力型を移動する
3. `src/types/model.rs` に入力・設定・Builder 型を移動する
4. `src/types/enums.rs` に enum 型を移動する
5. `src/types/mod.rs` を `pub use` の再エクスポートと `validate_imf_fixdate` のみに縮小する
6. `lib.rs` の `mod types;` 宣言がディレクトリモジュールとして解決されることを確認する
7. CHANGES.md の `## develop` の `### misc` にエントリを追加する
