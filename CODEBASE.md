# shiguredo_s3

- 利用者が aws-sdk-rust へ移行する際に違和感を感じないように、aws-sdk-rust スタイルの API を提供すること
- Issue には必ず AWS S3 API Reference の URL を追加すること
  - URL に加えて、該当箇所の原文を引用（`>` を使った引用ブロック）として記載すること
  - 複数の API が関係する場合はそれぞれの URL と引用を記載すること

## API 設計について

- **Amazon S3 API の仕様と aws-sdk-rust との互換性を最優先にすること**
  - API 名、メソッド名、型名、フィールド名は aws-sdk-rust に合わせること
  - 新しい API を追加する際は、必ず Amazon S3 API の公式ドキュメントと aws-sdk-rust のソースコードを確認すること
  - 推測で実装せず、仕様を確認してから実装すること
- S3 API の挙動は利用者を驚かせない（困惑させない）ために aws-sdk-rust 互換をできるだけ維持する
  - aws-sdk-rust と異なる独自の意味論を持つと、利用者が新しく覚えるコストが高くなる

## テストの禁止事項

- モックやスタブを使わないこと
- 実際の S3 互換サーバー (MinIO, RustFS, kikyo-local 等) を `shiguredo_container` で起動した統合テストで検証すること（macOS: Apple container、Linux: Docker Engine）
