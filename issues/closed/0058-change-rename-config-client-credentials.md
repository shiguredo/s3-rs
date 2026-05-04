# Config / Client / Credentials を aws-sdk-rust スタイルにリネームする

Created: 2026-05-04
Completed: 2026-05-04
Model: Opus 4.7

## 根拠

- `AGENTS.md` / `CLAUDE.md` で「利用者が aws-sdk-rust へ移行する際に違和感を感じないように、aws-sdk-rust スタイルの API を提供する」と明記されている。
- aws-sdk-rust では `aws_sdk_s3::Config`、`aws_sdk_s3::Client` という名称であり、`S3` プレフィックスを付けない。
- aws-sdk-rust では `aws_credential_types::Credentials` (複数形) であり、単数形の `Credential` は採用していない。
- `S3Client::new(S3Config)` は名前が `Client::new(&SdkConfig)` と紛らわしいが、aws-sdk-rust では `Client::from_conf(Config)` が `Config` を受ける正しい入り口となっており、意味論が異なる。利用者の混乱を避けるため `from_conf` に揃える。
- ビルダーメソッド名 `credential` / `use_path_style` は aws-sdk-rust の `credentials_provider` / `force_path_style` と異なる。
- 現状は `2026.1.0-canary.3` の canary 版であり、外部利用者は限定的なため、破壊的変更を加える適切な時期である。

## 変更内容

### 型のリネーム

| 旧 | 新 |
|---|---|
| `S3Config` | `Config` |
| `S3ConfigBuilder` | `config::Builder` (もしくは `ConfigBuilder` を維持) |
| `S3Client` | `Client` |
| `Credential` | `Credentials` |

### コンストラクタのリネーム

| 旧 | 新 |
|---|---|
| `S3Client::new(config)` | `Client::from_conf(config)` |
| `Credential::new(access_key_id, secret_access_key)` | `Credentials::new(access_key_id, secret_access_key, session_token: Option<String>, expires_after: Option<SystemTime>, provider_name: &'static str)` または `Credentials::builder()` |
| `Credential::with_session_token(...)` | `Credentials::new(...)` で `session_token` を `Some` で渡す形に統合 |

### ビルダーメソッドのリネーム

| 旧 | 新 |
|---|---|
| `S3ConfigBuilder::credential(Credential)` | `Builder::credentials_provider(Credentials)` |
| `S3ConfigBuilder::use_path_style(bool)` | `Builder::force_path_style(bool)` |
| `S3ConfigBuilder::ignore_cert_check(bool)` | (現状維持、aws-sdk-rust に対応 API がないため独自フィールドとして残す) |

### 公開 API の更新範囲

- `src/lib.rs:15-17` の `pub use` 句を更新する。
- `src/client.rs:31-152` の構造体・実装をリネームする。
- `src/credential.rs:5-64` の構造体・実装をリネームする。
- `examples/s3cli/src/util.rs:5,184-210` を新 API に書き換える。
- `tests/minio.rs:32,79-93` を新 API に書き換える。
- `tests/rustfs.rs:38,96-110` を新 API に書き換える。

### 互換 alias は提供しない

破壊的変更として旧名 (`S3Config`, `S3Client`, `Credential`) は完全に削除する。`pub use S3Client = Client` のような alias は残さない。

## 参考

- aws-sdk-rust `Client`: <https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/struct.Client.html>
- aws-sdk-rust `Config`: <https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/config/struct.Builder.html>
- aws-credential-types `Credentials`: <https://docs.rs/aws-credential-types/latest/aws_credential_types/struct.Credentials.html>

## AWS S3 API Reference

- [Authenticating Requests (AWS Signature Version 4)](https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-authenticating-requests.html)

  > Amazon S3 API requests must be signed by an authenticated entity. Authentication information must be in the form of an AWS Signature Version 4 signature. The signing process is implemented by AWS SDKs and tools.

  本 issue で扱うクレデンシャル・リージョン・エンドポイント設定は、すべて Signature V4 の構成要素 (`Credential` スコープ、`Region`、`Service` 名) に対応する。aws-sdk-rust が採用している型名・ビルダー名はこの AWS 仕様の用語と整合しており、shiguredo_s3 でも同じ用語に揃えることで利用者が両者を行き来しやすくなる。

