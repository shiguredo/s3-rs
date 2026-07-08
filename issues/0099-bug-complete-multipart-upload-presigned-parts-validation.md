# CompleteMultipartUpload の presigned が parts 検証をスキップする

- Priority: Medium
- Created: 2026-07-09
- Model: Grok 4.5
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

- parts 検証ロジックを共通関数（例: `validate_completed_parts`）に切り出す
- `build_request` と `presigned` の両方から呼ぶ
- 既存の必須・昇順チェックを維持する（`part_number` の 1..=10000 は UploadPart 側と揃えるかは実装時に判断し、揃えるなら `validate_part_number` を再利用）

## 完了条件

- `presigned` が `build_request` と同じ parts 検証で `Error::InvalidInput` を返すこと
- 単体テストで欠落 part_number / e_tag / 非昇順が `presigned` でも拒否されること
- `CHANGES.md` の `## develop` に `[FIX]` を記載すること
