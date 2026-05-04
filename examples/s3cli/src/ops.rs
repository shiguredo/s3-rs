// -------------------------------------------------------
// ファイル操作
// -------------------------------------------------------

use std::sync::Arc;

use shiguredo_s3::Client;
use shiguredo_s3::types::ObjectIdentifier;

use crate::params::UploadParams;
use crate::transport::send;
use crate::upload::{
    DEFAULT_CONCURRENCY, MultipartUploadParams, UploadData, calculate_part_size, upload_multipart,
};
use crate::util::{FilterRule, resolve_content_type, should_include};

/// 単一ファイルをアップロードする
#[allow(clippy::too_many_arguments)]
pub(crate) async fn upload_file(
    client: &Client,
    tls_config: &Arc<rustls::ClientConfig>,
    local_path: &str,
    bucket: &str,
    key: &str,
    content_type: Option<&str>,
    sse: &UploadParams,
    quiet: bool,
    dryrun: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if dryrun {
        eprintln!("(dryrun) upload: {local_path} to s3://{bucket}/{key}");
        return Ok(());
    }

    let file_size = tokio::fs::metadata(local_path).await?.len();
    let part_size = calculate_part_size(file_size as usize);

    let params = MultipartUploadParams {
        client,
        tls_config,
        bucket,
        key,
        data: UploadData::File {
            path: local_path,
            size: file_size,
        },
        content_type,
        part_size,
        concurrency: DEFAULT_CONCURRENCY,
        sse,
    };
    upload_multipart(&params).await?;

    if !quiet {
        eprintln!("upload: {local_path} -> s3://{bucket}/{key}");
    }
    Ok(())
}

/// 単一ファイルをダウンロードする
#[allow(clippy::too_many_arguments)]
pub(crate) async fn download_file(
    client: &Client,
    tls_config: &Arc<rustls::ClientConfig>,
    bucket: &str,
    key: &str,
    local_path: &str,
    sse: &UploadParams,
    quiet: bool,
    dryrun: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if dryrun {
        eprintln!("(dryrun) download: s3://{bucket}/{key} to {local_path}");
        return Ok(());
    }

    let request = sse
        .apply_to_get(client.get_object().bucket(bucket).key(key))
        .build_request()?;
    let output = send(
        tls_config,
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await?;

    // ディレクトリなら key のファイル名部分を付与する
    let path = if std::path::Path::new(local_path).is_dir() {
        let filename = key.rsplit('/').next().unwrap_or(key);
        format!("{local_path}/{filename}")
    } else {
        local_path.to_string()
    };

    if let Some(parent) = std::path::Path::new(&path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(&path, &output.body).await?;
    if !quiet {
        eprintln!("download: s3://{bucket}/{key} -> {path}");
    }
    Ok(())
}

/// 再帰アップロードのパラメータ
pub(crate) struct RecursiveUploadParams<'a> {
    pub(crate) client: &'a Client,
    pub(crate) tls_config: &'a Arc<rustls::ClientConfig>,
    pub(crate) local_dir: &'a str,
    pub(crate) bucket: &'a str,
    pub(crate) prefix: &'a str,
    pub(crate) content_type: Option<&'a str>,
    pub(crate) no_guess_mime_type: bool,
    pub(crate) follow_symlinks: bool,
    pub(crate) sse: &'a UploadParams,
    pub(crate) filters: &'a [FilterRule],
    pub(crate) quiet: bool,
    pub(crate) dryrun: bool,
}

/// 再帰的にアップロードする
pub(crate) async fn upload_recursive(
    params: &RecursiveUploadParams<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stack = vec![std::path::PathBuf::from(params.local_dir)];
    let base = std::path::Path::new(params.local_dir);

    while let Some(dir) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            // シンボリックリンクの処理
            if path.is_symlink() && !params.follow_symlinks {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else {
                let relative = path.strip_prefix(base).unwrap_or(&path);
                let relative_str = relative.to_string_lossy();
                // フィルタで除外されたファイルはスキップする
                if !should_include(&relative_str, params.filters) {
                    continue;
                }
                let key = if params.prefix.is_empty() {
                    relative_str.to_string()
                } else {
                    let prefix = params.prefix.trim_end_matches('/');
                    format!("{prefix}/{relative_str}")
                };
                let path_str = path.to_string_lossy();
                let ct =
                    resolve_content_type(params.content_type, &path_str, params.no_guess_mime_type);
                upload_file(
                    params.client,
                    params.tls_config,
                    &path_str,
                    params.bucket,
                    &key,
                    ct,
                    params.sse,
                    params.quiet,
                    params.dryrun,
                )
                .await?;
            }
        }
    }
    Ok(())
}

/// 再帰的にダウンロードする
#[allow(clippy::too_many_arguments)]
pub(crate) async fn download_recursive(
    client: &Client,
    tls_config: &Arc<rustls::ClientConfig>,
    bucket: &str,
    prefix: &str,
    local_dir: &str,
    sse: &UploadParams,
    filters: &[FilterRule],
    quiet: bool,
    dryrun: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut continuation_token: Option<String> = None;

    loop {
        let mut builder = client.list_objects_v2().bucket(bucket).prefix(prefix);
        if let Some(ref token) = continuation_token {
            builder = builder.continuation_token(token);
        }
        let request = builder.build_request()?;
        let output = send(
            tls_config,
            request,
            shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
        )
        .await?;

        if let Some(ref contents) = output.contents {
            for object in contents {
                if let Some(ref key) = object.key {
                    let relative = key.strip_prefix(prefix).unwrap_or(key);
                    let relative = relative.trim_start_matches('/');
                    // フィルタで除外されたファイルはスキップする
                    if !should_include(relative, filters) {
                        continue;
                    }
                    // ディレクトリトラバーサルを防止する
                    if !is_safe_relative_path(relative) {
                        eprintln!("skipping unsafe key: {key}");
                        continue;
                    }
                    let local_path = format!("{local_dir}/{relative}");
                    download_file(
                        client,
                        tls_config,
                        bucket,
                        key,
                        &local_path,
                        sse,
                        quiet,
                        dryrun,
                    )
                    .await?;
                }
            }
        }

        if output.is_truncated == Some(true) {
            continuation_token = output.next_continuation_token;
        } else {
            break;
        }
    }

    Ok(())
}

