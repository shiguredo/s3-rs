# types.rs の肥大化解消

- Priority: Medium
- Created: 2026-07-12
- Model: Composer 2.5 Fast
- Branch: feature/refactor-types-rs-split
- Polished: 2026-08-02

## 目的

`src/types.rs` (1938 行) に全 API の出力型・入力型・設定型・enum 型・Builder 型・バリデーション関数が詰め込まれている。責務別に `src/types/` ディレクトリモジュールに分割する。

## 優先度根拠

1938 行の単一ファイルは変更時の影響範囲把握が困難。shiguredo-rust スキルの「テストファイルが長くなった場合はファイル内で `mod` を使って分割すること。テストが長くなるのはモジュール自体が大きすぎるサインなので `src/<module>.rs` 側の分割を検討すること」の趣旨（肥大化したモジュールの分割）に合致する規模である。

## 現状

`types.rs` に以下の型が混在:

| カテゴリ | 例 | 備考 |
|----------|-----|------|
| 出力型 (56 struct) | `GetObjectOutput`, `PutObjectOutput`, `HeadObjectOutput`, `CopyObjectOutput` 等 | 全 API のレスポンス型（`*Output` 構造体） |
| レスポンスコンポーネント型 (12 struct) | `Object`, `Owner`, `RestoreStatus`, `Bucket`, `Part`, `MultipartUpload`, `ObjectVersion`, `DeleteMarkerEntry`, `CommonPrefix`, `DeletedObject`, `DeleteError`, `CopyObjectResult` | 出力型のフィールドを構成する副構造体 |
| 入力・リクエストボディ型 | `ObjectIdentifier`, `Delete`, `CompletedPart`, `CompletedMultipartUpload`, `CreateBucketConfiguration`, `Tagging`, `ObjectLockConfiguration` 等 | XML シリアライズを持つ型。`Tag` は入出力共用 |
| 設定型 | `ServerSideEncryptionByDefault`, `CorsRule`, `LifecycleRule`, `TopicConfiguration`, `RoutingRule`, `OwnershipControlsRule` 等 | バケット設定関連 |
| Builder 型 (7 struct) | `DeleteBuilder`, `TaggingBuilder`, `CorsRuleBuilder`, `ServerSideEncryptionByDefaultBuilder` 等 | 設定型のビルダー |
| enum 型 (9 enum) | `ChecksumAlgorithm`, `ChecksumMode`, `ServerSideEncryption`, `ObjectCannedAcl`, `StorageClass`, `MetadataDirective`, `TaggingDirective`, `EncodingType`, `ExpirationStatus` | `ExpirationStatus` は他 8 個と異なり `FromStr` を実装し `Unknown(String)` variant を持たない |
| バリデーション | `validate_imf_fixdate` | 日付検証関数 |

## 設計方針

1. `src/types/` ディレクトリモジュールに分割する。分割先のファイル分類は以下を目安とする（実装時に調整可）:
   - `output.rs`: 出力型（`*Output` 構造体 56 個）+ レスポンスコンポーネント型（`Object`, `Owner`, `Bucket`, `Part`, `MultipartUpload`, `ObjectVersion`, `DeleteMarkerEntry`, `CommonPrefix`, `DeletedObject`, `DeleteError`, `CopyObjectResult`, `RestoreStatus` の 12 個。出力型のフィールドを構成する副構造体であり、出力型と同じファイルにまとめる）
   - `model.rs`: 入力・リクエストボディ型・設定型・Builder 型（`Tag` 等の入出力共用型も含む）
   - `enums.rs`: enum 型（9 enum すべて。`ExpirationStatus` を含む。`src/types.rs` の区切りコメント「型付き enum (aws-sdk-rust 互換)」と 4 項目の実装方針（`#[non_exhaustive]`・`Unknown(String)`・`as_str()` / `From<&str>`・variants 同期）も enums.rs の先頭に移動する）
   - `mod.rs`: 再エクスポート + `validate_imf_fixdate`
   サブモジュールの可視性は `mod output;` / `mod model;` / `mod enums;`（すべて private）とし、`pub mod` にはしない。`pub mod` にすると `shiguredo_s3::types::output::GetObjectOutput` という新公開パスが生まれ、API 表面が拡大するため
2. `src/types/mod.rs` に `pub use` で再エクスポートし、`crate::types::XxxOutput` パスを維持する。tests/examples/README.md が `shiguredo_s3::types::{...}` パスで import しているため（`tests/test_xml.rs` の `use shiguredo_s3::types::{Delete, ObjectIdentifier, Tag, Tagging};`、`examples/s3cli/src/upload.rs` の `use shiguredo_s3::types::{CompletedMultipartUpload, CompletedPart, PutObjectOutput};` 等）、後方互換の維持のために再エクスポートが必要。これは自クレート内のサブモジュールを mod.rs で束ねる標準パターンであり、shiguredo-rust の「re-export は基本的にやらないこと」規約（外部クレートの再公開を対象とする）の対象外である
3. enum の手動実装はそのまま維持する（8 enum: `as_str()` / `From<&str>` / `Display`、`ExpirationStatus`: `as_str()` / `FromStr`。shiguredo-rust の「マクロを作らないこと」規約に従う。重複排除は本 issue のスコープ外とする）
4. 既存の `#[non_exhaustive]` 属性は維持する（除去の要否は別途判断する。shiguredo-rust スキルは「`#[non_exhaustive]` を使わないこと」を定めており、8 箇所が違反状態にあるが、除去は互換性影響を伴うため本 issue では扱わない）

## 完了条件

- `src/types/` ディレクトリが作成され、責務別にファイルが分割されていること
- 外部クレートからの公開 API パス（`shiguredo_s3::types::GetObjectOutput` 等）およびクレート内部パス（`crate::types::GetObjectOutput` 等）が変更されていないこと
- `lib.rs` の `pub use types::{...}` によるクレートルート再エクスポート（`shiguredo_s3::ChecksumAlgorithm` 等）も変更されていないこと
- 既存のテストが全て通過すること（`cargo test --workspace --lib`。統合テストは本 issue の変更範囲外だが、CI で通過すること）
- `cargo clippy --workspace --all-targets -- -D warnings` が通過すること

## 解決方法

1. `src/types/` ディレクトリを作成する
2. `src/types/output.rs` に出力型を移動する
3. `src/types/model.rs` に入力・設定・Builder 型を移動する
4. `src/types/enums.rs` に enum 型を移動する
5. `src/types/mod.rs` を `pub use` の再エクスポートと `validate_imf_fixdate` のみに縮小する
6. `lib.rs` の `mod types;` 宣言がディレクトリモジュールとして解決されることを確認する
7. CHANGES.md の `## develop` の `### misc` にエントリを追加する

## 他 issue との依存関係

- 0106（api/mod.rs 責務分離）は `src/request.rs` を使い、`src/types.rs` は変更しない。本 issue と干渉しない
- 0126（operation specific input fields）、0128（flexible checksum models）、0129（remaining output fields）、0130（s3 model types）、0131（fluent builder set methods）は `src/types.rs` に型を追加・変更する。本 issue 実装後はこれらが参照するパスが `src/types/` ディレクトリになるが、`mod.rs` の再エクスポートにより `crate::types::Xxx` パスは維持されるため、順序制約はない
