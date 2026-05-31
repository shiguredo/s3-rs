# AGENTS

- Premature Optimization is the Root of All Evil
- If it hurts, do it more often
- Don't live with broken windows
- 一切妥協をしないこと
- 一切忖度しないこと
- 常に日本語を利用すること
- 全角と半角の間には半角スペースを入れること
- 絵文字を使わないこと
- コメントは全て日本語にすること
- ログメッセージは全て英語にすること
- エラーメッセージは全て英語にすること
- テストメッセージは全て日本語にすること
- コメントはしっかり入れること
- テストはコメントを重視すること

## コミットについて

- 勝手にコミットしないこと
- 全てのテストが通らない限りコミットしないこと
- コミットメッセージは確認すること
- コミットメッセージは日本語で書くこと
- コミットメッセージは命令形で書くこと
- コミットメッセージは〜するという形で書くこと
- フックをスキップしないこと

## issues について

- `0000-template.md` を参考にすること
- 番号が小さい issues から順番に対応すること
- `{seqnum}-{category}-{short-description}.md` という命名規則を守ること
  - seqnum は `issues/SEQUENCE` ファイルの値を使うこと（9999 を超えたら 5 桁にする）
  - issue を新規作成したら `issues/SEQUENCE` の値を +1 して更新すること
  - 例: `0001-bug-fix-parse-error.md`
  - 例: `0002-fmt-enhance-support-for-joins.md`
- 仕様的に対応が難しい場合は issues/pending/ へ移動すること
- issue を作成したらコミットすること
  - 複数の issue を作成する際はまとめてコミットしてもよい
- issue をコミットするときはコミットメッセージに issue の番号とタイトルを記載すること
- 1 issue 完了ごとに 1 コミットすること
- Issue の作成日はファイルのタイトルの後に `Created: YYYY-MM-DD` として記載すること
- Issue の完了日はファイルのタイトルの後に `Completed: YYYY-MM-DD` として記載すること
- Issue の優先度はファイルのタイトルの後に `Priority: <優先度>` という形で記載すること
  - 優先度は High / Medium / Low のいずれかをつけること
  - High は最優先で対応する issue、Medium は優先的に対応する issue、Low は時間があれば対応する issue という意味合いで使うこと
- Issue の磨き上げはファイルのタイトルの後に `Polished: YYYY-MM-DD` として記載すること
  - Issue を磨き上げた最終日を記載すること
  - 最新の磨き上げた日で上書きすること
- Issue を作成した LLM の Model と Version をファイルのタイトルの後に `Model: <model-name> <version>` として記載すること
  - Opus 4.7 や GPT-5.4 など
- Issue はなぜこの対応が必要なのかの根拠を明確にすること

### git ブランチの命名規則

- Git Flow を使うこと
- バグ修正は prefix を `feature/fix-` でブランチを切って対応すること
- 機能追加は prefix を `feature/add-` でブランチを切って対応すること
- 後方互換のない変更は prefix を `feature/change-` でブランチを切って対応すること
- リファクタリングは prefix を `feature/refactor-` でブランチを切って対応すること
- ブランチ名に issue の番号を含めないこと

### issue が実は解決してなかった場合

- reopen の理由を issue に書いて issues/closed から issues/ に移動すること (git mv を使うこと)
- reopen の理由は、何がどう解決していなかったのかを明確にすること

### バグが見つかった場合

- issues/ 以下にバグを markdown 形式で登録すること
- バグは再現手順を明確にすること
- できる限りの情報を

### バグを修正した場合

- issues/ 以下のバグを修正した場合は、修正内容を markdown 形式で記載すること
- issues/closed に移動すること (git mv を使うこと)
- issues/closed に移動するときは issue ファイルに「## 解決方法」セクションを追記し、何をどう修正したかを明記すること

### 設計判断が必要な issue の場合

- 外部依存の追加や設計判断が必要で保留中の issue は `issues/pending/` に置くこと
- issues/pending に移動するときは issue ファイルに pending にした理由を明記すること
- pending の issue は修正せずそのまま残す（close しない）

## テストについて

- モックやスタブは絶対に利用しないこと

## 変更履歴について

- 変更履歴は `CHANGES.md` に記載すること
- 変更の種別は以下の 4 つを使うこと
  - `[CHANGE]`: 後方互換のない変更
  - `[ADD]`: 後方互換がある追加
  - `[UPDATE]`: 後方互換がある変更
  - `[FIX]`: バグ修正
- エントリは種別の順番を守って記載すること（CHANGE → ADD → UPDATE → FIX の順）
- 機能に直接影響しない変更（ドキュメント追加、リファクタリング等）は `### misc` サブセクションに記載すること
- 未リリースの変更は `## develop` セクションに追記すること
- 各エントリは `- [種別] 変更内容を〜するという形で書く` というフォーマットにすること
- 各エントリの担当者はエントリの次の行に記載し、変更内容より 2 文字分インデントを下げて `- @ユーザー名` の形式にすること
- 担当者の行はそのエントリの最後に書くこと
- 変更内容の説明は日本語で書くこと
- リリース時は `## develop` を `## バージョン` に変更し、`**リリース日**: YYYY-MM-DD` を記載すること
- 変更履歴は派生元ブランチとの最終的な差分のみを記載すること
  - 開発ブランチ内の中間状態の修正は記載しないこと

---

## Rust

