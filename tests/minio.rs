//! MinIO を使った統合テスト
//!
//! testcontainers で MinIO コンテナを起動し、全 API のラウンドトリップを検証する。
//! Docker が起動していない環境ではテストがスキップされる。
//!
//! ## テスト構成
//!
//! 各テストは独立した MinIO コンテナを起動するため、テスト間の状態汚染がない。
//! コンテナはテスト終了時に自動的に破棄される。
//!
//! ## 既知の不具合
//!
//! MinIO latest イメージは PutPublicAccessBlock の XML パースに不具合があり、
//! MalformedXML (400) を返す。aws-cli でも同じ結果になることを確認済み。
//! 該当テストでは 400 が返ることを明示的に検証する。

use shiguredo_http11::ResponseDecoder;
use shiguredo_s3::api::{
    DeleteBucketLifecycleFluentBuilder, DeleteBucketPolicyFluentBuilder,
    DeleteBucketTaggingFluentBuilder, GetBucketEncryptionFluentBuilder,
    GetBucketLifecycleConfigurationFluentBuilder, GetBucketPolicyFluentBuilder,
    GetBucketTaggingFluentBuilder, GetBucketVersioningFluentBuilder,
    ListMultipartUploadsFluentBuilder, ListPartsFluentBuilder,
    PutBucketLifecycleConfigurationFluentBuilder, PutBucketPolicyFluentBuilder,
    PutBucketTaggingFluentBuilder, PutBucketVersioningFluentBuilder,
};
use shiguredo_s3::types::{
    CompletedMultipartUpload, CompletedPart, ObjectIdentifier, ServerSideEncryptionByDefault,
    ServerSideEncryptionRule, Tag,
};
use shiguredo_s3::{
    Credential, HttpDate, PresignedRequest, S3Client, S3Config, S3Request, S3Response,
};
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// -------------------------------------------------------
// テスト用定数
// -------------------------------------------------------

/// MinIO のデフォルト管理者アクセスキー
const ACCESS_KEY: &str = "minioadmin";

/// MinIO のデフォルト管理者シークレットキー
const SECRET_KEY: &str = "minioadmin";

// -------------------------------------------------------
// テスト用ヘルパー
// -------------------------------------------------------

/// MinIO コンテナを起動して (コンテナ, ホストポート) を返す
///
/// コンテナのポート 9000 をホストのランダムポートにマッピングし、
/// "API:" というログが出力されるまで待機する。
/// この文字列は MinIO の S3 API が起動完了したことを示す。
async fn start_minio() -> (ContainerAsync<GenericImage>, u16) {
    let container = GenericImage::new("minio/minio", "latest")
        .with_exposed_port(9000.tcp())
        // MinIO が S3 API の起動完了を示すログを待つ
        .with_wait_for(WaitFor::message_on_either_std("API:"))
        .with_env_var("MINIO_ROOT_USER", ACCESS_KEY)
        .with_env_var("MINIO_ROOT_PASSWORD", SECRET_KEY)
        // MinIO をシングルノードのオブジェクトストレージとして起動する
        .with_cmd(vec!["server", "/data"])
        .start()
        .await
        .expect("failed to start MinIO container");

    let port = container
        .get_host_port_ipv4(9000)
        .await
        .expect("failed to get host port");

    (container, port)
}

/// テスト用の S3Client を構築する
///
/// - region: us-east-1 (CreateBucket で LocationConstraint を省略できる)
/// - endpoint: コンテナのホストポートに接続する HTTP エンドポイント
/// - use_path_style: true (MinIO はパススタイルが必要)
fn build_client(port: u16) -> S3Client {
    let config = S3Config::builder()
        .region("us-east-1")
        .credential(Credential::new(ACCESS_KEY, SECRET_KEY))
        .endpoint(format!("http://127.0.0.1:{port}"))
        // MinIO は仮想ホストスタイルに対応していないためパススタイルを使う
        .use_path_style(true)
        .build()
        .expect("failed to build S3Config");
    S3Client::new(config)
}

/// S3Request を HTTP/1.1 で送信して S3Response を返す
///
/// shiguredo_http11 の ResponseDecoder を使って TCP ストリームからレスポンスを読む。
/// HEAD レスポンスのようにボディを持たないレスポンスは expect_no_body フラグで制御する。
async fn execute(s3_request: S3Request) -> S3Response {
    let addr = format!("{}:{}", s3_request.host, s3_request.port);
    let tcp = tokio::net::TcpStream::connect(&addr)
        .await
        .expect("failed to connect");

    let encoded = encode_request(&s3_request);
    let (mut reader, mut writer) = tokio::io::split(tcp);
    writer
        .write_all(&encoded)
        .await
        .expect("failed to write request");

    let mut decoder = ResponseDecoder::new();
    if s3_request.expect_no_body {
        // HEAD レスポンスはボディなしなので EOF を待たずにパースする
        decoder.set_expect_no_body(true);
    }

    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf).await.expect("failed to read");
        if n == 0 {
            // サーバーが接続を閉じた場合は EOF を通知してパースを試みる
            decoder.mark_eof();
            if let Some(response) = decoder.decode().expect("decode error") {
                return into_s3_response(response);
            }
            panic!("unexpected EOF");
        }
        decoder.feed(&buf[..n]).expect("feed error");
        if let Some(response) = decoder.decode().expect("decode error") {
            return into_s3_response(response);
        }
    }
}

/// S3Request を shiguredo_http11 の Request に変換してエンコードする
fn encode_request(s3_request: &S3Request) -> Vec<u8> {
    let mut request = shiguredo_http11::Request::new(&s3_request.method, &s3_request.uri);
    for (name, value) in &s3_request.headers {
        request.add_header(name, value);
    }
    if !s3_request.body.is_empty() {
        request.body = s3_request.body.clone();
    }
    request.try_encode().expect("failed to encode request")
}

/// shiguredo_http11 の Response を S3Response に変換する
fn into_s3_response(response: shiguredo_http11::Response) -> S3Response {
    S3Response {
        status_code: response.status_code,
        headers: response.headers,
        body: response.body,
    }
}

/// PresignedRequest を HTTP/1.1 で送信して S3Response を返す
///
/// presigned URL をパースして host / port / path+query を抽出し、
/// 署名なしの HTTP リクエストとして送信する。
async fn execute_presigned(presigned: &PresignedRequest) -> S3Response {
    // "http://127.0.0.1:PORT/path?query" をパースする
    let url = &presigned.url;
    let without_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .expect("presigned URL must have http(s) scheme");
    let (authority, path_and_query) = without_scheme
        .split_once('/')
        .expect("presigned URL must have path");
    let uri = format!("/{path_and_query}");

    let tcp = tokio::net::TcpStream::connect(authority)
        .await
        .expect("failed to connect for presigned request");

    let mut request = shiguredo_http11::Request::new(&presigned.method, &uri);
    request.add_header("host", authority);
    for (name, value) in &presigned.headers {
        request.add_header(name, value);
    }
    if !presigned.body.is_empty() {
        request.body = presigned.body.clone();
    }
    let encoded = request
        .try_encode()
        .expect("failed to encode presigned request");

    let expect_no_body = presigned.method == "HEAD";
    let (mut reader, mut writer) = tokio::io::split(tcp);
    writer
        .write_all(&encoded)
        .await
        .expect("failed to write presigned request");

    let mut decoder = ResponseDecoder::new();
    if expect_no_body {
        decoder.set_expect_no_body(true);
    }

    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf).await.expect("failed to read");
        if n == 0 {
            decoder.mark_eof();
            if let Some(response) = decoder.decode().expect("decode error") {
                return into_s3_response(response);
            }
            panic!("unexpected EOF");
        }
        decoder.feed(&buf[..n]).expect("feed error");
        if let Some(response) = decoder.decode().expect("decode error") {
            return into_s3_response(response);
        }
    }
}

/// build_request + execute + parse_response をまとめた便利関数
///
/// レスポンスのパースに失敗した場合はパニックする。
/// エラーレスポンスを直接検査したい場合は execute() を直接使うこと。
async fn send<T>(
    request: S3Request,
    parse: impl FnOnce(&S3Response) -> Result<T, shiguredo_s3::Error>,
) -> T {
    let response = execute(request).await;
    parse(&response).expect("failed to parse response")
}

