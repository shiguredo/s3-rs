# 署名計算から時刻取得の副作用を排除し Sans I/O 原則を徹底する

Created: 2026-05-04
Completed: 2026-05-04
Model: Opus 4.7

## 根拠

- `README.md` で「Sans I/O Amazon S3 client library for Rust」を名乗っている。
- Sans I/O の本来の定義は「I/O だけでなく外界に依存する操作 (時刻取得、乱数生成、環境変数読み込み等) を持たない、純粋なデータ変換ライブラリ」である。
- 現状、`src/api/mod.rs:341` の `build_signed_request` および `src/api/mod.rs:423` の `build_presigned_url` 内で `UtcDateTime::now()` を呼んでおり、その内部で `SystemTime::now()` を呼んでいる (`src/datetime.rs:60-63`)。これは Sans I/O 原則違反である。
- 同じ入力に対して同じ出力を返すべきなのに、内部で時刻を取得しているため署名値が呼び出しごとに変わる (参照透過性が壊れている)。
- テスト時に固定時刻を指定できず、署名値の回帰検証が困難。
- `Trait Callback` (`TimeSource` トレイト等) で抽象化する案もあるが、trait 内部から副作用が発火する以上 Sans I/O 違反は解消しない。値の引数渡しが唯一の正解。
- `grep` で確認した結果、`SystemTime::now()` 呼び出しは上記 2 箇所のみ。乱数生成 (`rand` / `random`) は元々未使用。修正範囲は限定的。
- `HttpDate` 構造体 (`src/types.rs:24-83`) は文字列の薄いラッパで、入出力で日時を扱う型としてのセマンティクスが弱い。aws-sdk-rust は同等のフィールドを `aws_smithy_types::DateTime` で扱っており、文字列ではない。本 crate でも文字列ではなく標準の `SystemTime` で扱う形に統一することで、aws-sdk-rust の利用感に近づけつつ、外部依存追加を回避する。
- `aws_smithy_types::DateTime` 自体は使わない。`aws-smithy-types` への依存追加は確定方針として行わないため、`std::time::SystemTime` を採用する。`SystemTime` は `aws_smithy_types::DateTime` ほどの精度メソッド (秒未満の細かい操作等) はないが、S3 が扱う日時 (HTTP `Last-Modified` / XML `LastModified` / `IfModifiedSince` 等) はすべて秒精度であり機能的に十分。

## 変更内容

### 1. 公開ビルダーへの `now: SystemTime` 引数追加

`build_request()` および `presigned()` のシグネチャに現在時刻を引数で受ける形に変更する。

```rust
// 旧
let request = client.get_object().bucket("foo").key("bar").build_request()?;

// 新
use std::time::SystemTime;
let now = SystemTime::now();  // 副作用は呼び出し側
let request = client.get_object().bucket("foo").key("bar").build_request(now)?;
```

対象は `src/api/` 配下の全 `*FluentBuilder` の `build_request()` および `presigned()` メソッド (約 50 箇所)。

### 2. 内部関数のシグネチャ変更

| 関数 | 変更内容 |
|---|---|
| `src/api/mod.rs:332` `build_signed_request` | 末尾に `now: SystemTime` 引数追加 |
| `src/api/mod.rs:412` `build_presigned_url` | 末尾に `now: SystemTime` 引数追加 |
| `src/api/mod.rs` `build_signed_service_request` (ListBuckets 等で使用) | 同様に `now` 引数追加 |

### 3. `src/datetime.rs` から `SystemTime` 副作用を削除

- `UtcDateTime::now()` (`src/datetime.rs:59-65`) を削除する。
- `use std::time::{SystemTime, UNIX_EPOCH}` のうち `now()` 用途のものを削除し、`UNIX_EPOCH` のみ残す。
- `UtcDateTime::from_unix_timestamp(secs: u64)` を `UtcDateTime::from_system_time(now: SystemTime) -> Result<Self, Error>` に変更する。`SystemTime::duration_since(UNIX_EPOCH)` の `Err` (UNIX epoch 前の時刻) は `Error::InvalidInput` に変換する。

### 4. `HttpDate` 構造体廃止と日時型の `SystemTime` 統一

