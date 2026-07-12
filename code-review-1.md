# コードレビューレポート: shiguredo_s3

## 1. サマリ

| 項目 | 値 |
|---|---|
| 対象ブランチ | `develop` |
| 総 Rust ファイル数 | 84 |
| 総行数 (src/) | 約 14,700 行 |
| レビュー周回数 | 5 周 |
| レビュアー総数 | 32 名 |
| 確認エージェント総数 | 18 名 |
| 読了カバレッジ | 100% (全ファイル最低 2 名以上が読了) |
| refs/ | 存在せず (観点 5 スキップ) |

**主要モジュール**: `src/api/` (56 FluentBuilder)、`src/types.rs` (1939 行)、`src/client.rs` (431 行)、`src/signing.rs` (355 行)、`src/xml.rs` (315 行)、`src/datetime.rs` (464 行)

## 2. 致命的な指摘

| ID | ファイル | 内容 | 推奨対応 |
|---|---|---|---|
| **F-1** | `src/types.rs` (1939 行) | 単体テスト・PBT・Fuzzing が一切存在しない。公開型のバリデーション (Tag キー長、ObjectIdentifier キー長、各種 Builder 必須フィールド) が全く未検証。 | `#[cfg(test)] mod tests` を追加し、全公開 Builder の必須フィールド検証と文字列長制限のテストを実装する |
| **F-2** | `src/client.rs` (431 行) | 単体テストが一切存在しない。`ConfigBuilder::build()` の必須パラメータ検証 (region, credentials) が未テスト。 | ConfigBuilder の必須パラメータ欠落時エラーとセッターの単体テストを追加する |

## 3. 重要な指摘

### 3.1 正しさ — エラーハンドリング

| ID | ファイル:行 | 問題 | 引用 | 深刻度 |
|---|---|---|---|---|
| **C1-1** | `src/xml.rs:233-234` | `parse_s3_error` が `<Code>`/`<Message>` タグ欠落時に `unwrap_or_default()` で空文字列に握り潰す。エラー情報が完全喪失する。 | `let code = extract_element(text, "Code")?.unwrap_or_default();` | **重要** |
| **C1-2** | `src/api/delete_objects.rs:156-161` | `duration_since(UNIX_EPOCH).unwrap_or(0)` が UNIX epoch 前の日時をサイレントに 1970-01-01 に変換。Conditional Delete で誤動作の可能性。 | `let secs = t.duration_since(...).map(\|d\| d.as_secs()).unwrap_or(0);` | **重要** |
| **C1-3** | `src/api/mod.rs:706-707` | `parse_s3_error_xml` 失敗時にエラー詳細を `("UnknownError", "unknown error")` に完全破棄。非 UTF-8 や不正 XML のデバッグ不能。 | `let (code, message) = parse_s3_error_xml(body).unwrap_or_else(\|_\| ("UnknownError".to_string(), ...));` | **重要** |
| **C1-4** | `src/api/mod.rs:631-638` | `check_body_error()` が非 UTF-8 レスポンスボディを `unwrap_or("")` で空文字列に変換。非 UTF-8 エラーレスポンスを見逃す。 | `let text = std::str::from_utf8(&response.body).unwrap_or("");` | **重要** |
| **C1-5** | `src/api/put_object.rs:451` | `body` 未設定時に空ボディとして暗黙送信。aws-sdk-rust では `PutObject::body()` は必須。API 互換性の契約違反。 | `let body = self.body.as_deref().unwrap_or_default();` | **重要** |
| **C1-6** | `src/api/upload_part.rs:275` | C1-5 と同様。body 未設定時に空パートが暗黙送信される。 | 同パターン | **重要** |
| **C1-7** | `src/api/put_object.rs:66` | `content_length: Option<i64>` が負の値を受け付ける。`content-length: -1` が生成される可能性。 | `content_length: Option<i64>,` | **重要** |

### 3.2 設計 — コード重複

| ID | 対象 | 重複規模 | 深刻度 |
|---|---|---|---|
| **C2-1** | `extract_xml_tags` | 2 ファイルに完全同一実装 | **重要** |
| **C2-2** | `extract_xml_common_prefixes` | 3 ファイルに完全同一実装 | **重要** |
| **C2-3** | `build_tagging_xml` | 2 ファイルに完全同一実装 | **重要** |
| **C2-4** | `expected_bucket_owner` パターン | 6 ファイルに同一パターン | **重要** |
| **C2-5** | `content_md5` + `application/xml` ヘッダー構築 | 13 ファイルにほぼ同一 | **重要** |
| **C2-6** | SSE-C キー→MD5 計算ブロック | 7 ファイル 14 箇所 | **重要** |
| **C2-7** | `build_request` と `presigned` のヘッダー構築重複 | 8 ファイル、推定 375 行 | **重要** |
| **C2-8** | `parse_response` 内の `response.get_header(...).map(String::from)` パターン | ~100 箇所 | **重要** |

