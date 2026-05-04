// -------------------------------------------------------
// サブコマンド
// -------------------------------------------------------

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::ops::{
    RecursiveUploadParams, copy_recursive, delete_recursive, download_file, download_recursive,
    upload_file, upload_recursive,
};
use crate::params::UploadParams;
use crate::transport::send;
use crate::upload::{
    DEFAULT_CONCURRENCY, MultipartUploadParams, UploadData, calculate_part_size, upload_multipart,
};
use crate::util::{
    build_client, human_readable_size, is_s3_uri, parse_filters, parse_s3_uri,
    resolve_content_type, should_include,
};

/// cp サブコマンド
pub(crate) async fn cmd_cp(
    mut args: noargs::RawArgs,
    tls_config: Arc<rustls::ClientConfig>,
) -> noargs::Result<()> {
    noargs::HELP_FLAG.take_help(&mut args);

    let recursive = noargs::flag("recursive")
        .doc("Recursively copy files")
        .take(&mut args)
        .is_present();

    let quiet = noargs::flag("quiet")
        .doc("Suppress all output messages")
        .take(&mut args)
        .is_present()
        || noargs::flag("only-show-errors")
            .doc("Only show error messages")
            .take(&mut args)
            .is_present();

    let dryrun = noargs::flag("dryrun")
        .doc("Display operations without executing them")
        .take(&mut args)
        .is_present();

    let content_type: Option<String> = noargs::opt("content-type")
        .doc("Content type for the object")
        .ty("TYPE")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let cache_control: Option<String> = noargs::opt("cache-control")
        .doc("Cache-Control header")
        .ty("VALUE")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let content_disposition: Option<String> = noargs::opt("content-disposition")
        .doc("Content-Disposition header")
        .ty("VALUE")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let content_encoding: Option<String> = noargs::opt("content-encoding")
        .doc("Content-Encoding header")
        .ty("VALUE")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let content_language: Option<String> = noargs::opt("content-language")
        .doc("Content-Language header")
        .ty("VALUE")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let expires: Option<String> = noargs::opt("expires")
        .doc("Expires header (RFC 7234)")
        .ty("VALUE")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let metadata_directive: Option<String> = noargs::opt("metadata-directive")
        .doc("Metadata directive for S3-to-S3 copy (COPY or REPLACE)")
        .ty("DIRECTIVE")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let metadata_raw: Option<String> = noargs::opt("metadata")
        .doc("Custom metadata (key=value,key2=value2)")
        .ty("METADATA")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let no_guess_mime_type = noargs::flag("no-guess-mime-type")
        .doc("Do not guess MIME type from file extension")
        .take(&mut args)
        .is_present();

    let follow_symlinks = !noargs::flag("no-follow-symlinks")
        .doc("Do not follow symbolic links")
        .take(&mut args)
        .is_present();

    // --exclude / --include フィルタを構築する
    // 複数指定に対応するため、RawArgs から直接パースする
    let filters = parse_filters(&mut args);

    let expected_size: Option<u64> = noargs::opt("expected-size")
        .doc("Expected data size for stdin upload (bytes)")
        .ty("BYTES")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let storage_class: Option<String> = noargs::opt("storage-class")
        .doc("Storage class (STANDARD, STANDARD_IA, GLACIER, etc.)")
        .ty("CLASS")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let checksum_algorithm: Option<String> = noargs::opt("checksum-algorithm")
        .doc("Checksum algorithm (CRC32, CRC32C, SHA1, SHA256, CRC64NVME)")
        .ty("ALGO")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let acl: Option<String> = noargs::opt("acl")
        .doc("ACL (private, public-read, public-read-write, etc.)")
        .ty("ACL")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let sse_opt: Option<String> = noargs::opt("sse")
        .doc("Server-side encryption (AES256 or aws:kms)")
        .ty("METHOD")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let sse_kms_key_id: Option<String> = noargs::opt("sse-kms-key-id")
        .doc("SSE-KMS key ID")
        .ty("KEY_ID")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let sse_c: Option<String> = noargs::opt("sse-c")
        .doc("SSE-C algorithm (AES256)")
        .ty("ALGO")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let sse_c_key: Option<String> = noargs::opt("sse-c-key")
        .doc("SSE-C key (Base64 encoded)")
        .ty("KEY")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let sse_c_copy_source: Option<String> = noargs::opt("sse-c-copy-source")
        .doc("SSE-C algorithm for copy source (AES256)")
        .ty("ALGO")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let sse_c_copy_source_key: Option<String> = noargs::opt("sse-c-copy-source-key")
        .doc("SSE-C key for copy source (Base64 encoded)")
        .ty("KEY")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let src: String = noargs::arg("<SRC>")
        .doc("Source path (local path or s3://bucket/key)")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let dst: String = noargs::arg("<DST>")
        .doc("Destination path (local path or s3://bucket/key)")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let client = build_client(&args)?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    // --metadata パース: "key=value,key2=value2" 形式
    let metadata: Vec<(String, String)> = metadata_raw
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            Some((k.to_string(), v.to_string()))
        })
        .collect();

    let upload_params = UploadParams {
        sse: sse_opt,
        sse_kms_key_id,
        sse_c,
        sse_c_key,
        sse_c_copy_source,
        sse_c_copy_source_key,
        acl,
        cache_control,
        content_disposition,
        content_encoding,
        content_language,
        expires,
        metadata_directive,
        metadata,
        storage_class,
        checksum_algorithm,
    };

    match (is_s3_uri(&src), is_s3_uri(&dst)) {
        (false, true) => {
            let (bucket, key) = parse_s3_uri(&dst).ok_or("invalid S3 URI")?;
            let ct = resolve_content_type(content_type.as_deref(), &key, no_guess_mime_type);
            if src == "-" {
                // stdin からアップロードする
                let mut body = Vec::new();
                if let Some(size) = expected_size {
                    body.reserve(size as usize);
                }
                tokio::io::AsyncReadExt::read_to_end(&mut tokio::io::stdin(), &mut body).await?;
                let part_size = calculate_part_size(body.len());
                let params = MultipartUploadParams {
                    client: &client,
                    tls_config: &tls_config,
                    bucket: &bucket,
                    key: &key,
                    data: UploadData::Memory(&body),
                    content_type: ct,
                    part_size,
                    concurrency: DEFAULT_CONCURRENCY,
                    sse: &upload_params,
                };
                upload_multipart(&params).await?;
                if !quiet {
                    eprintln!("upload: (stdin) -> s3://{bucket}/{key}");
                }
            } else if recursive {
                upload_recursive(&RecursiveUploadParams {
                    client: &client,
                    tls_config: &tls_config,
                    local_dir: &src,
                    bucket: &bucket,
                    prefix: &key,
                    content_type: content_type.as_deref(),
                    no_guess_mime_type,
                    follow_symlinks,
                    sse: &upload_params,
                    filters: &filters,
                    quiet,
                    dryrun,
                })
                .await?;
            } else {
                let file_ct =
                    resolve_content_type(content_type.as_deref(), &src, no_guess_mime_type);
                upload_file(
                    &client,
                    &tls_config,
                    &src,
                    &bucket,
                    &key,
                    file_ct,
                    &upload_params,
                    quiet,
                    dryrun,
                )
                .await?;
            }
        }
        (true, false) => {
            let (bucket, key) = parse_s3_uri(&src).ok_or("invalid S3 URI")?;
            if recursive {
                download_recursive(
                    &client,
                    &tls_config,
                    &bucket,
                    &key,
                    &dst,
                    &upload_params,
                    &filters,
                    quiet,
                    dryrun,
                )
                .await?;
            } else {
                download_file(
                    &client,
                    &tls_config,
                    &bucket,
                    &key,
                    &dst,
                    &upload_params,
                    quiet,
                    dryrun,
                )
                .await?;
            }
        }
        (true, true) => {
            let (src_bucket, src_key) = parse_s3_uri(&src).ok_or("invalid S3 URI")?;
            let (dst_bucket, dst_key) = parse_s3_uri(&dst).ok_or("invalid S3 URI")?;
            if recursive {
                copy_recursive(
                    &client,
                    &tls_config,
                    &src_bucket,
                    &src_key,
                    &dst_bucket,
                    &dst_key,
                    &upload_params,
                    &filters,
                    quiet,
                    dryrun,
                )
                .await?;
            } else if dryrun {
                eprintln!(
                    "(dryrun) copy: s3://{src_bucket}/{src_key} to s3://{dst_bucket}/{dst_key}"
                );
            } else {
                let copy_source = format!("{src_bucket}/{src_key}");
                let builder = client
                    .copy_object()
                    .bucket(&dst_bucket)
                    .key(&dst_key)
                    .copy_source(&copy_source);
                let request = upload_params.apply_to_copy(builder).build_request()?;
                send(
                    &tls_config,
                    request,
                    shiguredo_s3::api::CopyObjectFluentBuilder::parse_response,
                )
                .await?;
                if !quiet {
                    eprintln!("copy: s3://{src_bucket}/{src_key} -> s3://{dst_bucket}/{dst_key}");
                }
            }
        }
        (false, false) => {
            return Err("at least one path must be an S3 URI".into());
        }
    }

    Ok(())
}

