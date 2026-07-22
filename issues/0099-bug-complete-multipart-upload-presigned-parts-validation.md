# CompleteMultipartUpload の presigned が parts 検証をスキップする

- Priority: Medium
- Created: 2026-07-09
- Model: Grok 4.5
- Polished: 2026-07-23
- Branch: feature/fix-complete-multipart-upload-presigned-parts-validation

## AWS S3 API Reference

- <https://docs.aws.amazon.com/AmazonS3/latest/API/API_CompleteMultipartUpload.html>

> In the CompleteMultipartUpload request, you must provide the parts list and ensure that the parts list is complete. ... For each part in the list, you must provide the PartNumber value and the ETag value that are returned after that part was uploaded.

Special errors より:

> **Error Code: InvalidPartOrder** — The list of parts was not in ascending order. The parts list must be specified in order by part number.

## 目的

`CompleteMultipartUploadFluentBuilder::presigned` が `build_request` と同じ parts 入力検証を行い、不正な completed parts で署名付きリクエストを発行しないようにする。

## 優先度根拠

- `build_request` と `presigned` の非対称は、利用者が同じ Fluent Builder から片方だけ成功しうる混乱を生む
- 不正 XML を署名付きで発行でき、サーバ到達後に 400 になるまで気づきにくい
- HeadObject の presigned 非対称は既に修正方針が立っており、同種の抜けを閉じる

## 現状

`build_request` は parts を検証する:

```98:116:src/api/complete_multipart_upload.rs
        if let Some(ref upload) = self.multipart_upload
            && let Some(ref parts) = upload.parts
        {
            let mut prev_part_number = 0i32;
            for (i, part) in parts.iter().enumerate() {
                let pn = part.part_number.ok_or_else(|| {
                    Error::InvalidInput(format!("part[{i}] is missing part_number"))
                })?;
                if part.e_tag.is_none() {
                    return Err(Error::InvalidInput(format!("part[{i}] is missing e_tag")));
                }
                if pn <= prev_part_number {
                    return Err(Error::InvalidInput(
                        "parts must be in ascending order of part_number".to_string(),
                    ));
                }
                prev_part_number = pn;
            }
        }
```

`presigned` は検証なしで XML を生成する:

```211:220:src/api/complete_multipart_upload.rs
    pub fn presigned(
        self,
        expires_in_secs: u64,
        now: std::time::SystemTime,
    ) -> Result<super::PresignedRequest, Error> {
        validate_presign_expires(expires_in_secs)?;
        let bucket = required(self.bucket.as_deref(), "bucket")?;
        let key = required(self.key.as_deref(), "key")?;
        let upload_id = required(self.upload_id.as_deref(), "upload_id")?;
        let xml_body = build_complete_multipart_xml(&self.multipart_upload)?;
```

`build_complete_multipart_xml` は `part_number` / `e_tag` が `None` の場合、該当要素を省略した `<Part>` を書きうる。

## 設計方針

- `build_request` 内の parts 検証ロジック（`part_number` 必須チェック、`e_tag` 必須チェック、昇順チェック）を `src/api/complete_multipart_upload.rs` 内のプライベート関数 `validate_completed_parts` に切り出す
- `build_request` と `presigned` の両方から `validate_completed_parts(self.multipart_upload.as_ref())?;` を呼ぶ
- `validate_completed_parts` は `&Option<CompletedMultipartUpload>` を受け取り、`None` または `parts` が空の場合は何もせず `Ok(())` を返す
- `part_number` の値域検証 (`1..=10000`) は既存の `validate_part_number` を流用する。ただし `validate_part_number` の追加呼び出しは `build_request` でも現状行われていないため、本 issue で併せて導入するかどうかは実装時に判断する
- issue 0101（`multipart_upload` 必須化）と同一ファイルを対象とする。0101 を先に実装すれば `validate_completed_parts` は `None` を扱う必要がなくなるため、0101 → 0099 の順で実装することを推奨する

## 完了条件

- `validate_completed_parts` 関数が抽出され、`build_request` と `presigned` の両方から呼ばれること
- `presigned` で `part_number` 欠落・`e_tag` 欠落・非昇順の parts を指定した場合に `Error::InvalidInput` が返ること
- `tests/test_complete_multipart_upload.rs` に parts 検証のエラーパステストを追加すること（欠落 part_number / 欠落 e_tag / 非昇順の 3 ケース）
- `CHANGES.md` の `## develop` に `[FIX]` エントリを記載すること

## 解決方法

1. `build_request` 内の parts 検証ロジック（`part_number` 必須・`e_tag` 必須・昇順チェック）をプライベート関数 `validate_completed_parts` に切り出す
2. `build_request` と `presigned` の両方から `validate_completed_parts` を呼ぶ
3. `tests/test_complete_multipart_upload.rs` に parts 検証のエラーパステストを追加する（欠落 part_number / 欠落 e_tag / 非昇順の 3 ケース）
4. `CHANGES.md` の `## develop` に `[FIX]` エントリを追加する