// -------------------------------------------------------
// テスト
// -------------------------------------------------------

/// Bucket Lifecycle Configuration の CRUD 操作を検証する
///
/// PutBucketLifecycleConfiguration → GetBucketLifecycleConfiguration →
/// DeleteBucketLifecycle のラウンドトリップを検証する。
#[tokio::test]
async fn test_bucket_lifecycle() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-bucket-lifecycle";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // ライフサイクルルールを設定する
    let request = client
        .put_bucket_lifecycle_configuration()
        .bucket(bucket)
        .rule(shiguredo_s3::types::LifecycleRule {
            id: Some("expire-logs".to_string()),
            filter: Some(shiguredo_s3::types::LifecycleRuleFilter {
                prefix: Some("logs/".to_string()),
                tag: None,
                object_size_greater_than: None,
                object_size_less_than: None,
                and: None,
            }),
            status: "Enabled".to_string(),
            expiration: Some(shiguredo_s3::types::LifecycleExpiration {
                days: Some(30),
                date: None,
                expired_object_delete_marker: None,
            }),
            transitions: vec![],
            noncurrent_version_expiration: None,
            noncurrent_version_transitions: vec![],
            abort_incomplete_multipart_upload: None,
        })
        .build_request()
        .unwrap();
    send(
        request,
        PutBucketLifecycleConfigurationFluentBuilder::parse_response,
    )
    .await;

    // ライフサイクル設定を取得して検証する
    let request = client
        .get_bucket_lifecycle_configuration()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let output = send(
        request,
        GetBucketLifecycleConfigurationFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.rules.len(), 1);
    let rule = &output.rules[0];
    assert_eq!(rule.id.as_deref(), Some("expire-logs"));
    assert_eq!(rule.status, "Enabled");
    let exp = rule.expiration.as_ref().expect("expiration should exist");
    assert_eq!(exp.days, Some(30));
    let filter = rule.filter.as_ref().expect("filter should exist");
    assert_eq!(filter.prefix.as_deref(), Some("logs/"));

    // ライフサイクル設定を削除する
    let request = client
        .delete_bucket_lifecycle()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(request, DeleteBucketLifecycleFluentBuilder::parse_response).await;

    // 削除後は NoSuchLifecycleConfiguration が返る
    let request = client
        .get_bucket_lifecycle_configuration()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert!(
        !response.is_success(),
        "should fail after deleting lifecycle config"
    );
}

/// オブジェクトの Put / Get / Head / Delete のラウンドトリップを検証する
///
/// ## 検証項目
/// - PutObject でオブジェクトを作成でき、ETag が返る
/// - GetObject でボディと Content-Length が正しく取得できる
/// - HeadObject で Content-Length と ETag が PutObject の結果と一致する
/// - DeleteObject でオブジェクトを削除できる
/// - 削除後に GetObject で 404 が返る
#[tokio::test]
async fn test_object_put_get_head_delete() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-object-crud";
    let key = "hello.txt";
    let body = b"Hello, S3!";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // オブジェクトをアップロードする
    // Content-Type を指定しないと binary/octet-stream として扱われる
    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body.to_vec())
        .content_type("text/plain")
        .build_request()
        .unwrap();
    let put_output = send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;
    // PutObject は成功すると ETag (MD5 ハッシュ) を返す
    assert!(put_output.e_tag.is_some());

    // オブジェクトのボディと Content-Length を取得して確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let get_output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    // ボディのバイト列が一致することを確認する
    assert_eq!(get_output.body, body);
    // Content-Length がアップロードしたボディのバイト数と一致することを確認する
    assert_eq!(get_output.content_length, Some(body.len() as i64));

    // HEAD でメタデータのみを取得する (ボディは含まれない)
    let request = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let head_output = send(
        request,
        shiguredo_s3::api::HeadObjectFluentBuilder::parse_response,
    )
    .await;
    // Content-Length が GET の結果と一致することを確認する
    assert_eq!(head_output.content_length, Some(body.len() as i64));
    // ETag が PUT の結果と一致することを確認する
    assert_eq!(head_output.e_tag, put_output.e_tag);

    // オブジェクトを削除する
    let request = client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::DeleteObjectFluentBuilder::parse_response,
    )
    .await;

    // 削除後に GetObject を実行して 404 が返ることを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 404);
}

/// オブジェクト一覧取得の prefix / delimiter フィルタリングを検証する
///
/// ## 検証項目
/// - prefix 指定で特定のキープレフィックスのオブジェクトのみ取得できる
/// - delimiter 指定で共通プレフィックス (CommonPrefixes) が返る
#[tokio::test]
async fn test_list_objects_v2() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-list-objects";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // "dir/" プレフィックスを持つオブジェクトを 3 つ作成する
    for i in 0..3 {
        let key = format!("dir/file{i}.txt");
        let request = client
            .put_object()
            .bucket(bucket)
            .key(&key)
            .body(format!("content-{i}").into_bytes())
            .build_request()
            .unwrap();
        send(
            request,
            shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
        )
        .await;
    }

    // prefix="dir/" で一覧取得すると 3 件が返ることを確認する
    let request = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix("dir/")
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
    )
    .await;
    let contents = output.contents.expect("contents should exist");
    assert_eq!(contents.len(), 3);

    // delimiter="/" で取得すると "dir/" が CommonPrefixes に含まれることを確認する
    // これにより仮想的なディレクトリ構造を表現できる
    let request = client
        .list_objects_v2()
        .bucket(bucket)
        .delimiter("/")
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
    )
    .await;
    let prefixes = output
        .common_prefixes
        .expect("common_prefixes should exist");
    assert!(prefixes.iter().any(|p| p.prefix.as_deref() == Some("dir/")));
}

/// 同一バケット内でのオブジェクトコピーを検証する
///
/// ## 検証項目
/// - CopyObject でコピー先に ETag が返る
/// - コピー先を GetObject で取得するとコピー元と同じボディが得られる
#[tokio::test]
async fn test_copy_object() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-copy-object";
    let src_key = "original.txt";
    let dst_key = "copied.txt";
    let body = b"copy me";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // コピー元オブジェクトを作成する
    let request = client
        .put_object()
        .bucket(bucket)
        .key(src_key)
        .body(body.to_vec())
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    // オブジェクトをコピーする
    // copy_source は "{bucket}/{key}" の形式で指定する
    let copy_source = format!("{bucket}/{src_key}");
    let request = client
        .copy_object()
        .bucket(bucket)
        .key(dst_key)
        .copy_source(&copy_source)
        .build_request()
        .unwrap();
    let copy_output = send(
        request,
        shiguredo_s3::api::CopyObjectFluentBuilder::parse_response,
    )
    .await;
    // コピー成功時は ETag が返る
    assert!(copy_output.e_tag.is_some());

    // コピー先を GetObject で取得してボディが一致することを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(dst_key)
        .build_request()
        .unwrap();
    let get_output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(get_output.body, body);
}

/// 複数オブジェクトの一括削除を検証する
///
/// ## 検証項目
/// - DeleteObjects で複数オブジェクトを一度に削除できる
/// - deleted フィールドに削除済みオブジェクト数が含まれる
/// - 削除後に ListObjectsV2 でバケットが空になっていることを確認できる
#[tokio::test]
async fn test_delete_objects() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-delete-objects";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // テスト対象の 3 つのオブジェクトを作成する
    let keys: Vec<String> = (0..3).map(|i| format!("batch{i}.txt")).collect();
    for key in &keys {
        let request = client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(b"data".to_vec())
            .build_request()
            .unwrap();
        send(
            request,
            shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
        )
        .await;
    }

    // DeleteObjects で 3 つのオブジェクトを一括削除する
    // DeleteObject (単体) を 3 回呼ぶより効率的
    let mut builder = client.delete_objects().bucket(bucket);
    for key in &keys {
        builder = builder.object(ObjectIdentifier {
            key: key.clone(),
            version_id: None,
        });
    }
    let request = builder.build_request().unwrap();
    let output = send(
        request,
        shiguredo_s3::api::DeleteObjectsFluentBuilder::parse_response,
    )
    .await;
    // 削除されたオブジェクト数が 3 であることを確認する
    let deleted = output.deleted.expect("deleted should exist");
    assert_eq!(deleted.len(), 3);

    // 一覧でバケットが空になっていることを確認する
    // contents が None の場合はオブジェクトなし
    let request = client
        .list_objects_v2()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
    )
    .await;
    assert!(output.contents.is_none());
}