/// mv サブコマンド
pub(crate) async fn cmd_mv(
    mut args: noargs::RawArgs,
    tls_config: Arc<rustls::ClientConfig>,
) -> noargs::Result<()> {
    noargs::HELP_FLAG.take_help(&mut args);

    let recursive = noargs::flag("recursive")
        .doc("Recursively move files")
        .take(&mut args)
        .is_present();

    let quiet = noargs::flag("quiet")
        .doc("Suppress all output messages")
        .take(&mut args)
        .is_present()
        || noargs::flag("only-show-errors")
            .doc("Only show error messages")
            .take(&mut args)
            .is_present();

    let dryrun = noargs::flag("dryrun")
        .doc("Display operations without executing them")
        .take(&mut args)
        .is_present();

    // --exclude / --include フィルタを構築する
    let filters = parse_filters(&mut args);

    let src: String = noargs::arg("<SRC>")
        .doc("Source path (local path or s3://bucket/key)")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let dst: String = noargs::arg("<DST>")
        .doc("Destination path (local path or s3://bucket/key)")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let client = build_client(&args)?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let upload_params = UploadParams::default();

    match (is_s3_uri(&src), is_s3_uri(&dst)) {
        (false, true) => {
            let (bucket, key) = parse_s3_uri(&dst).ok_or("invalid S3 URI")?;
            if recursive {
                upload_recursive(&RecursiveUploadParams {
                    client: &client,
                    tls_config: &tls_config,
                    local_dir: &src,
                    bucket: &bucket,
                    prefix: &key,
                    content_type: None,
                    no_guess_mime_type: false,
                    follow_symlinks: true,
                    sse: &upload_params,
                    filters: &filters,
                    quiet,
                    dryrun,
                })
                .await?;
                if !dryrun {
                    tokio::fs::remove_dir_all(&src).await?;
                }
            } else {
                let file_ct = resolve_content_type(None, &src, false);
                upload_file(
                    &client,
                    &tls_config,
                    &src,
                    &bucket,
                    &key,
                    file_ct,
                    &upload_params,
                    quiet,
                    dryrun,
                )
                .await?;
                if !dryrun {
                    tokio::fs::remove_file(&src).await?;
                }
            }
        }
        (true, false) => {
            let (bucket, key) = parse_s3_uri(&src).ok_or("invalid S3 URI")?;
            if recursive {
                download_recursive(
                    &client,
                    &tls_config,
                    &bucket,
                    &key,
                    &dst,
                    &upload_params,
                    &filters,
                    quiet,
                    dryrun,
                )
                .await?;
                delete_recursive(&client, &tls_config, &bucket, &key, &filters, quiet, dryrun)
                    .await?;
            } else {
                download_file(
                    &client,
                    &tls_config,
                    &bucket,
                    &key,
                    &dst,
                    &upload_params,
                    quiet,
                    dryrun,
                )
                .await?;
                if !dryrun {
                    let request = client
                        .delete_object()
                        .bucket(&bucket)
                        .key(&key)
                        .build_request()?;
                    send(
                        &tls_config,
                        request,
                        shiguredo_s3::api::DeleteObjectFluentBuilder::parse_response,
                    )
                    .await?;
                }
            }
        }
        (true, true) => {
            let (src_bucket, src_key) = parse_s3_uri(&src).ok_or("invalid S3 URI")?;
            let (dst_bucket, dst_key) = parse_s3_uri(&dst).ok_or("invalid S3 URI")?;
            if recursive {
                copy_recursive(
                    &client,
                    &tls_config,
                    &src_bucket,
                    &src_key,
                    &dst_bucket,
                    &dst_key,
                    &upload_params,
                    &filters,
                    quiet,
                    dryrun,
                )
                .await?;
                delete_recursive(
                    &client,
                    &tls_config,
                    &src_bucket,
                    &src_key,
                    &filters,
                    quiet,
                    dryrun,
                )
                .await?;
            } else if dryrun {
                eprintln!(
                    "(dryrun) move: s3://{src_bucket}/{src_key} to s3://{dst_bucket}/{dst_key}"
                );
            } else {
                let copy_source = format!("{src_bucket}/{src_key}");
                let builder = client
                    .copy_object()
                    .bucket(&dst_bucket)
                    .key(&dst_key)
                    .copy_source(&copy_source);
                let request = upload_params.apply_to_copy(builder).build_request()?;
                send(
                    &tls_config,
                    request,
                    shiguredo_s3::api::CopyObjectFluentBuilder::parse_response,
                )
                .await?;
                let request = client
                    .delete_object()
                    .bucket(&src_bucket)
                    .key(&src_key)
                    .build_request()?;
                send(
                    &tls_config,
                    request,
                    shiguredo_s3::api::DeleteObjectFluentBuilder::parse_response,
                )
                .await?;
                if !quiet {
                    eprintln!("move: s3://{src_bucket}/{src_key} -> s3://{dst_bucket}/{dst_key}");
                }
            }
        }
        (false, false) => {
            return Err("at least one path must be an S3 URI".into());
        }
    }

    Ok(())
}

