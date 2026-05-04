// -------------------------------------------------------
// マルチパートアップロード
// -------------------------------------------------------

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use shiguredo_s3::Client;
use shiguredo_s3::types::{CompletedMultipartUpload, CompletedPart, PutObjectOutput};

use crate::params::UploadParams;
use crate::transport::{ConnectionPool, execute_pooled, send, send_pooled};

/// S3 パートの最小サイズ (5 MB)
pub(crate) const MIN_PART_SIZE: usize = 5 * 1024 * 1024;
/// デフォルトパートサイズ (16 MB)
pub(crate) const DEFAULT_PART_SIZE: usize = 16 * 1024 * 1024;
/// デフォルト並行数
pub(crate) const DEFAULT_CONCURRENCY: usize = 12;

/// データサイズから最適なパートサイズを計算する
///
/// S3 の最大パート数 10000 に収まるよう調整する
pub(crate) fn calculate_part_size(data_size: usize) -> usize {
    let min_from_count = data_size / 10000 + 1;
    min_from_count.max(MIN_PART_SIZE).max(DEFAULT_PART_SIZE)
}

/// アップロード元データ
pub(crate) enum UploadData<'a> {
    /// メモリ上のデータ (stdin 等)
    Memory(&'a [u8]),
    /// ファイルパスとサイズ (ストリーミング読み込み)
    File { path: &'a str, size: u64 },
}

impl UploadData<'_> {
    pub(crate) fn len(&self) -> usize {
        match self {
            UploadData::Memory(data) => data.len(),
            UploadData::File { size, .. } => *size as usize,
        }
    }
}

/// マルチパートアップロードのパラメータ
pub(crate) struct MultipartUploadParams<'a> {
    pub(crate) client: &'a Client,
    pub(crate) tls_config: &'a Arc<rustls::ClientConfig>,
    pub(crate) bucket: &'a str,
    pub(crate) key: &'a str,
    pub(crate) data: UploadData<'a>,
    pub(crate) content_type: Option<&'a str>,
    pub(crate) part_size: usize,
    pub(crate) concurrency: usize,
    pub(crate) sse: &'a UploadParams,
}

/// マルチパートアップロードでオブジェクトをアップロードする
///
/// データがパートサイズ以下の場合は通常の PutObject を使用する。
/// エラー時は AbortMultipartUpload で中止する。
/// 接続プールにより TLS ハンドシェイクのコストを削減する。
pub(crate) async fn upload_multipart(
    params: &MultipartUploadParams<'_>,
) -> Result<PutObjectOutput, Box<dyn std::error::Error + Send + Sync>> {
    let MultipartUploadParams {
        client,
        tls_config,
        bucket,
        key,
        data,
        content_type,
        part_size,
        concurrency,
        ..
    } = params;
    let part_size = *part_size;
    let concurrency = *concurrency;

    if part_size < MIN_PART_SIZE {
        return Err("part_size must be >= 5 MB".into());
    }
    if concurrency < 1 {
        return Err("concurrency must be >= 1".into());
    }

    let sse = params.sse;
    let data_len = data.len();

    // パートサイズ以下の場合は通常の PutObject を使用する
    if data_len <= part_size {
        let body = match data {
            UploadData::Memory(bytes) => bytes.to_vec(),
            UploadData::File { path, .. } => tokio::fs::read(path).await?,
        };
        let mut builder = client.put_object().bucket(*bucket).key(*key).body(body);
        if let Some(ct) = content_type {
            builder = builder.content_type(*ct);
        }
        builder = sse.apply_to_put(builder);
        let request = builder.build_request()?;
        return send(
            tls_config,
            request,
            shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
        )
        .await;
    }

    // マルチパートアップロードを開始する
    let mut create_builder = client.create_multipart_upload().bucket(*bucket).key(*key);
    if let Some(ct) = content_type {
        create_builder = create_builder.content_type(*ct);
    }
    create_builder = sse.apply_to_create_multipart(create_builder);
    let create_request = create_builder.build_request()?;

    // リクエストからホスト情報を取得して接続プールを構築する
    let pool = Arc::new(ConnectionPool::new(
        tls_config,
        &create_request.host,
        create_request.port,
        create_request.https,
    ));

    let create_output = send_pooled(
        &pool,
        create_request,
        shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
    )
    .await?;
    let upload_id = create_output
        .upload_id
        .ok_or("missing upload_id in CreateMultipartUpload response")?;

    // パートを並行アップロードする
    match upload_parts_concurrent(params, &upload_id, &pool).await {
        Ok(completed_parts) => {
            let complete_request = client
                .complete_multipart_upload()
                .bucket(*bucket)
                .key(*key)
                .upload_id(&upload_id)
                .multipart_upload(CompletedMultipartUpload {
                    parts: Some(completed_parts),
                })
                .build_request()?;
            let output = send_pooled(
                &pool,
                complete_request,
                shiguredo_s3::api::CompleteMultipartUploadFluentBuilder::parse_response,
            )
            .await?;
            Ok(PutObjectOutput {
                e_tag: output.e_tag,
                version_id: output.version_id,
            })
        }
        Err(e) => {
            // アップロード失敗時は中止する
            if let Ok(abort_request) = client
                .abort_multipart_upload()
                .bucket(*bucket)
                .key(*key)
                .upload_id(&upload_id)
                .build_request()
                && let Err(abort_err) = execute_pooled(&pool, abort_request).await
            {
                eprintln!("warning: failed to abort multipart upload: {abort_err}");
            }
            Err(e)
        }
    }
}