/// 再帰的に S3 → S3 コピーする
#[allow(clippy::too_many_arguments)]
pub(crate) async fn copy_recursive(
    client: &Client,
    tls_config: &Arc<rustls::ClientConfig>,
    src_bucket: &str,
    src_prefix: &str,
    dst_bucket: &str,
    dst_prefix: &str,
    sse: &UploadParams,
    filters: &[FilterRule],
    quiet: bool,
    dryrun: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut continuation_token: Option<String> = None;

    loop {
        let mut builder = client
            .list_objects_v2()
            .bucket(src_bucket)
            .prefix(src_prefix);
        if let Some(ref token) = continuation_token {
            builder = builder.continuation_token(token);
        }
        let request = builder.build_request()?;
        let output = send(
            tls_config,
            request,
            shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
        )
        .await?;

        if let Some(ref contents) = output.contents {
            for object in contents {
                if let Some(ref src_key) = object.key {
                    let relative = src_key.strip_prefix(src_prefix).unwrap_or(src_key);
                    let relative = relative.trim_start_matches('/');
                    // フィルタで除外されたファイルはスキップする
                    if !should_include(relative, filters) {
                        continue;
                    }
                    let dst_key = if dst_prefix.is_empty() {
                        relative.to_string()
                    } else {
                        let dst_prefix = dst_prefix.trim_end_matches('/');
                        format!("{dst_prefix}/{relative}")
                    };
                    if dryrun {
                        eprintln!(
                            "(dryrun) copy: s3://{src_bucket}/{src_key} to s3://{dst_bucket}/{dst_key}"
                        );
                    } else {
                        let copy_source = format!("{src_bucket}/{src_key}");
                        let builder = client
                            .copy_object()
                            .bucket(dst_bucket)
                            .key(&dst_key)
                            .copy_source(&copy_source);
                        let request = sse.apply_to_copy(builder).build_request()?;
                        send(
                            tls_config,
                            request,
                            shiguredo_s3::api::CopyObjectFluentBuilder::parse_response,
                        )
                        .await?;
                        if !quiet {
                            eprintln!(
                                "copy: s3://{src_bucket}/{src_key} -> s3://{dst_bucket}/{dst_key}"
                            );
                        }
                    }
                }
            }
        }

        if output.is_truncated == Some(true) {
            continuation_token = output.next_continuation_token;
        } else {
            break;
        }
    }

    Ok(())
}

/// 相対パスが安全か検証する (ディレクトリトラバーサル防止)
///
/// `..` コンポーネントを含むパスを拒否する。
fn is_safe_relative_path(path: &str) -> bool {
    !std::path::Path::new(path)
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
}

/// 再帰的に削除する
pub(crate) async fn delete_recursive(
    client: &Client,
    tls_config: &Arc<rustls::ClientConfig>,
    bucket: &str,
    prefix: &str,
    filters: &[FilterRule],
    quiet: bool,
    dryrun: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut continuation_token: Option<String> = None;

    loop {
        let mut builder = client.list_objects_v2().bucket(bucket).prefix(prefix);
        if let Some(ref token) = continuation_token {
            builder = builder.continuation_token(token);
        }
        let request = builder.build_request()?;
        let output = send(
            tls_config,
            request,
            shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
        )
        .await?;

        if let Some(ref contents) = output.contents {
            // 1000 件ずつ一括削除する
            let filtered: Vec<_> = contents
                .iter()
                .filter(|obj| {
                    obj.key.as_ref().is_some_and(|key| {
                        let relative = key.strip_prefix(prefix).unwrap_or(key);
                        let relative = relative.trim_start_matches('/');
                        should_include(relative, filters)
                    })
                })
                .collect();
            if dryrun {
                for object in &filtered {
                    if let Some(ref key) = object.key {
                        eprintln!("(dryrun) delete: s3://{bucket}/{key}");
                    }
                }
            } else {
                for chunk in filtered.chunks(1000) {
                    let mut delete_builder = client.delete_objects().bucket(bucket).quiet(true);
                    for object in chunk {
                        if let Some(ref key) = object.key {
                            delete_builder = delete_builder.object(ObjectIdentifier {
                                key: key.clone(),
                                version_id: None,
                            });
                            if !quiet {
                                eprintln!("delete: s3://{bucket}/{key}");
                            }
                        }
                    }
                    let request = delete_builder.build_request()?;
                    send(
                        tls_config,
                        request,
                        shiguredo_s3::api::DeleteObjectsFluentBuilder::parse_response,
                    )
                    .await?;
                }
            }
        }

        if output.is_truncated == Some(true) {
            continuation_token = output.next_continuation_token;
        } else {
            break;
        }
    }

    Ok(())
}