/// ls サブコマンド
pub(crate) async fn cmd_ls(
    mut args: noargs::RawArgs,
    tls_config: Arc<rustls::ClientConfig>,
) -> noargs::Result<()> {
    noargs::HELP_FLAG.take_help(&mut args);

    let recursive = noargs::flag("recursive")
        .doc("Recursively list all objects")
        .take(&mut args)
        .is_present();

    let human_readable = noargs::flag("human-readable")
        .doc("Display file sizes in human-readable format")
        .take(&mut args)
        .is_present();

    let summarize = noargs::flag("summarize")
        .doc("Display summary information (total objects and size)")
        .take(&mut args)
        .is_present();

    let page_size: Option<i32> = noargs::opt("page-size")
        .doc("Number of results per page for pagination")
        .ty("SIZE")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let s3_uri: Option<String> = noargs::arg("<S3URI>")
        .doc("S3 URI (s3://bucket[/prefix]), omit to list buckets")
        .take(&mut args)
        .present_and_then(|a| a.value().parse())?;

    let client = build_client(&args)?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    // 引数なしの場合はバケット一覧を表示する
    let Some(s3_uri) = s3_uri else {
        let mut continuation_token: Option<String> = None;
        loop {
            let mut builder = client.list_buckets();
            if let Some(ref token) = continuation_token {
                builder = builder.continuation_token(token);
            }
            let request = builder.build_request()?;
            let output = send(
                &tls_config,
                request,
                shiguredo_s3::api::ListBucketsFluentBuilder::parse_response,
            )
            .await?;

            for bucket in &output.buckets {
                let name = bucket.name.as_deref().unwrap_or("");
                let date = bucket.creation_date.as_deref().unwrap_or("");
                println!("{date} {name}");
            }

            if let Some(token) = output.continuation_token {
                continuation_token = Some(token);
            } else {
                break;
            }
        }
        return Ok(());
    };

    let (bucket, prefix) = parse_s3_uri(&s3_uri).ok_or("invalid S3 URI")?;

    let delimiter = if recursive { None } else { Some("/") };

    let mut continuation_token: Option<String> = None;
    let mut total_objects: u64 = 0;
    let mut total_size: i64 = 0;

    loop {
        let mut builder = client.list_objects_v2().bucket(&bucket);

        if !prefix.is_empty() {
            builder = builder.prefix(&prefix);
        }
        if let Some(d) = delimiter {
            builder = builder.delimiter(d);
        }
        if let Some(ref token) = continuation_token {
            builder = builder.continuation_token(token);
        }
        if let Some(ps) = page_size {
            builder = builder.max_keys(ps);
        }

        let request = builder.build_request()?;
        let output = send(
            &tls_config,
            request,
            shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
        )
        .await?;

        // 共通プレフィックス (ディレクトリ) を表示する
        if let Some(ref prefixes) = output.common_prefixes {
            for cp in prefixes {
                if let Some(ref p) = cp.prefix {
                    println!("                           PRE {p}");
                }
            }
        }

        // オブジェクトを表示する
        if let Some(ref contents) = output.contents {
            for object in contents {
                let key = object.key.as_deref().unwrap_or("");
                let size = object.size.unwrap_or(0);
                let last_modified = object.last_modified.as_deref().unwrap_or("");

                let size_str = if human_readable {
                    format!("{:>10}", human_readable_size(size))
                } else {
                    format!("{size:>10}")
                };

                println!("{last_modified} {size_str} {key}");

                total_objects += 1;
                total_size += size;
            }
        }

        if output.is_truncated == Some(true) {
            continuation_token = output.next_continuation_token;
        } else {
            break;
        }
    }

    if summarize {
        println!();
        println!("Total Objects: {total_objects}");
        if human_readable {
            println!("   Total Size: {}", human_readable_size(total_size));
        } else {
            println!("   Total Size: {total_size}");
        }
    }

    Ok(())
}