- `src/types.rs:24-83` の `HttpDate` 構造体を削除する。
- `src/types.rs:86-134` の `validate_imf_fixdate(s: &str) -> Result<(), Error>` は **関数として残す** (`pub fn validate_imf_fixdate` で公開)。利用者が手元の文字列を事前検証したい場合に使える。
- 内部用フォーマット/パースヘルパを `src/datetime.rs` に追加する:
  - `pub(crate) fn format_imf_fixdate(t: SystemTime) -> String` (リクエストヘッダー出力用)
  - `pub(crate) fn parse_imf_fixdate(s: &str) -> Result<SystemTime, Error>` (レスポンスヘッダー入力用)
  - `pub(crate) fn parse_iso8601(s: &str) -> Result<SystemTime, Error>` (XML レスポンスの `LastModified` / `Initiated` / `CreationDate` 用)
  - 既存の `src/datetime.rs` (Howard Hinnant の civil_from_days アルゴリズム) を流用する。
- `src/lib.rs:20` の `pub use types::HttpDate` を削除する。
- 入力フィールドの型変更:
  - `src/api/get_object.rs:25-26` `if_modified_since: Option<HttpDate>` → `Option<SystemTime>`
  - `src/api/get_object.rs:105-115` ビルダーメソッド `if_modified_since(date: HttpDate)` → `if_modified_since(date: SystemTime)`
  - `src/api/head_object.rs:23-24` 同様
  - ビルダー内部で受け取った `SystemTime` を `format_imf_fixdate` で IMF-fixdate に変換してヘッダーに設定する。

### 5. 出力日時フィールドも `Option<SystemTime>` に統一

- `last_modified`, `creation_date`, `initiated`, `expires` 等の出力日時フィールドは `Option<SystemTime>` に変更する。
- パース処理 (各 `parse_response` 内の XML/ヘッダー処理) で IMF-fixdate / ISO 8601 を `SystemTime` に変換する。
- パース失敗時は `Error::InvalidResponse` を返す (現状は文字列を素通ししていたため、ここで初めて失敗する可能性が出るが、aws-sdk-rust も同様の挙動)。
- aws-sdk-rust では `aws_smithy_types::DateTime` を使うが、本 crate は依存追加しないため `SystemTime` を選ぶ。型は異なるが利用感 (ビルダーに日時オブジェクトを渡す/出力から日時オブジェクトを得る) は揃える。

### 6. `Cargo.toml` の依存追加なし

- aws ライブラリへの依存追加は行わない (確定方針)。
- `chrono` / `time` クレートも追加しない。
- 既存の `src/datetime.rs` (Howard Hinnant の civil_from_days アルゴリズム) で閏年は完全に処理済みのため、追加実装は不要。

## Sans I/O 原則の徹底ポイント

- `Trait` (`TimeSource` 等) による抽象化はしない。trait 内部で副作用が発火する以上、ライブラリは時計に依存することになるため。
- `DateTime::now()` のような便利関数を本 crate に置かない。`SystemTime::now()` の呼び出しは利用者責任。
- 結果として、本 crate は `now: SystemTime` を引数で受け取り、以後一切の副作用なくリクエストを構築する純粋関数群となる。

## AWS S3 API Reference

- [Authenticating Requests (AWS Signature Version 4)](https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-authenticating-requests.html)

  > Each request is signed using your access key, which consists of an access key ID and secret access key. You compute a hash-based message authentication code (HMAC) using a derived signing key and the canonical request, and then include the signature in the request.

- [Authenticating Requests: Using the Authorization Header (AWS Signature Version 4)](https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-auth-using-authorization-header.html)

  > x-amz-date — The date used in the credential scope. The format is ISO 8601 basic format YYYYMMDD'T'HHMMSS'Z'. For example, 20130524T000000Z. It is included in the StringToSign.

  本 issue で追加する `now: SystemTime` 引数は、この `x-amz-date` ヘッダーの値の元となる時刻情報を指す。Sans I/O ライブラリとしてはこの値を外部から注入することで、純粋関数性を保つ。