/// マルチパートアップロードの完全なフローを検証する
///
/// 5 MB 以上のオブジェクトは S3 の推奨に従いマルチパートアップロードを使う。
/// 各パートは 5 MB 以上である必要がある (最後のパートを除く)。
///
/// ## 検証項目
/// - CreateMultipartUpload で upload_id が返る
/// - UploadPart で各パートの ETag が返る
/// - CompleteMultipartUpload でオブジェクトが結合され ETag が返る
/// - GetObject で結合後のボディが正しく取得できる
#[tokio::test]
async fn test_multipart_upload() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-multipart";
    let key = "large.bin";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // マルチパートアップロードを開始して upload_id を取得する
    // upload_id は UploadPart と CompleteMultipartUpload で使用する
    let request = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let create_output = send(
        request,
        shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
    )
    .await;
    let upload_id = create_output.upload_id.expect("upload_id should exist");

    // 各パートを 5 MB に設定する (S3 の最小パートサイズ)
    // パート 1: 0xAA で埋めたデータ
    // パート 2: 0xBB で埋めたデータ
    let part_size = 5 * 1024 * 1024;
    let part1_data: Vec<u8> = vec![0xAA; part_size];
    let part2_data: Vec<u8> = vec![0xBB; part_size];

    // パート 1 をアップロードして ETag を取得する
    let request = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .part_number(1)
        .body(part1_data.clone())
        .build_request()
        .unwrap();
    let part1_output = send(
        request,
        shiguredo_s3::api::UploadPartFluentBuilder::parse_response,
    )
    .await;

    // パート 2 をアップロードして ETag を取得する
    let request = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .part_number(2)
        .body(part2_data.clone())
        .build_request()
        .unwrap();
    let part2_output = send(
        request,
        shiguredo_s3::api::UploadPartFluentBuilder::parse_response,
    )
    .await;

    // 全パートの ETag とパート番号を渡してアップロードを完了する
    // パート番号は昇順で指定する必要がある
    let request = client
        .complete_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .multipart_upload(CompletedMultipartUpload {
            parts: Some(vec![
                CompletedPart {
                    e_tag: part1_output.e_tag,
                    part_number: Some(1),
                },
                CompletedPart {
                    e_tag: part2_output.e_tag,
                    part_number: Some(2),
                },
            ]),
        })
        .build_request()
        .unwrap();
    let complete_output = send(
        request,
        shiguredo_s3::api::CompleteMultipartUploadFluentBuilder::parse_response,
    )
    .await;
    // 完了後は結合オブジェクト全体の ETag が返る
    assert!(complete_output.e_tag.is_some());

    // GetObject で結合されたオブジェクトのボディを取得して確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let get_output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    // 結合後のサイズが 2 パート分であることを確認する
    assert_eq!(get_output.body.len(), part_size * 2);
    // 前半がパート 1 のデータであることを確認する
    assert_eq!(&get_output.body[..part_size], &part1_data[..]);
    // 後半がパート 2 のデータであることを確認する
    assert_eq!(&get_output.body[part_size..], &part2_data[..]);
}

/// マルチパートアップロードの中断を検証する
///
/// AbortMultipartUpload でアップロードを中断するとオブジェクトが作成されない。
/// 中断しないとアップロード中のパートがストレージを占有し続けるため、
/// エラー時には必ず中断すること。
///
/// ## 検証項目
/// - AbortMultipartUpload でアップロードを中断できる
/// - 中断後に GetObject で 404 が返る (オブジェクトが作成されていない)
#[tokio::test]
async fn test_abort_multipart_upload() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-abort-multipart";
    let key = "aborted.bin";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // マルチパートアップロードを開始する
    let request = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let create_output = send(
        request,
        shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
    )
    .await;
    let upload_id = create_output.upload_id.expect("upload_id should exist");

    // アップロードを中断する (パートのアップロードなしで中断)
    let request = client
        .abort_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::AbortMultipartUploadFluentBuilder::parse_response,
    )
    .await;

    // 中断後はオブジェクトが存在しないことを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 404);
}

/// アップロード済みパートの一覧取得を検証する
///
/// ## 検証項目
/// - ListParts でアップロード済みパートの upload_id / part_number / size / e_tag が取得できる
/// - パートが昇順で返ること
#[tokio::test]
async fn test_list_parts() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-list-parts";
    let key = "multipart.bin";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // マルチパートアップロードを開始する
    let request = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let create_output = send(
        request,
        shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
    )
    .await;
    let upload_id = create_output.upload_id.expect("upload_id should exist");

    // 2 つのパート (各 5 MB) をアップロードする
    let part_size = 5 * 1024 * 1024;
    for part_number in 1..=2i32 {
        // パート番号をデータに埋め込んで識別しやすくする
        let data: Vec<u8> = vec![part_number as u8; part_size];
        let request = client
            .upload_part()
            .bucket(bucket)
            .key(key)
            .upload_id(&upload_id)
            .part_number(part_number)
            .body(data)
            .build_request()
            .unwrap();
        send(
            request,
            shiguredo_s3::api::UploadPartFluentBuilder::parse_response,
        )
        .await;
    }

    // ListParts でアップロード済みパートの情報を取得する
    let request = client
        .list_parts()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .build_request()
        .unwrap();
    let output = send(request, ListPartsFluentBuilder::parse_response).await;
    // レスポンスの upload_id が一致することを確認する
    assert_eq!(output.upload_id.as_deref(), Some(upload_id.as_str()));
    let parts = output.parts.expect("parts should exist");
    // アップロードした 2 パートが返ることを確認する
    assert_eq!(parts.len(), 2);
    // パート番号が昇順で返ることを確認する
    assert_eq!(parts[0].part_number, Some(1));
    assert_eq!(parts[1].part_number, Some(2));
    // 各パートのサイズと ETag が設定されていることを確認する
    assert!(parts[0].size.is_some());
    assert!(parts[0].e_tag.is_some());

    // テスト後は AbortMultipartUpload でクリーンアップする
    // 完了せずに放置するとストレージを占有し続ける
    let request = client
        .abort_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::AbortMultipartUploadFluentBuilder::parse_response,
    )
    .await;
}

/// 進行中マルチパートアップロードの一覧取得を検証する
///
/// ## 検証項目
/// - ListMultipartUploads で進行中のアップロード一覧が取得できる
/// - 各アップロードに upload_id と key が含まれる
/// - AbortMultipartUpload 後に一覧が空になる
#[tokio::test]
async fn test_list_multipart_uploads() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-list-mpu";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // 2 つのマルチパートアップロードを開始する
    // 異なるキーで開始することで別々のアップロードとして扱われる
    let keys = ["upload-a.bin", "upload-b.bin"];
    let mut upload_ids = Vec::new();
    for key in &keys {
        let request = client
            .create_multipart_upload()
            .bucket(bucket)
            .key(*key)
            .build_request()
            .unwrap();
        let output = send(
            request,
            shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
        )
        .await;
        upload_ids.push(output.upload_id.expect("upload_id should exist"));
    }

    // ListMultipartUploads で進行中の一覧を取得して 2 件であることを確認する
    let request = client
        .list_multipart_uploads()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let output = send(request, ListMultipartUploadsFluentBuilder::parse_response).await;
    let uploads = output.uploads.expect("uploads should exist");
    assert_eq!(uploads.len(), 2);
    // 各エントリに upload_id と key が含まれることを確認する
    assert!(uploads.iter().all(|u| u.upload_id.is_some()));
    assert!(uploads.iter().all(|u| u.key.is_some()));

    // 全てのアップロードを中止してクリーンアップする
    for (key, upload_id) in keys.iter().zip(upload_ids.iter()) {
        let request = client
            .abort_multipart_upload()
            .bucket(bucket)
            .key(*key)
            .upload_id(upload_id)
            .build_request()
            .unwrap();
        send(
            request,
            shiguredo_s3::api::AbortMultipartUploadFluentBuilder::parse_response,
        )
        .await;
    }

    // Abort 後は一覧が空 (None) になることを確認する
    let request = client
        .list_multipart_uploads()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let output = send(request, ListMultipartUploadsFluentBuilder::parse_response).await;
    assert!(output.uploads.is_none());
}