/// ListObjectsV2 の `Object` から実効ストレージクラスを得る
///
/// AWS では STANDARD のとき `StorageClass` 要素が省略されることがある。
/// 省略時は STANDARD とみなす。
fn effective_storage_class(object: &shiguredo_s3::types::Object) -> &str {
    object.storage_class.as_deref().unwrap_or("STANDARD")
}

/// check-storage-class サブコマンド
///
/// プレフィックス配下を ListObjectsV2 でページングし、`Contents` の `StorageClass` を検査する。
/// 省略は AWS 互換で STANDARD とみなす（`aws` の `(.StorageClass // "STANDARD")` と同じ前提）。
/// 非 STANDARD のキーは標準出力に `KEY<TAB>CLASS`、集計は標準エラーに出す。
///
/// バケットの「既定ストレージクラス」は S3 の ListBuckets 等には載らない（オブジェクト単位の一覧のみ）。
pub(crate) async fn cmd_check_storage_class(
    mut args: noargs::RawArgs,
    tls_config: Arc<rustls::ClientConfig>,
) -> noargs::Result<()> {
    noargs::HELP_FLAG.take_help(&mut args);

    let page_size: Option<i32> = noargs::opt("page-size")
        .doc("Number of results per page for pagination")
        .ty("SIZE")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    let s3_uri: String = noargs::arg("<S3URI>")
        .doc("S3 URI (s3://bucket[/prefix])")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let client = build_client(&args)?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let (bucket, prefix) = parse_s3_uri(&s3_uri).ok_or("invalid S3 URI")?;

    let mut continuation_token: Option<String> = None;
    let mut total: u64 = 0;
    let mut non_standard: u64 = 0;
    // 実効ストレージクラス（大文字化）ごとの件数（ListObjectsV2 と同じ解釈）
    let mut by_storage_class: BTreeMap<String, u64> = BTreeMap::new();

    loop {
        let mut builder = client.list_objects_v2().bucket(&bucket);
        if !prefix.is_empty() {
            builder = builder.prefix(&prefix);
        }
        if let Some(ref token) = continuation_token {
            builder = builder.continuation_token(token);
        }
        if let Some(ps) = page_size {
            builder = builder.max_keys(ps);
        }

        let request = builder.build_request()?;
        let output = send(
            &tls_config,
            request,
            shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
        )
        .await?;

        if let Some(ref contents) = output.contents {
            for object in contents {
                let key = object.key.as_deref().unwrap_or("");
                if key.is_empty() {
                    continue;
                }
                let actual = effective_storage_class(object);
                let class_key = actual.to_ascii_uppercase();
                *by_storage_class.entry(class_key).or_insert(0) += 1;
                total += 1;
                if !actual.eq_ignore_ascii_case("STANDARD") {
                    non_standard += 1;
                    println!("{key}\t{actual}");
                }
            }
        }

        if output.is_truncated == Some(true) {
            continuation_token = output.next_continuation_token;
        } else {
            break;
        }
    }

    let standard_total = *by_storage_class.get("STANDARD").unwrap_or(&0);
    eprintln!("check-storage-class: objects_total={total}");
    eprintln!("check-storage-class: standard_total={standard_total}");
    eprintln!("check-storage-class: non_standard_total={non_standard}");
    let parts: Vec<String> = by_storage_class
        .iter()
        .map(|(c, n)| format!("{c}={n}"))
        .collect();
    eprintln!("check-storage-class: by_storage_class {}", parts.join(" "));

    if non_standard > 0 {
        std::process::exit(1);
    }
    Ok(())
}