## 影響範囲

- 公開 API: `S3Config`, `S3ConfigBuilder`, `S3Client`, `Credential` を使用する全コードが破壊的変更の影響を受ける。
- 内部利用箇所: `examples/s3cli`, `tests/minio.rs`, `tests/rustfs.rs` を一括書き換え。

## 優先度

高 (他の issue の前提となる命名整理のため)

## CHANGES.md への記載

- `[CHANGE] S3Config を Config にリネームする`
- `[CHANGE] S3Client を Client にリネームする`
- `[CHANGE] S3Client::new を Client::from_conf にリネームする`
- `[CHANGE] Credential を Credentials にリネームする`
- `[CHANGE] ConfigBuilder::credential を credentials_provider にリネームする`
- `[CHANGE] ConfigBuilder::use_path_style を force_path_style にリネームする`

## 解決方法

### 実施した変更

1. **型のリネーム**
   - `src/client.rs`: `S3Config` → `Config`, `S3ConfigBuilder` → `ConfigBuilder`, `S3Client` → `Client`
   - `src/credential.rs`: `Credential` → `Credentials`
   - `src/api/mod.rs`: 内部型 `S3ClientConfig` → `ClientConfig`
   - `src/lib.rs`: `pub use` 句を新名称に更新

2. **`Client::from_conf` への変更**
   - `S3Client::new(config)` を廃止し `Client::from_conf(config)` に統一
   - aws-sdk-rust の `Client::from_conf(Config)` 流入口に揃える

3. **`Credentials::new` シグネチャの拡張**
   - aws-credential-types::Credentials::new と同じ 5 引数 (access_key_id, secret_access_key, session_token, expires_after, provider_name) に変更
   - `Credential::with_session_token` を廃止し、上記の `session_token` 引数で表現
   - `expires_after` と `provider_name` フィールドは保持するが署名計算では未使用 (情報目的のみ)

4. **ビルダーメソッドのリネーム**
   - `credential(Credential)` → `credentials_provider(Credentials)`
   - `use_path_style(bool)` → `force_path_style(bool)`
   - 同名のゲッターも setter と揃えてリネーム
   - 内部フィールドも `credentials_provider` / `force_path_style` に揃える
   - `ignore_cert_check` は aws-sdk-rust に対応 API がないため shiguredo_s3 独自として維持

5. **互換 alias は提供しない**
   - 旧名 (`S3Config`, `S3Client`, `Credential`) は完全削除
   - `2026.1.0-canary.3` の canary 版のため破壊的変更を許容

6. **関連箇所の更新**
   - `src/api/*.rs` 56 ファイル: `&'a S3Client` → `&'a Client` を一括更新
   - `src/signing.rs`: `SigningParams.credential` / `PresignParams.credential` を `credentials` にリネーム、テストも `Credentials::new` の新シグネチャに対応
   - `examples/s3cli/src/util.rs`: `build_client` を新 API に書き換え
   - `tests/minio.rs`, `tests/rustfs.rs`: テストヘルパを新 API に書き換え
   - `README.md`: 全コード例を新 API に更新
   - `CHANGES.md`: 10 件の `[CHANGE]` エントリを追加 (issue spec の 6 件 + getter リネーム等の追加 4 件)
   - 関連 issue ファイル (0061, 0069) の `S3Client` 参照も更新

### 検証結果

- `cargo check --workspace --all-targets`: 成功
- `cargo clippy --workspace --all-targets`: 警告ゼロ
- `cargo test --lib`: 12 tests passed
- `cargo test --test minio test_object_put_get_head_delete`: passed
- `cargo test --test minio test_presigned_put_get_head_delete`: passed (signing path 検証)
- `cargo test --test rustfs test_object_put_get_head_delete`: passed
- pre-commit hook: cargo fmt / clippy / test すべて pass