/// バケットバージョニングの有効化 / 停止のラウンドトリップを検証する
///
/// バージョニングを有効にすると、同一キーへの PutObject が新しいバージョンとして
/// 保存されるようになる。Suspended にするとバージョニングが停止するが、
/// 既存のバージョンは保持される。
///
/// ## 検証項目
/// - 初期状態で Status が未設定 (None) である
/// - Enabled に変更すると Status が "Enabled" になる
/// - Suspended に変更すると Status が "Suspended" になる
#[tokio::test]
async fn test_bucket_versioning() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-versioning";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // 初期状態はバージョニングが設定されていない (Status が None)
    let request = client
        .get_bucket_versioning()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let output = send(request, GetBucketVersioningFluentBuilder::parse_response).await;
    assert!(output.status.is_none());

    // バージョニングを有効化する
    let request = client
        .put_bucket_versioning()
        .bucket(bucket)
        .status("Enabled")
        .build_request()
        .unwrap();
    send(request, PutBucketVersioningFluentBuilder::parse_response).await;

    // Status が "Enabled" になっていることを確認する
    let request = client
        .get_bucket_versioning()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let output = send(request, GetBucketVersioningFluentBuilder::parse_response).await;
    assert_eq!(output.status.as_deref(), Some("Enabled"));

    // バージョニングを停止する (Disabled には戻せないため Suspended を使う)
    let request = client
        .put_bucket_versioning()
        .bucket(bucket)
        .status("Suspended")
        .build_request()
        .unwrap();
    send(request, PutBucketVersioningFluentBuilder::parse_response).await;

    // Status が "Suspended" になっていることを確認する
    let request = client
        .get_bucket_versioning()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let output = send(request, GetBucketVersioningFluentBuilder::parse_response).await;
    assert_eq!(output.status.as_deref(), Some("Suspended"));
}

/// バケットタグの設定 / 取得 / 削除のラウンドトリップを検証する
///
/// バケットタグはコスト配分や管理目的で使用される任意のキー・バリューペア。
///
/// ## 検証項目
/// - PutBucketTagging で複数のタグを設定できる
/// - GetBucketTagging で設定したタグが取得できる
/// - DeleteBucketTagging でタグを全削除できる
/// - 削除後に GetBucketTagging で 404 が返る
#[tokio::test]
async fn test_bucket_tagging() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-bucket-tagging";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // 2 つのタグを設定する
    let request = client
        .put_bucket_tagging()
        .bucket(bucket)
        .tag(Tag {
            key: "env".to_string(),
            value: "test".to_string(),
        })
        .tag(Tag {
            key: "project".to_string(),
            value: "s3-rs".to_string(),
        })
        .build_request()
        .unwrap();
    send(request, PutBucketTaggingFluentBuilder::parse_response).await;

    // タグを取得して設定した内容が含まれることを確認する
    let request = client
        .get_bucket_tagging()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let output = send(request, GetBucketTaggingFluentBuilder::parse_response).await;
    // タグが 2 件返ることを確認する
    assert_eq!(output.tag_set.len(), 2);
    // "env=test" タグが含まれることを確認する
    assert!(
        output
            .tag_set
            .iter()
            .any(|t| t.key == "env" && t.value == "test")
    );
    // "project=s3-rs" タグが含まれることを確認する
    assert!(
        output
            .tag_set
            .iter()
            .any(|t| t.key == "project" && t.value == "s3-rs")
    );

    // タグを全削除する
    let request = client
        .delete_bucket_tagging()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(request, DeleteBucketTaggingFluentBuilder::parse_response).await;

    // 削除後は GetBucketTagging で 404 が返ることを確認する
    let request = client
        .get_bucket_tagging()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 404);
}

/// Presigned URL による PutObject / GetObject / HeadObject / DeleteObject のラウンドトリップを検証する
///
/// 署名済み URL を使うことで、S3 クレデンシャルを持たないクライアントが
/// 一時的にオブジェクト操作を実行できる。
///
/// ## 検証項目
/// - PutObject の presigned URL でオブジェクトをアップロードできる
/// - GetObject の presigned URL でボディを取得できる
/// - HeadObject の presigned URL で Content-Length を取得できる
/// - DeleteObject の presigned URL でオブジェクトを削除できる
/// - 削除後に GetObject で 404 が返る
#[tokio::test]
async fn test_presigned_put_get_head_delete() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-presigned-crud";
    let key = "presigned.txt";
    let body = b"Hello, Presigned!";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // PutObject の presigned URL でアップロードする
    let presigned = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type("text/plain")
        .presigned(3600)
        .unwrap();
    assert_eq!(presigned.method, "PUT");
    // presigned URL にボディを付けて送信する
    let mut put_presigned = presigned;
    put_presigned.body = body.to_vec();
    let response = execute_presigned(&put_presigned).await;
    assert!(
        response.is_success(),
        "presigned PutObject failed: {}",
        response.status_code
    );

    // GetObject の presigned URL でダウンロードする
    let presigned = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(3600)
        .unwrap();
    assert_eq!(presigned.method, "GET");
    let response = execute_presigned(&presigned).await;
    assert!(response.is_success());
    assert_eq!(response.body, body);

    // HeadObject の presigned URL でメタデータを取得する
    let presigned = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .presigned(3600)
        .unwrap();
    assert_eq!(presigned.method, "HEAD");
    let response = execute_presigned(&presigned).await;
    assert!(response.is_success());
    assert_eq!(response.content_length(), Some(body.len() as u64));

    // DeleteObject の presigned URL でオブジェクトを削除する
    let presigned = client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .presigned(3600)
        .unwrap();
    assert_eq!(presigned.method, "DELETE");
    let response = execute_presigned(&presigned).await;
    assert!(response.is_success());

    // 削除後に通常の GetObject で 404 が返ることを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 404);
}

/// Presigned URL によるマルチパートアップロードのフローを検証する
///
/// ## 検証項目
/// - CreateMultipartUpload の presigned URL で upload_id を取得できる
/// - UploadPart の presigned URL でパートをアップロードできる
/// - CompleteMultipartUpload の presigned URL でアップロードを完了できる
/// - 完了後に GetObject で結合されたボディを取得できる
#[tokio::test]
async fn test_presigned_multipart_upload() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-presigned-mpu";
    let key = "presigned-multipart.bin";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // 通常の CreateMultipartUpload で upload_id を取得する
    // (presigned CreateMultipartUpload のレスポンスパースも検証する)
    let request = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let create_output = send(
        request,
        shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
    )
    .await;
    let upload_id = create_output.upload_id.expect("upload_id should exist");

    // 各パートを 5 MB に設定する (S3 の最小パートサイズ)
    let part_size = 5 * 1024 * 1024;
    let part1_data: Vec<u8> = vec![0xCC; part_size];
    let part2_data: Vec<u8> = vec![0xDD; part_size];

    // UploadPart の presigned URL でパート 1 をアップロードする
    let presigned = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .part_number(1)
        .presigned(3600)
        .unwrap();
    assert_eq!(presigned.method, "PUT");
    let mut part1_presigned = presigned;
    part1_presigned.body = part1_data.clone();
    let response = execute_presigned(&part1_presigned).await;
    assert!(
        response.is_success(),
        "presigned UploadPart 1 failed: {}",
        response.status_code
    );
    let part1_etag = response.get_header("etag").map(String::from);

    // UploadPart の presigned URL でパート 2 をアップロードする
    let presigned = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .part_number(2)
        .presigned(3600)
        .unwrap();
    let mut part2_presigned = presigned;
    part2_presigned.body = part2_data.clone();
    let response = execute_presigned(&part2_presigned).await;
    assert!(
        response.is_success(),
        "presigned UploadPart 2 failed: {}",
        response.status_code
    );
    let part2_etag = response.get_header("etag").map(String::from);

    // CompleteMultipartUpload の presigned URL でアップロードを完了する
    let presigned = client
        .complete_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .multipart_upload(CompletedMultipartUpload {
            parts: Some(vec![
                CompletedPart {
                    e_tag: part1_etag,
                    part_number: Some(1),
                },
                CompletedPart {
                    e_tag: part2_etag,
                    part_number: Some(2),
                },
            ]),
        })
        .presigned(3600)
        .unwrap();
    assert_eq!(presigned.method, "POST");
    // CompleteMultipartUpload の presigned URL はボディに XML を含む
    assert!(!presigned.body.is_empty());
    let response = execute_presigned(&presigned).await;
    assert!(
        response.is_success(),
        "presigned CompleteMultipartUpload failed: {}",
        response.status_code
    );

    // GetObject で結合されたオブジェクトのボディを取得して確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let get_output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(get_output.body.len(), part_size * 2);
    assert_eq!(&get_output.body[..part_size], &part1_data[..]);
    assert_eq!(&get_output.body[part_size..], &part2_data[..]);
}