/// rm サブコマンド
pub(crate) async fn cmd_rm(
    mut args: noargs::RawArgs,
    tls_config: Arc<rustls::ClientConfig>,
) -> noargs::Result<()> {
    noargs::HELP_FLAG.take_help(&mut args);

    let recursive = noargs::flag("recursive")
        .doc("Recursively delete all objects under the prefix")
        .take(&mut args)
        .is_present();

    let quiet = noargs::flag("quiet")
        .doc("Suppress all output messages")
        .take(&mut args)
        .is_present()
        || noargs::flag("only-show-errors")
            .doc("Only show error messages")
            .take(&mut args)
            .is_present();

    let dryrun = noargs::flag("dryrun")
        .doc("Display operations without executing them")
        .take(&mut args)
        .is_present();

    // --exclude / --include フィルタを構築する
    let filters = parse_filters(&mut args);

    let s3_uri: String = noargs::arg("<S3URI>")
        .doc("S3 URI (s3://bucket/key)")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let client = build_client(&args)?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let (bucket, key) = parse_s3_uri(&s3_uri).ok_or("invalid S3 URI")?;

    if recursive {
        delete_recursive(&client, &tls_config, &bucket, &key, &filters, quiet, dryrun).await?;
    } else if dryrun {
        eprintln!("(dryrun) delete: s3://{bucket}/{key}");
    } else {
        if key.is_empty() {
            return Err("key is required for non-recursive delete".into());
        }
        let request = client
            .delete_object()
            .bucket(&bucket)
            .key(&key)
            .build_request()?;
        send(
            &tls_config,
            request,
            shiguredo_s3::api::DeleteObjectFluentBuilder::parse_response,
        )
        .await?;
        if !quiet {
            eprintln!("delete: s3://{bucket}/{key}");
        }
    }

    Ok(())
}

/// mb サブコマンド
pub(crate) async fn cmd_mb(
    mut args: noargs::RawArgs,
    tls_config: Arc<rustls::ClientConfig>,
) -> noargs::Result<()> {
    noargs::HELP_FLAG.take_help(&mut args);

    let s3_uri: String = noargs::arg("<S3URI>")
        .doc("S3 URI (s3://bucket-name)")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let client = build_client(&args)?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let (bucket, _) = parse_s3_uri(&s3_uri).ok_or("invalid S3 URI")?;

    let region =
        std::env::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "ap-northeast-1".to_string());

    let mut builder = client.create_bucket().bucket(&bucket);
    // us-east-1 以外のリージョンでは LocationConstraint の指定が必要
    if region != "us-east-1" {
        builder =
            builder.create_bucket_configuration(shiguredo_s3::types::CreateBucketConfiguration {
                location_constraint: Some(region),
            });
    }
    let request = builder.build_request()?;
    send(
        &tls_config,
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await?;
    eprintln!("make_bucket: s3://{bucket}");

    Ok(())
}