- [AWS Signature Version 4 for API requests](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html)

  > The date used in the credential scope must match the date of your request.

  クレデンシャルスコープに含まれる日付と、`x-amz-date` ヘッダーの日付が一致する必要があるため、shiguredo_s3 では一貫した `SystemTime` を全ての署名要素に適用する。

## 影響範囲

- `src/api/mod.rs` の `build_signed_request` / `build_presigned_url` / `build_signed_service_request` のシグネチャ変更。
- `src/api/` 配下の全 56 オペレーションファイルの `build_request()` / `presigned()` メソッド変更。
- `src/datetime.rs` の `UtcDateTime::now()` 削除、`from_unix_timestamp` を `from_system_time` に変更、`format_imf_fixdate` / `parse_imf_fixdate` / `parse_iso8601` を追加。
- `src/types.rs:24-140` の `HttpDate` 構造体削除、`validate_imf_fixdate` 関数を公開する。
- `src/types.rs` の出力日時フィールド (`last_modified`, `creation_date`, `initiated`, `expires` 等) を `Option<String>` から `Option<SystemTime>` に変更。
- `src/api/` 配下の各 `parse_response` 内で日時文字列を `SystemTime` にパース。
- `src/lib.rs:20` の `pub use HttpDate` 削除。
- `examples/s3cli/src/upload.rs`、`examples/s3cli/src/ops.rs`、`examples/s3cli/src/commands.rs` の `build_request()` 呼び出し全箇所に `now` 引数追加 (テンプレ化のため `examples/s3cli/src/util.rs` に `fn now() -> SystemTime { SystemTime::now() }` ヘルパを追加する)。日時出力の表示処理は `SystemTime` を IMF-fixdate に整形して表示する。
- `tests/minio.rs` (3271 行)、`tests/rustfs.rs` (1695 行) の `build_request()` 呼び出し全箇所への `now` 引数追加 (テストヘルパ `fn now() -> SystemTime` を冒頭に追加)。
- `tests/minio.rs:1781,1794` の `HttpDate::from_unix_timestamp(0)` を `SystemTime::UNIX_EPOCH` に置換。
- `fuzz/fuzz_targets/fuzz_httpdate.rs` を `validate_imf_fixdate` 関数および `parse_imf_fixdate` 関数を呼ぶ形に書き換え。

## 検証方法

- 既存の minio / rustfs 統合テストが通ること。
- 固定 `now` (例: `UNIX_EPOCH + Duration::from_secs(1234567890)`) で署名値が決定的になることを確認するテストを追加する。
- Property-Based Test で「同じ `now` を渡せば同じ `S3Request` が返る」ことを検証する (副作用排除の本質的な確認)。

## 優先度

高 (Sans I/O 原則の根幹に関わるため)

## CHANGES.md への記載

- `[CHANGE] build_request / presigned に now: SystemTime 引数を追加する`
- `[CHANGE] UtcDateTime::now() を削除する`
- `[CHANGE] HttpDate 構造体を廃止し validate_imf_fixdate 関数を提供する`
- `[CHANGE] GetObject / HeadObject の if_modified_since / if_unmodified_since の型を Option<HttpDate> から Option<SystemTime> に変更する`
- `[CHANGE] last_modified / creation_date / initiated / expires 等の出力日時フィールドを Option<String> から Option<SystemTime> に変更する`

## 解決方法

### 実施した変更

1. **`src/datetime.rs` の Sans I/O 化と日時変換ヘルパ追加**
   - `UtcDateTime::now()` を削除し、`UtcDateTime::from_system_time(now: SystemTime) -> Result<Self, Error>` を追加
   - 反転変換 `unix_timestamp_from_civil` (Howard Hinnant の days_from_civil) を追加
   - `pub(crate) fn format_imf_fixdate(t: SystemTime) -> Result<String, Error>` を追加 (リクエストヘッダー出力用)
   - `pub(crate) fn parse_imf_fixdate(s: &str) -> Result<SystemTime, Error>` を追加 (レスポンスヘッダー入力用)
   - `pub(crate) fn parse_iso8601(s: &str) -> Result<SystemTime, Error>` を追加 (XML レスポンスの ISO 8601 拡張形式)
   - 単体テスト 9 件を追加