/// Presigned URL の有効期限バリデーションを検証する
///
/// ## 検証項目
/// - 0 秒は InvalidInput エラーになる
/// - 604801 秒 (7 日 + 1 秒) は InvalidInput エラーになる
/// - 1 秒と 604800 秒 (境界値) は成功する
#[tokio::test]
async fn test_presigned_expires_validation() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-presigned-expires";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // 0 秒はエラーになる
    let result = client
        .get_object()
        .bucket(bucket)
        .key("test.txt")
        .presigned(0);
    assert!(result.is_err());

    // 604801 秒 (7 日 + 1 秒) はエラーになる
    let result = client
        .get_object()
        .bucket(bucket)
        .key("test.txt")
        .presigned(604801);
    assert!(result.is_err());

    // 1 秒 (最小値) は成功する
    let result = client
        .get_object()
        .bucket(bucket)
        .key("test.txt")
        .presigned(1);
    assert!(result.is_ok());

    // 604800 秒 (7 日 = 最大値) は成功する
    let result = client
        .get_object()
        .bucket(bucket)
        .key("test.txt")
        .presigned(604800);
    assert!(result.is_ok());
}

/// Presigned URL によるマルチパートアップロードの中断を検証する
///
/// ## 検証項目
/// - AbortMultipartUpload の presigned URL でアップロードを中断できる
/// - 中断後にオブジェクトが存在しないことを確認する
#[tokio::test]
async fn test_presigned_abort_multipart_upload() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-presigned-abort";
    let key = "presigned-abort.bin";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // マルチパートアップロードを開始する
    let request = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let create_output = send(
        request,
        shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
    )
    .await;
    let upload_id = create_output.upload_id.expect("upload_id should exist");

    // AbortMultipartUpload の presigned URL でアップロードを中断する
    let presigned = client
        .abort_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .presigned(3600)
        .unwrap();
    assert_eq!(presigned.method, "DELETE");
    let response = execute_presigned(&presigned).await;
    assert!(
        response.is_success(),
        "presigned AbortMultipartUpload failed: {}",
        response.status_code
    );

    // 中断後はオブジェクトが存在しないことを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 404);
}

/// MinIO が PutPublicAccessBlock に対応していないことを検証する
///
/// MinIO latest イメージは PutPublicAccessBlock の XML パースに不具合があり
/// MalformedXML (400) を返す。aws-cli でも同じ結果になることを確認済み。
///
/// ## 検証項目
/// - PutPublicAccessBlock が 400 (MalformedXML) を返す
#[tokio::test]
async fn test_public_access_block_not_supported() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-public-access-block";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // MinIO は PutPublicAccessBlock の XML パースに不具合があり 400 を返す
    let request = client
        .put_public_access_block()
        .bucket(bucket)
        .block_public_acls(true)
        .ignore_public_acls(true)
        .block_public_policy(true)
        .restrict_public_buckets(true)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(
        response.status_code, 400,
        "MinIO should return 400 for PutPublicAccessBlock (known MinIO issue)"
    );
}

/// バケットポリシーの設定 / 取得 / 削除のラウンドトリップを検証する
///
/// バケットポリシーは IAM ポリシー形式の JSON 文字列で、
/// バケットへのアクセス制御を詳細に制御できる。
///
/// ## 検証項目
/// - PutBucketPolicy で JSON ポリシーを設定できる
/// - GetBucketPolicy でポリシーが取得できる
/// - DeleteBucketPolicy でポリシーを削除できる
/// - 削除後に GetBucketPolicy で 404 が返る
#[tokio::test]
async fn test_bucket_policy() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-bucket-policy";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // バケット内の全オブジェクトへの匿名読み取りを許可するポリシーを設定する
    // 実際の運用では Principal を "*" にするのは避けること
    let policy = format!(
        r#"{{"Version":"2012-10-17","Statement":[{{"Effect":"Allow","Principal":"*","Action":["s3:GetObject"],"Resource":["arn:aws:s3:::{bucket}/*"]}}]}}"#
    );
    let request = client
        .put_bucket_policy()
        .bucket(bucket)
        .policy(&policy)
        .build_request()
        .unwrap();
    send(request, PutBucketPolicyFluentBuilder::parse_response).await;

    // ポリシーを取得して存在することを確認する
    // レスポンスは JSON 文字列として返される
    let request = client
        .get_bucket_policy()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let output = send(request, GetBucketPolicyFluentBuilder::parse_response).await;
    assert!(output.policy.is_some());

    // ポリシーを削除する
    let request = client
        .delete_bucket_policy()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(request, DeleteBucketPolicyFluentBuilder::parse_response).await;

    // 削除後は GetBucketPolicy で 404 が返ることを確認する
    let request = client
        .get_bucket_policy()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 404);
}

/// 条件付きリクエストヘッダー (If-Match / If-None-Match) を検証する
///
/// ## 検証項目
/// - If-Match に正しい ETag を指定すると 200 が返る
/// - If-Match に不正な ETag を指定すると 412 Precondition Failed が返る
/// - If-None-Match に同じ ETag を指定すると 304 Not Modified が返る
/// - If-None-Match に異なる ETag を指定すると 200 が返る
#[tokio::test]
async fn test_conditional_headers_etag() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-conditional-etag";
    let key = "cond.txt";

    // テスト用バケットとオブジェクトを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"conditional".to_vec())
        .build_request()
        .unwrap();
    let put_output = send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;
    let etag = put_output.e_tag.expect("ETag should exist");

    // If-Match に正しい ETag を指定すると 200 が返る
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .if_match(&etag)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert!(response.is_success());

    // If-Match に不正な ETag を指定すると 412 が返る
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .if_match("\"invalid-etag\"")
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 412);

    // If-None-Match に同じ ETag を指定すると 304 が返る
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .if_none_match(&etag)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 304);

    // If-None-Match に異なる ETag を指定すると 200 が返る
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .if_none_match("\"different-etag\"")
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert!(response.is_success());
}

/// 条件付きリクエストヘッダー (If-Modified-Since / If-Unmodified-Since) を検証する
///
/// ## 検証項目
/// - If-Modified-Since に古い日時を指定すると 200 が返る
/// - If-Unmodified-Since に古い日時を指定すると 412 が返る
#[tokio::test]
async fn test_conditional_headers_date() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-conditional-date";
    let key = "cond-date.txt";

    // テスト用バケットとオブジェクトを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"date-conditional".to_vec())
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    // If-Modified-Since に十分古い日時を指定すると 200 が返る
    // (オブジェクトはその日時より後に作成されている)
    let old_date = HttpDate::from_unix_timestamp(0);
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .if_modified_since(old_date)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert!(response.is_success());

    // If-Unmodified-Since に十分古い日時を指定すると 412 が返る
    // (オブジェクトはその日時より後に変更されている)
    let old_date = HttpDate::from_unix_timestamp(0);
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .if_unmodified_since(old_date)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 412);
}