/// rb サブコマンド
pub(crate) async fn cmd_rb(
    mut args: noargs::RawArgs,
    tls_config: Arc<rustls::ClientConfig>,
) -> noargs::Result<()> {
    noargs::HELP_FLAG.take_help(&mut args);

    let force = noargs::flag("force")
        .doc("Remove all objects before deleting the bucket")
        .take(&mut args)
        .is_present();

    let s3_uri: String = noargs::arg("<S3URI>")
        .doc("S3 URI (s3://bucket-name)")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let client = build_client(&args)?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let (bucket, _) = parse_s3_uri(&s3_uri).ok_or("invalid S3 URI")?;

    if force {
        delete_recursive(&client, &tls_config, &bucket, "", &[], false, false).await?;
    }

    let request = client.delete_bucket().bucket(&bucket).build_request()?;
    send(
        &tls_config,
        request,
        shiguredo_s3::api::DeleteBucketFluentBuilder::parse_response,
    )
    .await?;
    eprintln!("remove_bucket: s3://{bucket}");

    Ok(())
}

/// ローカルファイルの相対パス・サイズ・最終更新日時を収集する
async fn collect_local_files(
    base: &str,
) -> Result<Vec<(String, i64, std::time::SystemTime)>, Box<dyn std::error::Error + Send + Sync>> {
    let mut files = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(base)];
    let base_path = std::path::Path::new(base);

    while let Some(dir) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let meta = tokio::fs::metadata(&path).await?;
                let relative = path
                    .strip_prefix(base_path)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                let modified = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                files.push((relative, meta.len() as i64, modified));
            }
        }
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

/// S3 オブジェクトのキー・サイズ・LastModified を収集する
async fn collect_s3_objects(
    client: &shiguredo_s3::Client,
    tls_config: &Arc<rustls::ClientConfig>,
    bucket: &str,
    prefix: &str,
) -> Result<Vec<(String, i64, String)>, Box<dyn std::error::Error + Send + Sync>> {
    let mut objects = Vec::new();
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
                    if relative.is_empty() {
                        continue;
                    }
                    objects.push((
                        relative.to_string(),
                        object.size.unwrap_or(0),
                        object.last_modified.clone().unwrap_or_default(),
                    ));
                }
            }
        }

        if output.is_truncated == Some(true) {
            continuation_token = output.next_continuation_token;
        } else {
            break;
        }
    }

    objects.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(objects)
}

/// ISO 8601 形式の日時文字列を SystemTime に変換する
fn parse_s3_timestamp(s: &str) -> Option<std::time::SystemTime> {
    // "2024-01-15T12:30:45.000Z" 形式
    let s = s.trim_end_matches('Z');
    let (date_part, time_part) = s.split_once('T')?;
    let mut date_iter = date_part.split('-');
    let year: i64 = date_iter.next()?.parse().ok()?;
    let month: u64 = date_iter.next()?.parse().ok()?;
    let day: u64 = date_iter.next()?.parse().ok()?;

    let time_part = time_part.split('.').next()?;
    let mut time_iter = time_part.split(':');
    let hour: u64 = time_iter.next()?.parse().ok()?;
    let min: u64 = time_iter.next()?.parse().ok()?;
    let sec: u64 = time_iter.next()?.parse().ok()?;

    // 簡易的なエポック秒計算
    let mut days: i64 = 0;
    for y in 1970..year {
        days += if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
    }
    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    for d in month_days.iter().take(month as usize - 1) {
        days += *d as i64;
    }
    days += day as i64 - 1;

    let secs = days as u64 * 86400 + hour * 3600 + min * 60 + sec;
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
}

/// 同期時にファイルをスキップすべきか判定する
fn should_skip_sync(
    src_size: i64,
    dst_size: i64,
    src_mtime: Option<std::time::SystemTime>,
    dst_mtime: Option<std::time::SystemTime>,
    size_only: bool,
    exact_timestamps: bool,
) -> bool {
    if size_only {
        return src_size == dst_size;
    }
    if src_size != dst_size {
        return false;
    }
    // サイズが同じ場合はタイムスタンプで判定する
    match (src_mtime, dst_mtime) {
        (Some(src_t), Some(dst_t)) => {
            if exact_timestamps {
                src_t == dst_t
            } else {
                // デフォルト: ソースが新しくなければスキップする
                src_t <= dst_t
            }
        }
        _ => false,
    }
}