### 3.3 設計 — 肥大化

| ID | ファイル:行 | 問題 | 深刻度 |
|---|---|---|---|
| **C2-9** | `src/api/get_bucket_lifecycle_configuration.rs:67-576` | `extract_lifecycle_rules` が単一関数 510 行。Context 10 種、状態変数 30 個超。 | **重要** |
| **C2-10** | `src/api/get_bucket_notification_configuration.rs:66-294` | `parse_notification_configuration` が 225 行の単一関数。 | **重要** |
| **C2-11** | `src/types.rs:1568-1908` | 8 つの公開 enum に `#[non_exhaustive]`。shiguredo-rust スキル違反。`Unknown(String)` variant と二重防御。 | **重要** |

### 3.4 規約整合性

| ID | ファイル | 問題 | 深刻度 |
|---|---|---|---|
| **C3-1** | `src/api/get_bucket_website.rs:85-95` | 英語セクションマーカーコメント (`// IndexDocument` 等) | **重要** |
| **C3-2** | `src/api/put_bucket_lifecycle_configuration.rs:126-231` | 英語セクションマーカーコメント (`// Filter` 等) | **重要** |
| **C3-3** | `src/api/upload_part_copy.rs:3` | モジュール doc コメントに文字化け | **重要** |
| **C3-4** | `src/datetime.rs:334-370` | テスト `.expect()` メッセージが英語 (`"epoch"` 等) | **重要** |
| **C3-5** | `fuzz/fuzz_targets/fuzz_httpdate.rs:9` | `issue 0060` への言及 | **重要** |

### 3.5 テスト戦略

| ID | 内容 | 深刻度 |
|---|---|---|
| **C4-1** | `fuzz_target_1.rs` が空テンプレートでデッドコード | **重要** |
| **C4-2** | PBT が `datetime` 1 件のみ。checksum, signing, xml に未追加 | **重要** |
| **C4-3** | 14 FluentBuilder API (25%) が統合テスト未カバー | **重要** |
| **C4-4** | `tests/minio.rs` 3337 行、`tests/rustfs.rs` 1743 行。`mod` 分割なし | **重要** |
| **C4-5** | `tests/helpers/` 不在。共通ヘルパーが minio.rs / rustfs.rs で重複 | **重要** |
| **C4-6** | 18 の XML パース API に破損 XML の単体エラーテストなし | **重要** |

## 4. 改善提案

| ID | 内容 |
|---|---|
| P-1 | `parse::<bool>().ok()` パターンが 15 ファイル以上でエラーを握り潰している |
| P-2 | Tag Key/Value XML 欠落時に無言でスキップ (`get_bucket_tagging.rs` 他) |
| P-3 | `put_object_legal_hold.rs` の `legal_hold_status` に任意文字列を渡せる |
| P-4 | `put_object_retention.rs` の `mode` に任意文字列を渡せる |
| P-5 | `required()` が空白のみ文字列 (`"   "`) を許可 |
| P-6 | `examples/s3cli/Cargo.toml` の tokio バージョンが `"1"` (メジャーのみ指定) |
| P-7 | 全 55 API ファイル先頭の `//! {API名} API` が英語 |
| P-8 | `src/checksum.rs` のテストで `.unwrap()` 多用 (メッセージなし) |
| P-9 | `tests/` 内に `.unwrap()` 60+ 箇所 (`.expect()` に置換すべき) |
| P-10 | `CHANGES.md` misc セクションのエントリに `[種別]` プレフィックスなし |
| P-11 | `parse_port_from_authority` がポートパースエラーを `.ok()` で握り潰し |
| P-12 | `src/types.rs` 8 enum の `as_str()` と `Display` 実装が doc comment 欠落 |
| P-13 | `Credentials` の `session_token`/`expires_after`/`provider_name` フィールドが不要な `pub(crate)` |
| P-14 | `parse_response` 内 `response.get_header(...).map(String::from)` が ~100 箇所で冗長 |

## 5. 削除候補

| ID | ファイル | 内容 |
|---|---|---|
| D-1 | `fuzz/fuzz_targets/fuzz_target_1.rs` | cargo-fuzz テンプレートのまま空実装。`[[bin]]` 未登録。完全なデッドコード。**削除推奨**。 |
| D-2 | `fuzz/fuzz_targets/fuzz_httpdate.rs:9` | `(issue 0060)` 参照。理由そのものに置換し issue 参照を削除。 |
| D-3 | `src/types.rs` 8 enum | `#[non_exhaustive]` 属性。`Unknown(String)` variant と重複。 |
| D-4 | `src/api/put_object.rs:460-461` | リファクタリング時の内部開発ノート。利用者には不要。 |