/// HeadObject の条件付きヘッダーを検証する
///
/// ## 検証項目
/// - HeadObject + If-Match で 200 / 412 が正しく返る
/// - HeadObject + If-None-Match で 304 が正しく返る
#[tokio::test]
async fn test_head_object_conditional_headers() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-head-conditional";
    let key = "head-cond.txt";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"head-conditional".to_vec())
        .build_request()
        .unwrap();
    let put_output = send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;
    let etag = put_output.e_tag.expect("ETag should exist");

    // If-Match に正しい ETag を指定すると 200 が返る
    let request = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .if_match(&etag)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert!(response.is_success());

    // If-None-Match に同じ ETag を指定すると 304 が返る
    let request = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .if_none_match(&etag)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 304);
}

/// Range リクエストによる部分取得を検証する
///
/// ## 検証項目
/// - Range ヘッダーで先頭 5 バイトのみ取得できる
/// - レスポンスステータスが 206 Partial Content である
/// - ボディが指定範囲のバイト列と一致する
#[tokio::test]
async fn test_range_request() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-range";
    let key = "range.txt";
    let body = b"0123456789ABCDEF";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body.to_vec())
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    // bytes=0-4 で先頭 5 バイトを取得する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .range("bytes=0-4")
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 206);
    assert_eq!(response.body, b"01234");

    // bytes=10-15 で末尾付近 6 バイトを取得する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .range("bytes=10-15")
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 206);
    assert_eq!(response.body, b"ABCDEF");
}

/// カスタムメタデータ (x-amz-meta-*) のラウンドトリップを検証する
///
/// ## 検証項目
/// - PutObject で設定したカスタムメタデータが GetObject / HeadObject で取得できる
/// - メタデータのキーと値が正しく往復する
#[tokio::test]
async fn test_custom_metadata() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-metadata";
    let key = "meta.txt";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // カスタムメタデータを 2 つ設定してオブジェクトをアップロードする
    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"with metadata".to_vec())
        .metadata("author", "test-user")
        .metadata("version", "42")
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    // GetObject でメタデータを取得して確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    let metadata = output.metadata.expect("metadata should exist");
    assert_eq!(
        metadata.get("author").map(String::as_str),
        Some("test-user")
    );
    assert_eq!(metadata.get("version").map(String::as_str), Some("42"));

    // HeadObject でもメタデータを取得して確認する
    let request = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::HeadObjectFluentBuilder::parse_response,
    )
    .await;
    let metadata = output.metadata.expect("metadata should exist");
    assert_eq!(
        metadata.get("author").map(String::as_str),
        Some("test-user")
    );
    assert_eq!(metadata.get("version").map(String::as_str), Some("42"));
}

/// System Metadata (Content-Encoding / Content-Disposition / Cache-Control / Expires) のラウンドトリップを検証する
///
/// ## 検証項目
/// - PutObject で設定した System Metadata が HeadObject で取得できる
#[tokio::test]
async fn test_system_metadata() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-sys-metadata";
    let key = "sys-meta.txt";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // System Metadata を設定してオブジェクトをアップロードする
    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"system metadata test".to_vec())
        .content_type("application/json")
        .content_disposition("attachment; filename=\"test.json\"")
        .cache_control("max-age=3600")
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    // HeadObject でレスポンスヘッダーを確認する
    let request = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert!(response.is_success());
    assert_eq!(
        response.get_header("content-type"),
        Some("application/json")
    );
    assert_eq!(
        response.get_header("content-disposition"),
        Some("attachment; filename=\"test.json\"")
    );
    assert_eq!(response.get_header("cache-control"), Some("max-age=3600"));
}

/// CopyObject の metadata_directive=REPLACE によるメタデータ置換を検証する
///
/// ## 検証項目
/// - REPLACE ディレクティブでコピー先のメタデータを上書きできる
/// - コピー先の Content-Type とカスタムメタデータがコピー元と異なる
#[tokio::test]
async fn test_copy_object_metadata_replace() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-copy-meta-replace";
    let src_key = "original.txt";
    let dst_key = "replaced.txt";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // コピー元を text/plain + メタデータ付きで作成する
    let request = client
        .put_object()
        .bucket(bucket)
        .key(src_key)
        .body(b"copy with replace".to_vec())
        .content_type("text/plain")
        .metadata("env", "original")
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    // metadata_directive=REPLACE でコピーし、メタデータを上書きする
    let copy_source = format!("{bucket}/{src_key}");
    let request = client
        .copy_object()
        .bucket(bucket)
        .key(dst_key)
        .copy_source(&copy_source)
        .metadata_directive("REPLACE")
        .content_type("application/octet-stream")
        .metadata("env", "replaced")
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::CopyObjectFluentBuilder::parse_response,
    )
    .await;
    assert!(output.e_tag.is_some());

    // コピー先のメタデータが上書きされていることを確認する
    let request = client
        .head_object()
        .bucket(bucket)
        .key(dst_key)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::HeadObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(
        output.content_type.as_deref(),
        Some("application/octet-stream")
    );
    let metadata = output.metadata.expect("metadata should exist");
    assert_eq!(metadata.get("env").map(String::as_str), Some("replaced"));
}

/// ListObjectsV2 のページネーション (max_keys / continuation_token) を検証する
///
/// ## 検証項目
/// - max_keys で 1 ページあたりの件数を制限できる
/// - is_truncated が true のときに next_continuation_token が返る
/// - continuation_token を使って次のページを取得できる
/// - 全ページを結合すると全件取得できる
#[tokio::test]
async fn test_list_objects_v2_pagination() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-list-pagination";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // 5 つのオブジェクトを作成する
    for i in 0..5 {
        let key = format!("page-{i:02}.txt");
        let request = client
            .put_object()
            .bucket(bucket)
            .key(&key)
            .body(format!("data-{i}").into_bytes())
            .build_request()
            .unwrap();
        send(
            request,
            shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
        )
        .await;
    }

    // max_keys=2 で 1 ページ目を取得する
    let request = client
        .list_objects_v2()
        .bucket(bucket)
        .max_keys(2)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
    )
    .await;
    let page1 = output.contents.expect("contents should exist");
    assert_eq!(page1.len(), 2);
    assert_eq!(output.is_truncated, Some(true));
    let token = output
        .next_continuation_token
        .expect("next_continuation_token should exist");

    // continuation_token で 2 ページ目を取得する
    let request = client
        .list_objects_v2()
        .bucket(bucket)
        .max_keys(2)
        .continuation_token(&token)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
    )
    .await;
    let page2 = output.contents.expect("contents should exist");
    assert_eq!(page2.len(), 2);
    assert_eq!(output.is_truncated, Some(true));
    let token = output
        .next_continuation_token
        .expect("next_continuation_token should exist");

    // 3 ページ目 (最終ページ) を取得する
    let request = client
        .list_objects_v2()
        .bucket(bucket)
        .max_keys(2)
        .continuation_token(&token)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
    )
    .await;
    let page3 = output.contents.expect("contents should exist");
    assert_eq!(page3.len(), 1);
    assert_eq!(output.is_truncated, Some(false));

    // 全ページを合わせると 5 件になることを確認する
    assert_eq!(page1.len() + page2.len() + page3.len(), 5);
}

/// ListObjectsV2 の start_after パラメータを検証する
///
/// ## 検証項目
/// - start_after で指定したキーより後のオブジェクトのみ返る
#[tokio::test]
async fn test_list_objects_v2_start_after() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-list-start-after";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // a.txt, b.txt, c.txt を作成する
    for key in ["a.txt", "b.txt", "c.txt"] {
        let request = client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(b"data".to_vec())
            .build_request()
            .unwrap();
        send(
            request,
            shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
        )
        .await;
    }

    // start_after="a.txt" で b.txt, c.txt のみ返ることを確認する
    let request = client
        .list_objects_v2()
        .bucket(bucket)
        .start_after("a.txt")
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::ListObjectsV2FluentBuilder::parse_response,
    )
    .await;
    let contents = output.contents.expect("contents should exist");
    assert_eq!(contents.len(), 2);
    assert_eq!(contents[0].key.as_deref(), Some("b.txt"));
    assert_eq!(contents[1].key.as_deref(), Some("c.txt"));
}