- 性能より堅牢性を優先すること
- 依存は最小限にすること
- PBT(Property-Based Testing) や Fuzzing でテストを行うこと
- 資料を由来の機能を実装する場合は、根拠資料名、節番号、将来変更される可能性があることをコードコメントで明記すること
- 入力バイナリデータをデコードする際には `Vec::with_capacity()` などのメモリを事前に割り当てるメソッドを原則として使用しないこと
  - 入力データが破損している場合などに、サイズやカウントを示す値のデコード結果が極端に大きくなり、メモリを大量に消費してしまうリスクがあるため
  - このケースでも `Vec::new()` を使っておけば、メモリ消費量のオーダーは実際の入力データのサイズから大きく乖離することはない
  - なお、性能がきわめて重要な箇所でもなければ、現実的には `Vec::with_capacity()` と `Vec::new()` の性能差は誤差程度という認識（そのため、基本的に後者を使っていればいい）
- lint 警告を抑制する必要がある時は `#[allow(...)]` ではなく `#`[expect(...)]` を使う
  - allow では、その lint 項目がなくなったり、コードの修正によって不要になった時に気づけないため
- `.unwrap()` ではなく `.expect("MESSAGE")` を使用する
  - `.unwrap()` では情報が少ない
  - `.expect("MESSAGE")` を使用して、最低限「このパニックが状況によっては発生する可能性がある」のか、それとも「絶対に発生しない想定（発生した場合は実装バグ）」なのかがメッセージから分かるようにすること

### ライブラリ

- バージョン番号はマイナーバージョンまで指定すること
  - 例: `spam = "0.3.10"` ではなく `spam = "0.3"` とする
  - 例: `egg = "1.0.1"` ではなく `egg = "1.0"` とする
- 依存ライブラリには用途をコメントで明記すること
- PBT は proptest を使うこと
- Fuzzing は cargo-fuzz を使うこと
- TLS は rustls を使う事
- 非同期処理は tokio を使うこと
- 暗号ライブラリは aws-lc-rs を使うこと
- JSON は nojson を使うこと
- 引数のパースは noargs を使うこと
- base64 は base64ct を使うこと
- ログは tracing を使うこと
  - ログのフィルタリングは tracing-subscriber を使うこと

### テストについて

- 何があってもモックやスタブを使わないこと
- `examples/whip` の WHIP シグナリング結合は実 URL（環境変数 `WHIP_ENDPOINT`）と `--features whip_client` のみとする
- pbt 以下に unittest を書かないこと
- unittest は pbt で実現できないものだけを書くこと
- 単体テストのファイル名は `tests/test_<module>.rs` とし、`src/<module>.rs` に対応させること
- PBT のファイル名は `pbt/tests/prop_<module>.rs` とし、`src/<module>.rs` に対応させること
- 特定のモジュールに対応しないテストには `test_` や `prop_` プレフィックスを付けないこと
- `#[ignore]` を使わないこと
- テストファイルが長くなった場合はファイル内で `mod` を使って分割すること
  - テストが長くなるのはモジュール自体が大きすぎるサインなので `src/<module>.rs` 側の分割を検討すること
- `src/<module>/` のようにディレクトリモジュールの場合は `pbt/tests/prop_<module>/main.rs` にサブモジュール対応で分割すること

#### テストの役割分担

- PBT: 型情報（Strategy）に基づいて入力を生成し、プロパティを検証する（ラウンドトリップ等）
- Fuzzing: 任意入力に対するクラッシュ耐性（パニック安全性）
- 単体テスト: 意図的なエラーパス、境界値など PBT で実現できないケース
- PBT でカバーできるものを単体テストで書かない

#### カバレッジ駆動のテスト作成手順

1. 対象モジュールの PBT + 単体テストのカバレッジを llvm-cov で取得する
2. 未カバー行を分類する:
   - 正常系ロジック未カバー → PBT の strategy を修正または PBT を追加する
   - エラーパス未カバー → 単体テストまたは fuzzing で対応する
   - 到達不可能なコード → デッドコードとして削除する
3. PBT に「任意入力でパニックしないことだけを検証するテスト」を書かない（fuzzing の役割）

#### カバレッジ取得コマンド例

対象モジュールに関連するテストだけを実行し、カバレッジをマージして確認する:

```bash
# 前回の計測結果をクリアする
cargo llvm-cov clean --workspace
# src/<module>.rs 内の #[cfg(test)] mod tests を実行する
cargo llvm-cov --no-report -p {crate} --lib -- <module>
# tests/test_<module>.rs の単体テストを実行する
cargo llvm-cov --no-report -p {crate} --test test_<module>
# pbt/tests/prop_<module>.rs の PBT を実行する
cargo llvm-cov --no-report -p {crate} --test prop_<module>
# 上記すべての計測結果をマージしてレポートを出力する
cargo llvm-cov report
```

---

## shiguredo_s3

- 利用者が aws-sdk-rust へ移行する際に違和感を感じないように、aws-sdk-rust スタイルの API を提供すること
- Issue には必ず AWS S3 API Reference の URL を追加すること
  - URL に加えて、該当箇所の原文を引用（`>` を使った引用ブロック）として記載すること
  - 複数の API が関係する場合はそれぞれの URL と引用を記載すること

### API 設計について

- **Amazon S3 API の仕様と aws-sdk-rust との互換性を最優先にすること**
  - API 名、メソッド名、型名、フィールド名は aws-sdk-rust に合わせること
  - 新しい API を追加する際は、必ず Amazon S3 API の公式ドキュメントと aws-sdk-rust のソースコードを確認すること
  - 推測で実装せず、仕様を確認してから実装すること
- S3 API の挙動は利用者を驚かせない（困惑させない）ために aws-sdk-rust 互換をできるだけ維持する
  - aws-sdk-rust と異なる独自の意味論を持つと、利用者が新しく覚えるコストが高くなる

### テストの禁止事項

- モックやスタブを使わないこと
- 実際の S3 互換サーバー (MinIO, RustFS 等) を使った統合テストで検証すること