## 6. モジュール別評価

| モジュール | 行数 | 品質 | 主な問題 |
|---|---|---|---|
| `src/types.rs` | 1939 | 🔴 低 | テストゼロ、`#[non_exhaustive]` 違反、肥大化。**最優先改善対象**。 |
| `src/client.rs` | 431 | 🟡 中 | テストゼロ。コード品質は良好。 |
| `src/xml.rs` | 315 | 🟡 中 | `parse_s3_error` エラー喪失、`XmlWriter` expect パニック。 |
| `src/api/mod.rs` | 884 | 🟡 中 | 複数のエラー握り潰し、肥大化。 |
| `src/api/put_object.rs` | 786 | 🟡 中 | body 暗黙空送信、`content_length` 負数許容、責務過剰。 |
| `src/api/get_bucket_lifecycle_configuration.rs` | 577 | 🟡 中 | 510 行の単一関数。 |
| `src/api/*.rs` (56 ファイル) | ~10,900 | 🟡 中 | 広範なコード重複。共通化の余地が大きい。 |
| `src/datetime.rs` | 464 | 🟢 高 | PBT あり。英語テストメッセージのみ。 |
| `src/signing.rs` | 355 | 🟢 高 | PBT 不在だがコード品質良好。 |
| `src/checksum.rs` | 164 | 🟢 高 | PBT 不在、`.unwrap()` 多用。 |
| `tests/` | ~5,600 | 🟡 中 | 過大ファイル、`test_` プレフィックスなし、ヘルパー重複。 |
| `fuzz/` | ~76 | 🟡 中 | 1 ファイルが完全デッドコード。5 関数未カバー。 |
| `pbt/` | 29 | 🔴 低 | 1 件のみ。checksum, signing, xml への PBT 追加が急務。 |

## 7. 検証統計

| 指標 | 値 |
|---|---|
| 生成された指摘票の総数 | ~120 件 |
| 機械照合で棄却 | 1 件 (引用不一致) |
| 独立追認で棄却 (誤り) | 1 件 (G1: CloudFunctionConfiguration) |
| 独立追認で降格 | 1 件 (finish() UTF-8 expect) |
| 冗長割当で片方のみ発見 | 各観点で 20〜40% の指摘が片方のみ |
| 周別新規重要指摘数 | 第1周: 8, 第2周: 6, 第3周: 5, 第4周: 4, 第5周: 0 |

周回を重ねるごとに新規指摘が減少し、第 5 周でゼロになったことから、主要な問題はほぼ網羅できたと判断する。

## 8. 残った懸念

- **第 3〜4 周の指摘は独立追認未了**: これらの指摘は機械照合のみ通過した状態。特に `content_length` 負数許容 (C1-7) と `check_body_error` UTF-8 握り潰し (C1-4) は要確認。
- **Sans I/O の限界**: ライブラリ本体には I/O がなく、実際の HTTP 通信での問題 (タイムアウト、リダイレクト、再接続) は検出できない。統合テスト (minio.rs, rustfs.rs) がこの役割を担うが、14 API が未カバー。

## 9. 推奨アクション (優先度順)

1. **types.rs テスト追加** (致命) — 公開 Builder のバリデーションテスト
2. **client.rs テスト追加** (致命) — ConfigBuilder の必須パラメータ検証テスト
3. **body 必須化** (重要) — put_object/upload_part で body 未設定時にエラー返却
4. **`parse_s3_error` 修正** (重要) — Code/Message 欠落時のエラー情報保持
5. **`delete_objects.rs` unwrap_or(0) 修正** (重要) — エラー伝播に変更
6. **`#[non_exhaustive]` 除去** (重要) — 8 enum から non_exhaustive を削除
7. **コード重複解消** (重要) — extract_xml_tags, extract_xml_common_prefixes, build_tagging_xml, expected_bucket_owner, content_md5, SSE-C MD5, build_request/presigned 重複の共通化
8. **PBT 拡充** (重要) — checksum, signing, xml, types に PBT を追加
9. **fuzz_target_1.rs 削除** (改善) — 完全なデッドコードの除去
10. **テストメッセージ日本語化** (改善) — datetime.rs, signing.rs, checksum.rs 他
11. **英語コメント修正** (改善) — get_bucket_website.rs, put_bucket_lifecycle_configuration.rs 他
12. **upload_part_copy.rs 文字化け修正** (重要)
13. **未カバー 14 API の統合テスト追加** (重要)