/// バージョニング有効時のオブジェクト操作を検証する
///
/// ## 検証項目
/// - バージョニング有効化後に PutObject すると version_id が返る
/// - 同一キーに上書きすると別の version_id が返る
/// - version_id を指定して GetObject で特定バージョンを取得できる
/// - version_id を指定して HeadObject で特定バージョンのメタデータを取得できる
/// - version_id を指定して DeleteObject で特定バージョンを削除できる
#[tokio::test]
async fn test_versioned_object_operations() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-versioned-ops";
    let key = "versioned.txt";

    // バケットを作成してバージョニングを有効化する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    let request = client
        .put_bucket_versioning()
        .bucket(bucket)
        .status("Enabled")
        .build_request()
        .unwrap();
    send(request, PutBucketVersioningFluentBuilder::parse_response).await;

    // バージョン 1 をアップロードする
    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"version-1".to_vec())
        .build_request()
        .unwrap();
    let put1 = send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;
    let version_id_1 = put1.version_id.expect("version_id should exist");

    // バージョン 2 をアップロードする (上書き)
    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"version-2".to_vec())
        .build_request()
        .unwrap();
    let put2 = send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;
    let version_id_2 = put2.version_id.expect("version_id should exist");

    // version_id が異なることを確認する
    assert_ne!(version_id_1, version_id_2);

    // version_id を指定してバージョン 1 を取得する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .version_id(&version_id_1)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.body, b"version-1");

    // version_id なしで取得すると最新バージョン (version-2) が返る
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.body, b"version-2");

    // version_id を指定して HeadObject でバージョン 1 のメタデータを取得する
    let request = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .version_id(&version_id_1)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::HeadObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.content_length, Some(b"version-1".len() as i64));

    // version_id を指定してバージョン 1 を削除する
    let request = client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .version_id(&version_id_1)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::DeleteObjectFluentBuilder::parse_response,
    )
    .await;

    // バージョン 1 が取得できなくなることを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .version_id(&version_id_1)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 404);

    // バージョン 2 はまだ取得できることを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.body, b"version-2");
}

/// チェックサムアルゴリズム指定によるアップロードを検証する
///
/// ## 検証項目
/// - CRC32C を指定して PutObject が成功する
/// - SHA256 を指定して PutObject が成功する
/// - デフォルト (CRC32) と同じくオブジェクトが正しく保存される
#[tokio::test]
async fn test_checksum_algorithm() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-checksum-algo";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // CRC32C アルゴリズムを指定してアップロードする
    let request = client
        .put_object()
        .bucket(bucket)
        .key("crc32c.txt")
        .body(b"crc32c-data".to_vec())
        .checksum_algorithm("CRC32C")
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;
    assert!(output.e_tag.is_some());

    // SHA256 アルゴリズムを指定してアップロードする
    let request = client
        .put_object()
        .bucket(bucket)
        .key("sha256.txt")
        .body(b"sha256-data".to_vec())
        .checksum_algorithm("SHA256")
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;
    assert!(output.e_tag.is_some());

    // 保存されたオブジェクトを取得してボディが正しいことを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key("crc32c.txt")
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.body, b"crc32c-data");

    let request = client
        .get_object()
        .bucket(bucket)
        .key("sha256.txt")
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.body, b"sha256-data");
}

/// MinIO が ListMultipartUploads の prefix / max_uploads フィルタを無視することを検証する
///
/// MinIO は ListMultipartUploads で prefix や max_uploads パラメータを
/// 指定しても無視して全件返す。パラメータ付きでリクエストしてもエラーに
/// ならないことだけを確認する。
///
/// ## 検証項目
/// - max_uploads を指定してもエラーにならない
/// - prefix を指定してもエラーにならない
#[tokio::test]
async fn test_list_multipart_uploads_filter_params() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-list-mpu-filter";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // マルチパートアップロードを 2 つ開始する
    let keys = ["file1.bin", "file2.bin"];
    let mut upload_ids = Vec::new();
    for key in &keys {
        let request = client
            .create_multipart_upload()
            .bucket(bucket)
            .key(*key)
            .build_request()
            .unwrap();
        let output = send(
            request,
            shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
        )
        .await;
        upload_ids.push(output.upload_id.expect("upload_id should exist"));
    }

    // max_uploads / prefix 付きでリクエストしてもエラーにならないことを確認する
    let request = client
        .list_multipart_uploads()
        .bucket(bucket)
        .max_uploads(1)
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert!(response.is_success());

    let request = client
        .list_multipart_uploads()
        .bucket(bucket)
        .prefix("file")
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert!(response.is_success());

    // クリーンアップ
    for (key, upload_id) in keys.iter().zip(upload_ids.iter()) {
        let request = client
            .abort_multipart_upload()
            .bucket(bucket)
            .key(*key)
            .upload_id(upload_id)
            .build_request()
            .unwrap();
        send(
            request,
            shiguredo_s3::api::AbortMultipartUploadFluentBuilder::parse_response,
        )
        .await;
    }
}

/// ListParts のページネーション (max_parts / part_number_marker) を検証する
///
/// ## 検証項目
/// - max_parts で取得件数を制限できる
/// - part_number_marker で指定したパート番号以降を取得できる
#[tokio::test]
async fn test_list_parts_pagination() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-list-parts-page";
    let key = "parts-page.bin";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // マルチパートアップロードを開始する
    let request = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let create_output = send(
        request,
        shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
    )
    .await;
    let upload_id = create_output.upload_id.expect("upload_id should exist");

    // 3 つのパート (各 5 MB) をアップロードする
    let part_size = 5 * 1024 * 1024;
    for part_number in 1..=3i32 {
        let data: Vec<u8> = vec![part_number as u8; part_size];
        let request = client
            .upload_part()
            .bucket(bucket)
            .key(key)
            .upload_id(&upload_id)
            .part_number(part_number)
            .body(data)
            .build_request()
            .unwrap();
        send(
            request,
            shiguredo_s3::api::UploadPartFluentBuilder::parse_response,
        )
        .await;
    }

    // max_parts=1 で 1 件のみ取得する
    let request = client
        .list_parts()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .max_parts(1)
        .build_request()
        .unwrap();
    let output = send(request, ListPartsFluentBuilder::parse_response).await;
    let parts = output.parts.expect("parts should exist");
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].part_number, Some(1));

    // part_number_marker=1 でパート 2 以降を取得する
    let request = client
        .list_parts()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .part_number_marker(1)
        .build_request()
        .unwrap();
    let output = send(request, ListPartsFluentBuilder::parse_response).await;
    let parts = output.parts.expect("parts should exist");
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].part_number, Some(2));
    assert_eq!(parts[1].part_number, Some(3));

    // クリーンアップ
    let request = client
        .abort_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::AbortMultipartUploadFluentBuilder::parse_response,
    )
    .await;
}

/// ACL 指定によるオブジェクトアップロードを検証する
///
/// ## 検証項目
/// - acl="private" を指定して PutObject が成功する
#[tokio::test]
async fn test_put_object_acl() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-put-acl";
    let key = "acl.txt";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // acl=private を指定してアップロードする
    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"private data".to_vec())
        .acl("private")
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;
    assert!(output.e_tag.is_some());

    // オブジェクトが正しく保存されていることを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.body, b"private data");
}

/// Storage Class 指定によるオブジェクトアップロードを検証する
///
/// ## 検証項目
/// - storage_class="STANDARD" を指定して PutObject が成功する
#[tokio::test]
async fn test_put_object_storage_class() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-storage-class";
    let key = "standard.txt";

    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // storage_class=STANDARD を指定してアップロードする
    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"standard class data".to_vec())
        .storage_class("STANDARD")
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;
    assert!(output.e_tag.is_some());

    // オブジェクトが正しく保存されていることを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.body, b"standard class data");

    // HEAD の x-amz-storage-class (STANDARD では省略されうる)
    let request = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let head_output = send(
        request,
        shiguredo_s3::api::HeadObjectFluentBuilder::parse_response,
    )
    .await;
    assert!(
        matches!(
            head_output.storage_class.as_deref(),
            None | Some("STANDARD")
        ),
        "unexpected storage_class: {:?}",
        head_output.storage_class
    );
}