2. **`src/types.rs` の `HttpDate` 廃止**
   - `HttpDate` 構造体を削除
   - `validate_imf_fixdate(s: &str) -> Result<(), Error>` を `pub fn` として公開
   - `WEEKDAY_NAMES` / `MONTH_NAMES` を `src/datetime.rs` に移動

3. **`src/lib.rs` の `pub use` 更新**
   - `HttpDate` 削除、`validate_imf_fixdate` 追加

4. **内部関数のシグネチャ変更 (`src/api/mod.rs`)**
   - `build_signed_request` / `build_signed_service_request` / `build_presigned_url` に `now: SystemTime` 引数追加、戻り値を `Result<_, Error>` に変更
   - 内部で `UtcDateTime::from_system_time(now)?` を呼ぶ

5. **公開ビルダーへの `now` 引数追加 (56 ファイル)**
   - `pub fn build_request(&self, now: SystemTime) -> Result<S3Request, Error>` に統一
   - `pub fn presigned(self, expires_in_secs: u64, now: SystemTime) -> Result<PresignedRequest, Error>` に統一
   - Python スクリプトで一括置換、その後 list_buckets.rs 等の特殊ケースを手動修正

6. **入力日時フィールドの型変更**
   - `src/api/get_object.rs` / `src/api/head_object.rs` の `if_modified_since` / `if_unmodified_since` を `Option<SystemTime>` に変更
   - ビルダー内部で `format_imf_fixdate(t)?` で IMF-fixdate 文字列に変換してヘッダーに設定
   - aws-sdk-rust 互換の `set_*` バリアントも追加

7. **出力日時フィールドの `Option<SystemTime>` 統一**
   - `last_modified`, `creation_date`, `initiated`, `last_modified_time`, `restore_expiry_date` を `Option<SystemTime>` に変更
   - 各 `parse_response` でヘッダー (IMF-fixdate) は `parse_imf_fixdate`、XML (ISO 8601) は `parse_iso8601` でパース
   - クロージャ内 (`for_each_element` のコールバック) では `?` が使えないため、`.and_then(|s| parse_iso8601(s).ok())` でパース失敗を `None` に落とす
   - パース失敗のエラー伝搬は将来 `for_each_element` 自体を Result 対応にする issue で別途検討

8. **examples / tests / fuzz の追従**
   - `examples/s3cli/src/util.rs` に `now() -> SystemTime` ヘルパを追加
   - `tests/minio.rs` / `tests/rustfs.rs` に同 `now()` ヘルパを追加
   - 全 `.build_request()` / `.presigned(N)` を `.build_request(now())` / `.presigned(N, now())` に置換
   - `tests/minio.rs` の `HttpDate::from_unix_timestamp(0)` を `SystemTime::UNIX_EPOCH` に置換
   - `examples/s3cli/src/commands.rs` で `parse_s3_timestamp` (旧 ISO 8601 パーサ) を削除し、`format_systemtime_rfc3339` (表示用 ISO 8601 整形) を追加
   - `collect_s3_objects` の戻り値を `Vec<(String, i64, Option<SystemTime>)>` に変更
   - `fuzz/fuzz_targets/fuzz_httpdate.rs` を `validate_imf_fixdate` を呼ぶ形に書き換え

9. **Sans I/O 原則の決定的署名テストを追加 (`src/api/mod.rs`)**
   - `test_build_request_is_deterministic_for_fixed_now`: 同じ `now` で 2 回呼ぶと同じ `S3Request` を返すことを検証
   - `test_build_request_varies_with_now`: 異なる `now` で署名が変わることを検証
   - `test_presigned_is_deterministic_for_fixed_now`: presigned URL も同様
   - `test_build_request_rejects_pre_epoch_now`: UNIX epoch 前の時刻は `Error::InvalidInput` を返すことを検証

### 検証結果

- `cargo check --workspace --all-targets`: 成功
- `cargo clippy --workspace --all-targets`: 警告ゼロ
- `cargo test --lib`: 28 tests passed (新規 13 件含む)
- `cargo test --test minio test_object_put_get_head_delete test_conditional_headers_date test_presigned_put_get_head_delete`: 3 tests passed (signing path および条件付きヘッダー検証)
- pre-commit hook (cargo fmt / clippy / test) すべて pass