/// sync サブコマンド
#[allow(clippy::too_many_lines)]
pub(crate) async fn cmd_sync(
    mut args: noargs::RawArgs,
    tls_config: Arc<rustls::ClientConfig>,
) -> noargs::Result<()> {
    noargs::HELP_FLAG.take_help(&mut args);

    let delete = noargs::flag("delete")
        .doc("Delete files that exist in destination but not in source")
        .take(&mut args)
        .is_present();

    let size_only = noargs::flag("size-only")
        .doc("Compare only file sizes (ignore timestamps)")
        .take(&mut args)
        .is_present();

    let exact_timestamps = noargs::flag("exact-timestamps")
        .doc("Skip only when timestamps match exactly")
        .take(&mut args)
        .is_present();

    let quiet = noargs::flag("quiet")
        .doc("Suppress all output messages")
        .take(&mut args)
        .is_present()
        || noargs::flag("only-show-errors")
            .doc("Only show error messages")
            .take(&mut args)
            .is_present();

    let dryrun = noargs::flag("dryrun")
        .doc("Display operations without executing them")
        .take(&mut args)
        .is_present();

    let filters = parse_filters(&mut args);

    let src: String = noargs::arg("<SRC>")
        .doc("Source path (local path or s3://bucket/prefix)")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let dst: String = noargs::arg("<DST>")
        .doc("Destination path (local path or s3://bucket/prefix)")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let client = build_client(&args)?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let upload_params = UploadParams::default();

    match (is_s3_uri(&src), is_s3_uri(&dst)) {
        // ローカル → S3
        (false, true) => {
            let (bucket, prefix) = parse_s3_uri(&dst).ok_or("invalid S3 URI")?;
            let local_files = collect_local_files(&src).await?;
            let s3_objects = collect_s3_objects(&client, &tls_config, &bucket, &prefix).await?;

            // ローカルファイルをアップロードする
            for (relative, local_size, local_mtime) in &local_files {
                if !should_include(relative, &filters) {
                    continue;
                }
                // S3 側に同じキーが存在するか確認する
                if let Ok(idx) = s3_objects.binary_search_by(|o| o.0.cmp(relative)) {
                    let (_, s3_size, ref s3_mtime) = s3_objects[idx];
                    if should_skip_sync(
                        *local_size,
                        s3_size,
                        Some(*local_mtime),
                        parse_s3_timestamp(s3_mtime),
                        size_only,
                        exact_timestamps,
                    ) {
                        continue;
                    }
                }
                let key = if prefix.is_empty() {
                    relative.clone()
                } else {
                    let prefix = prefix.trim_end_matches('/');
                    format!("{prefix}/{relative}")
                };
                let local_path = format!("{src}/{relative}");
                let ct = resolve_content_type(None, &local_path, false);
                upload_file(
                    &client,
                    &tls_config,
                    &local_path,
                    &bucket,
                    &key,
                    ct,
                    &upload_params,
                    quiet,
                    dryrun,
                )
                .await?;
            }

            // --delete: S3 にのみ存在するファイルを削除する
            if delete {
                let local_set: std::collections::HashSet<&str> =
                    local_files.iter().map(|(k, _, _)| k.as_str()).collect();
                for (relative, _, _) in &s3_objects {
                    if !should_include(relative, &filters) {
                        continue;
                    }
                    if !local_set.contains(relative.as_str()) {
                        let key = if prefix.is_empty() {
                            relative.clone()
                        } else {
                            let prefix = prefix.trim_end_matches('/');
                            format!("{prefix}/{relative}")
                        };
                        if dryrun {
                            eprintln!("(dryrun) delete: s3://{bucket}/{key}");
                        } else {
                            let request = client
                                .delete_object()
                                .bucket(&bucket)
                                .key(&key)
                                .build_request()?;
                            send(
                                &tls_config,
                                request,
                                shiguredo_s3::api::DeleteObjectFluentBuilder::parse_response,
                            )
                            .await?;
                            if !quiet {
                                eprintln!("delete: s3://{bucket}/{key}");
                            }
                        }
                    }
                }
            }
        }
        // S3 → ローカル
        (true, false) => {
            let (bucket, prefix) = parse_s3_uri(&src).ok_or("invalid S3 URI")?;
            let s3_objects = collect_s3_objects(&client, &tls_config, &bucket, &prefix).await?;

            for (relative, s3_size, s3_mtime) in &s3_objects {
                if !should_include(relative, &filters) {
                    continue;
                }
                let local_path = format!("{dst}/{relative}");
                // ローカルファイルが存在する場合は比較する
                if let Ok(meta) = tokio::fs::metadata(&local_path).await {
                    let local_mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                    if should_skip_sync(
                        meta.len() as i64,
                        *s3_size,
                        parse_s3_timestamp(s3_mtime.as_str()),
                        Some(local_mtime),
                        size_only,
                        exact_timestamps,
                    ) {
                        continue;
                    }
                }
                let key = if prefix.is_empty() {
                    relative.clone()
                } else {
                    let prefix = prefix.trim_end_matches('/');
                    format!("{prefix}/{relative}")
                };
                download_file(
                    &client,
                    &tls_config,
                    &bucket,
                    &key,
                    &local_path,
                    &upload_params,
                    quiet,
                    dryrun,
                )
                .await?;
            }

            // --delete: ローカルにのみ存在するファイルを削除する
            if delete {
                let s3_set: std::collections::HashSet<&str> =
                    s3_objects.iter().map(|(k, _, _)| k.as_str()).collect();
                let local_files = collect_local_files(&dst).await?;
                for (relative, _, _) in &local_files {
                    if !should_include(relative, &filters) {
                        continue;
                    }
                    if !s3_set.contains(relative.as_str()) {
                        let local_path = format!("{dst}/{relative}");
                        if dryrun {
                            eprintln!("(dryrun) delete: {local_path}");
                        } else {
                            tokio::fs::remove_file(&local_path).await?;
                            if !quiet {
                                eprintln!("delete: {local_path}");
                            }
                        }
                    }
                }
            }
        }
        // S3 → S3
        (true, true) => {
            let (src_bucket, src_prefix) = parse_s3_uri(&src).ok_or("invalid S3 URI")?;
            let (dst_bucket, dst_prefix) = parse_s3_uri(&dst).ok_or("invalid S3 URI")?;
            let src_objects =
                collect_s3_objects(&client, &tls_config, &src_bucket, &src_prefix).await?;
            let dst_objects =
                collect_s3_objects(&client, &tls_config, &dst_bucket, &dst_prefix).await?;

            for (relative, src_size, src_mtime) in &src_objects {
                if !should_include(relative, &filters) {
                    continue;
                }
                if let Ok(idx) = dst_objects.binary_search_by(|o| o.0.cmp(relative)) {
                    let (_, dst_size, ref dst_mtime) = dst_objects[idx];
                    if should_skip_sync(
                        *src_size,
                        dst_size,
                        parse_s3_timestamp(src_mtime.as_str()),
                        parse_s3_timestamp(dst_mtime.as_str()),
                        size_only,
                        exact_timestamps,
                    ) {
                        continue;
                    }
                }
                let src_key = if src_prefix.is_empty() {
                    relative.clone()
                } else {
                    let prefix = src_prefix.trim_end_matches('/');
                    format!("{prefix}/{relative}")
                };
                let dst_key = if dst_prefix.is_empty() {
                    relative.clone()
                } else {
                    let prefix = dst_prefix.trim_end_matches('/');
                    format!("{prefix}/{relative}")
                };
                if dryrun {
                    eprintln!(
                        "(dryrun) copy: s3://{src_bucket}/{src_key} to s3://{dst_bucket}/{dst_key}"
                    );
                } else {
                    let copy_source = format!("{src_bucket}/{src_key}");
                    let builder = client
                        .copy_object()
                        .bucket(&dst_bucket)
                        .key(&dst_key)
                        .copy_source(&copy_source);
                    let request = upload_params.apply_to_copy(builder).build_request()?;
                    send(
                        &tls_config,
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

            // --delete: 送信先にのみ存在するオブジェクトを削除する
            if delete {
                let src_set: std::collections::HashSet<&str> =
                    src_objects.iter().map(|(k, _, _)| k.as_str()).collect();
                for (relative, _, _) in &dst_objects {
                    if !should_include(relative, &filters) {
                        continue;
                    }
                    if !src_set.contains(relative.as_str()) {
                        let key = if dst_prefix.is_empty() {
                            relative.clone()
                        } else {
                            let prefix = dst_prefix.trim_end_matches('/');
                            format!("{prefix}/{relative}")
                        };
                        if dryrun {
                            eprintln!("(dryrun) delete: s3://{dst_bucket}/{key}");
                        } else {
                            let request = client
                                .delete_object()
                                .bucket(&dst_bucket)
                                .key(&key)
                                .build_request()?;
                            send(
                                &tls_config,
                                request,
                                shiguredo_s3::api::DeleteObjectFluentBuilder::parse_response,
                            )
                            .await?;
                            if !quiet {
                                eprintln!("delete: s3://{dst_bucket}/{key}");
                            }
                        }
                    }
                }
            }
        }
        (false, false) => {
            return Err("at least one path must be an S3 URI".into());
        }
    }

    Ok(())
}

/// presign サブコマンド
pub(crate) fn cmd_presign(mut args: noargs::RawArgs) -> noargs::Result<()> {
    noargs::HELP_FLAG.take_help(&mut args);

    let expires_in: u64 = noargs::opt("expires-in")
        .doc("Expiration time in seconds")
        .ty("SECONDS")
        .default("3600")
        .take(&mut args)
        .then(|o| o.value().parse())?;

    let s3_uri: String = noargs::arg("<S3URI>")
        .doc("S3 URI (s3://bucket/key)")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    let client = build_client(&args)?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let (bucket, key) = parse_s3_uri(&s3_uri).ok_or("invalid S3 URI")?;
    if key.is_empty() {
        return Err("key is required for presign".into());
    }

    let presigned = client
        .get_object()
        .bucket(&bucket)
        .key(&key)
        .presigned(expires_in)?;

    println!("{}", presigned.url);

    Ok(())
}