/// オブジェクトタグの CRUD を検証する
///
/// ## 検証項目
/// - PutObjectTagging でタグを設定できる
/// - GetObjectTagging で設定したタグを取得できる
/// - DeleteObjectTagging でタグを削除できる
/// - 削除後は空のタグセットが返る
#[tokio::test]
async fn test_object_tagging() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-object-tagging";
    let key = "tagged-object.txt";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // オブジェクトを作成する
    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"hello".to_vec())
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    // タグを設定する
    let request = client
        .put_object_tagging()
        .bucket(bucket)
        .key(key)
        .tag(Tag {
            key: "env".to_string(),
            value: "test".to_string(),
        })
        .tag(Tag {
            key: "project".to_string(),
            value: "s3-rs".to_string(),
        })
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectTaggingFluentBuilder::parse_response,
    )
    .await;

    // タグを取得する
    let request = client
        .get_object_tagging()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectTaggingFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.tag_set.len(), 2);

    let mut tags: Vec<_> = output.tag_set.iter().map(|t| t.key.as_str()).collect();
    tags.sort();
    assert_eq!(tags, vec!["env", "project"]);

    // タグを削除する
    let request = client
        .delete_object_tagging()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::DeleteObjectTaggingFluentBuilder::parse_response,
    )
    .await;

    // 削除後は空のタグセットが返る
    let request = client
        .get_object_tagging()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectTaggingFluentBuilder::parse_response,
    )
    .await;
    assert!(output.tag_set.is_empty());
}

/// UploadPartCopy でサーバー側コピーを検証する
///
/// ## 検証項目
/// - UploadPartCopy でコピー元オブジェクトの全範囲をパートとしてコピーできる
/// - CompleteMultipartUpload で結合したオブジェクトの内容がコピー元と一致する
#[tokio::test]
async fn test_upload_part_copy() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-upload-part-copy";
    let src_key = "source.txt";
    let dst_key = "destination.txt";
    let body = b"upload part copy test data";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // コピー元オブジェクトを作成する
    let request = client
        .put_object()
        .bucket(bucket)
        .key(src_key)
        .body(body.to_vec())
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    // マルチパートアップロードを開始する
    let request = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(dst_key)
        .build_request()
        .unwrap();
    let create_output = send(
        request,
        shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
    )
    .await;
    let upload_id = create_output.upload_id.expect("upload_id should exist");

    // UploadPartCopy でコピー元からパートをコピーする
    let request = client
        .upload_part_copy()
        .bucket(bucket)
        .key(dst_key)
        .upload_id(&upload_id)
        .part_number(1)
        .copy_source(format!("{bucket}/{src_key}"))
        .build_request()
        .unwrap();
    let copy_output = send(
        request,
        shiguredo_s3::api::UploadPartCopyFluentBuilder::parse_response,
    )
    .await;
    assert!(copy_output.e_tag.is_some());

    // マルチパートアップロードを完了する
    let request = client
        .complete_multipart_upload()
        .bucket(bucket)
        .key(dst_key)
        .upload_id(&upload_id)
        .multipart_upload(shiguredo_s3::types::CompletedMultipartUpload {
            parts: Some(vec![shiguredo_s3::types::CompletedPart {
                e_tag: copy_output.e_tag,
                part_number: Some(1),
            }]),
        })
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CompleteMultipartUploadFluentBuilder::parse_response,
    )
    .await;

    // コピー先オブジェクトの内容を検証する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(dst_key)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetObjectFluentBuilder::parse_response,
    )
    .await;
    assert_eq!(output.body, body);
}

/// ListObjectVersions でバージョン一覧を取得する
///
/// ## 検証項目
/// - バージョニング有効バケットで複数バージョンが返る
/// - 削除後に削除マーカーが返る
#[tokio::test]
async fn test_list_object_versions() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-list-object-versions";
    let key = "versioned.txt";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // バージョニングを有効にする
    let request = client
        .put_bucket_versioning()
        .bucket(bucket)
        .status("Enabled")
        .build_request()
        .unwrap();
    send(request, PutBucketVersioningFluentBuilder::parse_response).await;

    // 2 つのバージョンを作成する
    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"v1".to_vec())
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    let request = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(b"v2".to_vec())
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    // オブジェクトを削除する (削除マーカーが作成される)
    let request = client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::DeleteObjectFluentBuilder::parse_response,
    )
    .await;

    // ListObjectVersions で全バージョンと削除マーカーを取得する
    let request = client
        .list_object_versions()
        .bucket(bucket)
        .build_request()
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::ListObjectVersionsFluentBuilder::parse_response,
    )
    .await;

    // 2 つのバージョンが返る
    let versions = output.versions.expect("versions should exist");
    assert_eq!(versions.len(), 2);

    // 削除マーカーが返る
    let delete_markers = output.delete_markers.expect("delete_markers should exist");
    assert_eq!(delete_markers.len(), 1);
    assert_eq!(delete_markers[0].key.as_deref(), Some(key));
}

/// Bucket CORS の操作を検証する
///
/// MinIO は PutBucketCors で Content-MD5 ヘッダーに対して 501 NotImplemented を返す。
/// リクエスト構築とエラーハンドリングが正しく動作することを検証する。
#[tokio::test]
async fn test_bucket_cors_not_supported() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-bucket-cors";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // MinIO は PutBucketCors を完全にサポートしていないため 501 が返る
    let request = client
        .put_bucket_cors()
        .bucket(bucket)
        .cors_rule(shiguredo_s3::types::CorsRule {
            allowed_origins: vec!["https://example.com".to_string()],
            allowed_methods: vec!["GET".to_string()],
            allowed_headers: vec![],
            max_age_seconds: None,
            expose_headers: vec![],
        })
        .build_request()
        .unwrap();
    let response = execute(request).await;
    assert!(
        response.status_code == 501 || response.status_code == 200,
        "unexpected status: {}",
        response.status_code
    );
}

/// Bucket Encryption の操作を検証する
///
/// MinIO は KMS が未設定の場合 PutBucketEncryption で 501 NotImplemented を返す。
/// リクエスト構築とエラーハンドリングが正しく動作することを検証する。
#[tokio::test]
async fn test_bucket_encryption() {
    let (_container, port) = start_minio().await;
    let client = build_client(port);
    let bucket = "test-bucket-encryption";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request()
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // MinIO は KMS 未設定では PutBucketEncryption で 501 を返す
    let request = client
        .put_bucket_encryption()
        .bucket(bucket)
        .rule(ServerSideEncryptionRule {
            apply_server_side_encryption_by_default: Some(ServerSideEncryptionByDefault {
                sse_algorithm: "AES256".to_string(),
                kms_master_key_id: None,
            }),
            bucket_key_enabled: None,
        })
        .build_request()
        .unwrap();
    let response = execute(request).await;
    // MinIO は KMS 未設定時に 501 を返すか、設定済みなら 200 を返す
    assert!(
        response.status_code == 501 || response.status_code == 200,
        "unexpected status: {}",
        response.status_code
    );

    // XML レスポンスパースのテスト（合成レスポンス）
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ServerSideEncryptionConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Rule>
    <ApplyServerSideEncryptionByDefault>
      <SSEAlgorithm>AES256</SSEAlgorithm>
    </ApplyServerSideEncryptionByDefault>
    <BucketKeyEnabled>true</BucketKeyEnabled>
  </Rule>
</ServerSideEncryptionConfiguration>"#;
    let synthetic_response = S3Response {
        status_code: 200,
        headers: vec![],
        body: xml.as_bytes().to_vec(),
    };
    let output = GetBucketEncryptionFluentBuilder::parse_response(&synthetic_response).unwrap();
    assert_eq!(output.rules.len(), 1);
    let rule = &output.rules[0];
    let default = rule
        .apply_server_side_encryption_by_default
        .as_ref()
        .expect("default encryption should exist");
    assert_eq!(default.sse_algorithm, "AES256");
    assert!(default.kms_master_key_id.is_none());
    assert_eq!(rule.bucket_key_enabled, Some(true));
}
