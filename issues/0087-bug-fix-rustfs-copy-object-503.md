# RustFS 統合テスト test_copy_object が 503 エラーで失敗する

- Priority: Medium
- Created: 2026-05-25
- Model: Opus 4.7 1M
- Branch: feature/fix-rustfs-copy-object-503

## 目的

develop ブランチの CI が RustFS 統合テストの `test_copy_object` 失敗で赤くなっている状態を解消し、CI を安定して通る状態に戻す。

## 優先度根拠

CI がスケジュール実行で毎日失敗し続けるため早めに対応すべき。ただし MinIO テストは全て通っており shiguredo_s3 ライブラリ自体のバグではなく、テスト基盤の問題である。

## 現状

- GitHub Actions run: https://github.com/shiguredo/s3-rs/actions/runs/26381355217
- `cargo test --test rustfs` の `test_copy_object` が以下のエラーで失敗する:

```
thread 'test_copy_object' panicked at tests/rustfs.rs
failed to parse response: S3 { status_code: 503, code: "UnknownError", message: "unknown error" }
```

- 失敗箇所: `test_copy_object` テスト内で CopyObject リクエストを `send()` 経由で実行した際、`CopyObjectFluentBuilder::parse_response` が `is_success()` チェックで 503 を検出し `Error::S3` を返す
- 同じ CopyObject API は MinIO テスト (`tests/minio.rs`) では正常に通っている
- 前回の CI 成功 (2026-05-24, run 26366337613) では RustFS でも `test_copy_object` が通っていた
- Docker Hub 上の `rustfs/rustfs:latest` は `1.0.0-beta.4` (2026-05-21 リリース) を指しており、5/24〜5/25 で Docker Hub のタグ自体は更新されていない

### 重要な切り分け事実

- `test_copy_object` の内部で CopyObject の前に実行される CreateBucket と PutObject は成功している → API 全般の初期化遅延ではなく **CopyObject 固有のコードパス** に問題がある可能性が高い
- Docker Hub のタグが 5/21 以降更新されていないのに 5/24 は pass、5/25 は fail → 問題は **間欠的** であり決定論的バグではない可能性がある（GitHub Actions の Docker キャッシュにより 5/24 と 5/25 で異なるレイヤーが使われた可能性も排除できない）

## 原因の仮説

蓋然性の高い順に記載する:

1. **CopyObject 固有の間欠バグ**: `1.0.0-beta.4` の CopyObject 実装に間欠的な 503 を返すバグがある（PutObject が同一テスト内で成功している事実がこの仮説を支持する。間欠的であるため 5/24 は偶然 pass した可能性がある）
2. **CopyObject の初期化遅延**: RustFS の `/health` エンドポイントは「プロセスが生きている」レベルの応答であり、CopyObject のような内部的にオブジェクト読み取り + 書き込みを伴う API は追加の初期化が必要な可能性がある（MinIO は `WaitFor::message_on_either_std("API:")` で S3 API 起動完了を直接検知しており、この問題が起きない）
3. **CI 環境依存**: GitHub Actions のリソース制約により、RustFS コンテナの CopyObject 処理がタイムアウトする

## 設計方針

### 着手前の調査（必須）

1. 5/24 の CI run (26366337613) のログで RustFS step が実際に pass していること、および `docker pull` の出力からイメージダイジェストを確認する（testcontainers のログに `Pulling image ...` の出力がある）
2. ローカルで `rustfs/rustfs:1.0.0-beta.4` を明示的に pull し、`cargo test --test rustfs test_copy_object` を 10 回実行する。**1 回でも失敗したら「再現する」と判定する**
3. 再現する場合、まず `rustfs/rustfs:1.0.0-beta.3` で同テストを 10 回実行し、安定するか確認する（仮説 1 の検証を先に行う。beta.3 で安定し beta.4 で再現するなら仮説 1 確定）
4. beta.3 でも再現する場合、`start_rustfs()` の `WaitFor::seconds(2)` を `WaitFor::seconds(5)` に延ばして 10 回実行する（仮説 2 の検証）
5. 手順 2 でローカルで 10 回全て成功する場合、仮説 3（CI 環境依存）として本 issue は pending に移動し、別 issue を起票する

### 調査結果に応じた対応

- **仮説 1 確定（`1.0.0-beta.3` で 10 回成功、`1.0.0-beta.4` で再現）の場合**: イメージタグを `1.0.0-beta.3` に固定する。issue 0074 では「`latest` タグの追従は維持する」と判断したが、beta.4 で間欠的���失敗する��題が確認された場合は CI 安定性を優先して方針を変更する。RustFS 側に issue を報告し、修正後に latest へ戻す
- **仮説 2 確定（beta.3 でも再現するが待機時間延長で安定）の場合**: `start_rustfs()` の `WaitFor::seconds(2)` を `WaitFor::seconds(5)` に変更する（10 回成功する最小値 + マージンとして 5 秒を採用。18 テスト x 3 秒増 = 最大 54 秒の CI 所要時間増加はトレードオフとして許容する）
- **仮説 3（ローカルで再現しない）の場合**: 本 issue を pending に移動し、CI 環境依存の問題として別 issue を起票する

## 変更対象ファイル

- `tests/rustfs.rs`:
  - `start_rustfs()` 関数内の `GenericImage::new("rustfs/rustfs", "latest")` のタグ変更（仮説 1 の場合）
  - `start_rustfs()` 関数内の `WaitFor::seconds(2)` の値変更（仮説 2 の場合）
  - ファイル先頭の「既知の不具合 (RustFS 0.0.5)」セクション: バージョン表記を現行バージョンに更新し、仮説 1 が確定した場合は CopyObject の 503 を追記する（バージョン表記の更新は対応仮説に関わらず実施する。既存の 2 件の不具合が現行バージョンでも再現するか確認し、解消済みのものは削除する）
- `CHANGES.md`: `### misc` セクションに変更内容を記載する

## 完了条件

- ローカルで `cargo test --test rustfs test_copy_object` が 10 回連続で pass すること
- CI の RustFS 統合テスト全 18 件が pass すること
- `tests/rustfs.rs` 先頭の「既知の不具合」セクションのバージョン表記が現行バージョンに更新されていること
- 仮説 1 が確定した場合、「既知の不具合」セクションに CopyObject の 503 が追記されていること
- `CHANGES.md` の `### misc` セクションに変更内容が記載されていること
- 原因と対策の根拠がコミットメッセージに記載されていること

## 再現手順

1. ローカルの RustFS イメージを最新にする: `docker pull rustfs/rustfs:1.0.0-beta.4`
2. `cargo test --test rustfs test_copy_object` を実行する
3. 503 エラーでテストが失敗することを確認する（再現しない場合は 10 回試行する）

## 補足

本 issue は AWS S3 API 仕様ではなく RustFS テスト基盤の問題のため、AWS S3 API Reference の URL は記載しない。