/// パートを並行アップロードする
///
/// 接続プールを使用して TLS 接続を再利用する。
/// UploadData::File の場合は各タスクがファイルから直接チャンクを読み込む。
async fn upload_parts_concurrent(
    params: &MultipartUploadParams<'_>,
    upload_id: &str,
    pool: &Arc<ConnectionPool>,
) -> Result<Vec<CompletedPart>, Box<dyn std::error::Error + Send + Sync>> {
    let semaphore = Arc::new(Semaphore::new(params.concurrency));
    let mut join_set = JoinSet::new();

    let data_len = params.data.len();
    let num_parts = data_len.div_ceil(params.part_size);

    for i in 0..num_parts {
        let sem = semaphore.clone();
        let pool = pool.clone();
        let client = params.client.clone();
        let bucket = params.bucket.to_string();
        let key = params.key.to_string();
        let upload_id = upload_id.to_string();
        let part_number = (i as i32) + 1;
        let sse = params.sse.clone();
        let part_size = params.part_size;
        let offset = i * part_size;

        match &params.data {
            UploadData::Memory(body) => {
                let end = (offset + part_size).min(body.len());
                let chunk = body[offset..end].to_vec();
                join_set.spawn(async move {
                    let _permit = sem
                        .acquire()
                        .await
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
                    upload_single_part(
                        &pool,
                        &client,
                        &bucket,
                        &key,
                        &upload_id,
                        part_number,
                        chunk,
                        &sse,
                    )
                    .await
                });
            }
            UploadData::File { path, size } => {
                let file_path = path.to_string();
                let file_size = *size as usize;
                join_set.spawn(async move {
                    let _permit = sem
                        .acquire()
                        .await
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
                    // セマフォ取得後にファイルからチャンクを読み込む
                    // (同時にメモリ上に存在するチャンク数を並行数に制限する)
                    let read_size = part_size.min(file_size - offset);
                    let mut chunk = vec![0u8; read_size];
                    let mut file = tokio::fs::File::open(&file_path).await?;
                    file.seek(std::io::SeekFrom::Start(offset as u64)).await?;
                    file.read_exact(&mut chunk).await?;
                    upload_single_part(
                        &pool,
                        &client,
                        &bucket,
                        &key,
                        &upload_id,
                        part_number,
                        chunk,
                        &sse,
                    )
                    .await
                });
            }
        }
    }

    let mut completed_parts = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(part)) => completed_parts.push(part),
            Ok(Err(e)) => {
                join_set.abort_all();
                return Err(e);
            }
            Err(join_err) => {
                join_set.abort_all();
                return Err(join_err.into());
            }
        }
    }

    // パート番号でソートする
    completed_parts.sort_by_key(|p| p.part_number);

    Ok(completed_parts)
}

/// 単一パートをアップロードする
#[allow(clippy::too_many_arguments)]
async fn upload_single_part(
    pool: &ConnectionPool,
    client: &Client,
    bucket: &str,
    key: &str,
    upload_id: &str,
    part_number: i32,
    chunk: Vec<u8>,
    sse: &UploadParams,
) -> Result<CompletedPart, Box<dyn std::error::Error + Send + Sync>> {
    let builder = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .part_number(part_number)
        .body(chunk);
    let request = sse.apply_to_upload_part(builder).build_request()?;
    let response = execute_pooled(pool, request).await?;
    let output = shiguredo_s3::api::UploadPartFluentBuilder::parse_response(&response)?;
    Ok(CompletedPart {
        e_tag: output.e_tag,
        part_number: Some(part_number),
    })
}
