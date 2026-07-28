//! RustFS を使った統合テスト
//!
//! `shiguredo_container` で RustFS コンテナを起動し、全 API のラウンドトリップを検証する。
//! macOS では Apple container、Linux では Docker Engine が必要。
//!
//! ## テスト構成
//!
//! 各テストは独立した RustFS コンテナを起動するため、テスト間の状態汚染がない。
//! コンテナはガードの Drop 時に明示削除する。
//!
//! ## 起動待機について
//!
//! RustFS は /health エンドポイントで HTTP 200 が返った後も S3 API の初期化に
//! 若干の時間を要するため、ヘルスチェック通過後に 2 秒の追加待機を設けている。
//!
//! ## 既知の不具合 (RustFS 0.0.5)
//!
//! - ListMultipartUploads: 進行中アップロードが空リストで返る
//! - DeletePublicAccessBlock 後の GetPublicAccessBlock: 404 ではなく 500 が返る

use shiguredo_container::core::IntoContainerPort;
use shiguredo_container::core::wait::HttpWaitStrategy;
use shiguredo_container::{AsyncRunner, ContainerAsync, GenericImage, ImageExt, WaitFor};
use shiguredo_http11::{HeaderName, HttpHead, Method, ResponseDecoder};
use shiguredo_s3::api::{
    DeleteBucketCorsFluentBuilder, DeleteBucketEncryptionFluentBuilder,
    DeleteBucketPolicyFluentBuilder, DeleteBucketTaggingFluentBuilder,
    DeleteObjectTaggingFluentBuilder, DeletePublicAccessBlockFluentBuilder,
    GetBucketCorsFluentBuilder, GetBucketPolicyFluentBuilder, GetBucketTaggingFluentBuilder,
    GetBucketVersioningFluentBuilder, GetObjectTaggingFluentBuilder,
    GetPublicAccessBlockFluentBuilder, ListMultipartUploadsFluentBuilder, ListPartsFluentBuilder,
    PutBucketCorsFluentBuilder, PutBucketEncryptionFluentBuilder, PutBucketPolicyFluentBuilder,
    PutBucketTaggingFluentBuilder, PutBucketVersioningFluentBuilder, PutObjectTaggingFluentBuilder,
    PutPublicAccessBlockFluentBuilder,
};
use shiguredo_s3::types::{
    CompletedMultipartUpload, CompletedPart, CorsConfiguration, CorsRule, ObjectIdentifier,
    ServerSideEncryptionByDefault, ServerSideEncryptionConfiguration, ServerSideEncryptionRule,
    Tag, Tagging,
};
use shiguredo_s3::{Client, Config, Credentials, S3Request, S3Response};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// -------------------------------------------------------
// テスト用定数
// -------------------------------------------------------

/// RustFS のアクセスキー
/// rustfs/rustfs リポジトリの docker-compose.yml で使用されている値に合わせている
const ACCESS_KEY: &str = "devadmin";

/// RustFS のシークレットキー
/// rustfs/rustfs リポジトリの docker-compose.yml で使用されている値に合わせている
const SECRET_KEY: &str = "devadmin";

// -------------------------------------------------------
// テスト用ヘルパー
// -------------------------------------------------------

/// コンテナランタイム向けに、指定 ID のコンテナを同期的に削除する。
///
/// `ContainerAsync` の Drop は tokio Runtime 内だと削除スレッドを join しないため、
/// 連続テストで孤立コンテナが溜まり得る。ガードの Drop から明示的に掃除する。
fn force_remove_container(id: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("container")
            .args(["stop", id])
            .output();
        let _ = std::process::Command::new("container")
            .args(["rm", id])
            .output();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("docker")
            .args(["rm", "-f", id])
            .output();
    }
}

/// RustFS コンテナの生存期間をテスト中に保つためのガード。
struct RustfsGuard {
    container: Option<ContainerAsync<GenericImage>>,
    host: String,
    port: u16,
}

impl Drop for RustfsGuard {
    fn drop(&mut self) {
        if let Some(container) = self.container.take() {
            let id = container.id().to_string();
            drop(container);
            force_remove_container(&id);
        }
    }
}

/// RustFS コンテナを起動してガードを返す。
///
/// コンテナのポート 9000 をホストのランダムポートにマッピングし、
/// /health エンドポイントが HTTP 200 を返すまで待機する。
/// RustFS はログをファイル (/logs) に書き込むため、stdout/stderr では
/// 起動完了を検知できないので HTTP ポーリングを使用する。
///
/// ## 追加待機について
/// ヘルスチェック通過直後は S3 API がまだ初期化中のことがあるため、
/// 2 秒の追加待機を設けて安定性を確保する。
async fn start_rustfs() -> RustfsGuard {
    let container = GenericImage::new("rustfs/rustfs", "latest")
        .with_exposed_port(9000.tcp())
        // /health が 200 を返すまでポーリングする
        .with_wait_for(WaitFor::http(
            HttpWaitStrategy::new("/health")
                .with_port(9000.tcp())
                .with_expected_status_code(200_u16),
        ))
        // ヘルスチェック通過後も S3 API の初期化に時間がかかるため追加待機する
        .with_wait_for(WaitFor::seconds(2))
        .with_env_var("RUSTFS_ACCESS_KEY", ACCESS_KEY)
        .with_env_var("RUSTFS_SECRET_KEY", SECRET_KEY)
        // データを保存するボリュームパスを指定する
        .with_env_var("RUSTFS_VOLUMES", "/data")
        .start()
        .await
        .expect("RustFS コンテナの起動に成功すること");

    let host = container
        .get_host()
        .await
        .expect("コンテナのホスト名の取得に成功すること")
        .to_string();
    let port = container
        .get_host_port_ipv4(9000)
        .await
        .expect("コンテナの 9000 ポート番号の取得に成功すること");

    RustfsGuard {
        container: Some(container),
        host,
        port,
    }
}

/// `SystemTime::now()` をテスト本体から呼び出すためのヘルパ
///
/// shiguredo_s3 は Sans I/O のため `build_request` / `presigned` に
/// 現在時刻を引数で渡す。テスト側で副作用を 1 箇所に閉じ込める目的。
fn now() -> std::time::SystemTime {
    std::time::SystemTime::now()
}

/// テスト用の Client を構築する
///
/// - region: us-east-1 (CreateBucket で LocationConstraint を省略できる)
/// - endpoint: コンテナのホストポートに接続する HTTP エンドポイント
/// - force_path_style: true (RustFS はパススタイルが必要)
fn build_client(host: &str, port: u16) -> Client {
    let config = Config::builder()
        .region("us-east-1")
        .credentials_provider(Credentials::new(ACCESS_KEY, SECRET_KEY, None, None, "test"))
        .endpoint(format!("http://{host}:{port}"))
        // RustFS は仮想ホストスタイルに対応していないためパススタイルを使う
        .force_path_style(true)
        .build()
        .expect("Config の構築に成功すること");
    Client::from_conf(config)
}

/// S3Request を HTTP/1.1 で送信して S3Response を返す
///
/// shiguredo_http11 の ResponseDecoder を使って TCP ストリームからレスポンスを読む。
/// HEAD レスポンスのようにボディを持たないレスポンスは
/// `ResponseDecoder::set_request_method` でリクエストメソッドを通知する。
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
    decoder.set_request_method(&s3_request.method);

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
    let method = Method::new(&s3_request.method).expect("failed to parse method");
    let mut request =
        shiguredo_http11::Request::new(method, &s3_request.uri).expect("failed to build request");
    for (name, value) in &s3_request.headers {
        let header_name = HeaderName::new(name).expect("failed to parse header name");
        request
            .add_header(header_name, value)
            .expect("failed to add header");
    }
    if !s3_request.body.is_empty() {
        request.set_body(s3_request.body.clone());
    }
    request.encode().expect("failed to encode request")
}

/// shiguredo_http11 の Response を S3Response に変換する
fn into_s3_response(response: shiguredo_http11::Response) -> S3Response {
    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(name, value)| (name.to_string(), value.clone()))
        .collect();
    S3Response {
        status_code: response.status_code(),
        headers,
        body: response.body_bytes().unwrap_or_default().to_vec(),
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

/// バケットの作成・確認・削除のライフサイクルを検証する
///
/// ## 検証項目
/// - CreateBucket でバケットを作成できる
/// - HeadBucket で存在確認できる
/// - ListBuckets で作成したバケットが含まれる
/// - DeleteBucket でバケットを削除できる
/// - 削除後に ListBuckets でバケットが消えていることを確認できる
#[tokio::test]
async fn test_bucket_lifecycle() {
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-bucket-lifecycle";

    // バケットを作成する
    // us-east-1 では LocationConstraint が不要なためボディは空
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let _output = send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // バケットが存在することを HEAD で確認する
    // 存在しない場合は 404 が返る
    let request = client
        .head_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let _output = send(
        request,
        shiguredo_s3::api::HeadBucketFluentBuilder::parse_response,
    )
    .await;

    // ListBuckets で作成したバケットが一覧に含まれていることを確認する
    let request = client.list_buckets().build_request(now()).unwrap();
    let output = send(
        request,
        shiguredo_s3::api::ListBucketsFluentBuilder::parse_response,
    )
    .await;
    assert!(
        output
            .buckets
            .iter()
            .any(|b| b.name.as_deref() == Some(bucket)),
        "bucket not found in list"
    );

    // バケットを削除する
    // バケット内にオブジェクトが残っている場合は BucketNotEmpty エラーになる
    let request = client
        .delete_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let _output = send(
        request,
        shiguredo_s3::api::DeleteBucketFluentBuilder::parse_response,
    )
    .await;

    // 削除後に ListBuckets でバケットが消えていることを確認する
    let request = client.list_buckets().build_request(now()).unwrap();
    let output = send(
        request,
        shiguredo_s3::api::ListBucketsFluentBuilder::parse_response,
    )
    .await;
    assert!(
        !output
            .buckets
            .iter()
            .any(|b| b.name.as_deref() == Some(bucket)),
        "bucket should not exist after delete"
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
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-object-crud";
    let key = "hello.txt";
    let body = b"Hello, S3!";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
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
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-list-objects";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
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
            .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
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
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-copy-object";
    let src_key = "original.txt";
    let dst_key = "copied.txt";
    let body = b"copy me";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
        .unwrap();
    let copy_output = send(
        request,
        shiguredo_s3::api::CopyObjectFluentBuilder::parse_response,
    )
    .await;
    // コピー成功時は ETag が返る
    assert!(
        copy_output
            .copy_object_result
            .as_ref()
            .and_then(|r| r.e_tag.as_ref())
            .is_some()
    );

    // コピー先を GetObject で取得してボディが一致することを確認する
    let request = client
        .get_object()
        .bucket(bucket)
        .key(dst_key)
        .build_request(now())
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
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-delete-objects";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
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
            .build_request(now())
            .unwrap();
        send(
            request,
            shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
        )
        .await;
    }

    // DeleteObjects で 3 つのオブジェクトを一括削除する
    // DeleteObject (単体) を 3 回呼ぶより効率的
    let delete = shiguredo_s3::Delete::builder()
        .set_objects(
            keys.iter()
                .map(|k| ObjectIdentifier {
                    key: k.clone(),
                    version_id: None,
                    e_tag: None,
                    last_modified_time: None,
                    size: None,
                })
                .collect(),
        )
        .build();
    let request = client
        .delete_objects()
        .bucket(bucket)
        .delete(delete)
        .build_request(now())
        .unwrap();
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
        .build_request(now())
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
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-multipart";
    let key = "large.bin";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
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
                    checksum_crc32: None,
                    checksum_crc32_c: None,
                    checksum_crc64_nvme: None,
                    checksum_sha1: None,
                    checksum_sha256: None,
                },
                CompletedPart {
                    e_tag: part2_output.e_tag,
                    part_number: Some(2),
                    checksum_crc32: None,
                    checksum_crc32_c: None,
                    checksum_crc64_nvme: None,
                    checksum_sha1: None,
                    checksum_sha256: None,
                },
            ]),
        })
        .build_request(now())
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
        .build_request(now())
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

/// checksum 付きマルチパートアップロードの完了を検証する
///
/// CreateMultipartUpload で checksum アルゴリズムを指定し、
/// UploadPart で各パートの checksum を取得して CompleteMultipartUpload に渡すと、
/// 完了レスポンスの XML ボディにオブジェクト全体の checksum が含まれる。
///
/// ## 検証項目
/// - UploadPart のレスポンスヘッダーにパートの checksum が含まれる
/// - CompleteMultipartUpload のレスポンス XML に checksum が含まれる
#[tokio::test]
async fn test_multipart_upload_with_checksum() {
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-mpu-checksum";
    let key = "checksum.bin";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // CRC32 checksum アルゴリズムを指定してマルチパートアップロードを開始する
    let request = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .checksum_algorithm(shiguredo_s3::ChecksumAlgorithm::Crc32)
        .build_request(now())
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

    // パート 1 を checksum アルゴリズム付きでアップロードする
    let request = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .part_number(1)
        .body(part1_data)
        .checksum_algorithm(shiguredo_s3::ChecksumAlgorithm::Crc32)
        .build_request(now())
        .unwrap();
    let part1_output = send(
        request,
        shiguredo_s3::api::UploadPartFluentBuilder::parse_response,
    )
    .await;
    // UploadPart のレスポンスヘッダーに checksum が含まれる
    let part1_crc32 = part1_output
        .checksum_crc32
        .expect("part1 checksum_crc32 should exist");

    // パート 2 を checksum アルゴリズム付きでアップロードする
    let request = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .part_number(2)
        .body(part2_data)
        .checksum_algorithm(shiguredo_s3::ChecksumAlgorithm::Crc32)
        .build_request(now())
        .unwrap();
    let part2_output = send(
        request,
        shiguredo_s3::api::UploadPartFluentBuilder::parse_response,
    )
    .await;
    let part2_crc32 = part2_output
        .checksum_crc32
        .expect("part2 checksum_crc32 should exist");

    // 各パートの checksum を渡してアップロードを完了する
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
                    checksum_crc32: Some(part1_crc32),
                    checksum_crc32_c: None,
                    checksum_crc64_nvme: None,
                    checksum_sha1: None,
                    checksum_sha256: None,
                },
                CompletedPart {
                    e_tag: part2_output.e_tag,
                    part_number: Some(2),
                    checksum_crc32: Some(part2_crc32),
                    checksum_crc32_c: None,
                    checksum_crc64_nvme: None,
                    checksum_sha1: None,
                    checksum_sha256: None,
                },
            ]),
        })
        .build_request(now())
        .unwrap();
    let complete_output = send(
        request,
        shiguredo_s3::api::CompleteMultipartUploadFluentBuilder::parse_response,
    )
    .await;
    // 完了後は結合オブジェクト全体の ETag が返る
    assert!(complete_output.e_tag.is_some());
    // S3 仕様では checksum は XML ボディに含まれる
    // checksum 付き MPU の完了レスポンスに checksum が含まれることを検証する
    assert!(
        complete_output.checksum_crc32.is_some(),
        "checksum_crc32 should be present in CompleteMultipartUpload response XML"
    );
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
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-abort-multipart";
    let key = "aborted.bin";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
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
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-list-parts";
    let key = "multipart.bin";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
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
        .build_request(now())
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
            .build_request(now())
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
        .build_request(now())
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
        .build_request(now())
        .unwrap();
    send(
        request,
        shiguredo_s3::api::AbortMultipartUploadFluentBuilder::parse_response,
    )
    .await;
}

/// RustFS が ListMultipartUploads で進行中アップロードを返さないことを検証する
///
/// ## 既知の不具合 (RustFS 0.0.5)
/// RustFS 0.0.5 は ListMultipartUploads で進行中アップロードを空リストで返す。
/// aws-cli でも同様の結果になることを確認済み。
///
/// ## 検証項目
/// - ListMultipartUploads が進行中のアップロードを返さない (None)
#[tokio::test]
async fn test_list_multipart_uploads_not_supported() {
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-list-mpu";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // 2 つのマルチパートアップロードを開始する
    let keys = ["upload-a.bin", "upload-b.bin"];
    let mut upload_ids = Vec::new();
    for key in &keys {
        let request = client
            .create_multipart_upload()
            .bucket(bucket)
            .key(*key)
            .build_request(now())
            .unwrap();
        let output = send(
            request,
            shiguredo_s3::api::CreateMultipartUploadFluentBuilder::parse_response,
        )
        .await;
        upload_ids.push(output.upload_id.expect("upload_id should exist"));
    }

    // RustFS は進行中アップロードを空リストで返す
    let request = client
        .list_multipart_uploads()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let output = send(request, ListMultipartUploadsFluentBuilder::parse_response).await;
    assert!(
        output.uploads.is_none(),
        "RustFS should return empty uploads (known RustFS 0.0.5 issue)"
    );

    // クリーンアップ
    for (key, upload_id) in keys.iter().zip(upload_ids.iter()) {
        let request = client
            .abort_multipart_upload()
            .bucket(bucket)
            .key(*key)
            .upload_id(upload_id)
            .build_request(now())
            .unwrap();
        send(
            request,
            shiguredo_s3::api::AbortMultipartUploadFluentBuilder::parse_response,
        )
        .await;
    }
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
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-versioning";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
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
        .build_request(now())
        .unwrap();
    let output = send(request, GetBucketVersioningFluentBuilder::parse_response).await;
    assert!(output.status.is_none());

    // バージョニングを有効化する
    let request = client
        .put_bucket_versioning()
        .bucket(bucket)
        .status("Enabled")
        .build_request(now())
        .unwrap();
    send(request, PutBucketVersioningFluentBuilder::parse_response).await;

    // Status が "Enabled" になっていることを確認する
    let request = client
        .get_bucket_versioning()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let output = send(request, GetBucketVersioningFluentBuilder::parse_response).await;
    assert_eq!(output.status.as_deref(), Some("Enabled"));

    // バージョニングを停止する (Disabled には戻せないため Suspended を使う)
    let request = client
        .put_bucket_versioning()
        .bucket(bucket)
        .status("Suspended")
        .build_request(now())
        .unwrap();
    send(request, PutBucketVersioningFluentBuilder::parse_response).await;

    // Status が "Suspended" になっていることを確認する
    let request = client
        .get_bucket_versioning()
        .bucket(bucket)
        .build_request(now())
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
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-bucket-tagging";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
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
        .tagging(
            Tagging::builder()
                .tag_set(Tag {
                    key: "env".to_string(),
                    value: "test".to_string(),
                })
                .tag_set(Tag {
                    key: "project".to_string(),
                    value: "s3-rs".to_string(),
                })
                .build(),
        )
        .build_request(now())
        .unwrap();
    send(request, PutBucketTaggingFluentBuilder::parse_response).await;

    // タグを取得して設定した内容が含まれることを確認する
    let request = client
        .get_bucket_tagging()
        .bucket(bucket)
        .build_request(now())
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
        .build_request(now())
        .unwrap();
    send(request, DeleteBucketTaggingFluentBuilder::parse_response).await;

    // 削除後は GetBucketTagging で 404 が返ることを確認する
    let request = client
        .get_bucket_tagging()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 404);
}

/// オブジェクトタグの設定・取得・削除のラウンドトリップを検証する
#[tokio::test]
async fn test_object_tagging() {
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-object-tagging";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // テスト用オブジェクトを作成する
    let request = client
        .put_object()
        .bucket(bucket)
        .key("test.txt")
        .body(b"hello".to_vec())
        .build_request(now())
        .unwrap();
    send(
        request,
        shiguredo_s3::api::PutObjectFluentBuilder::parse_response,
    )
    .await;

    // タグ未設定の状態では空のタグセットが返ることを確認する
    let request = client
        .get_object_tagging()
        .bucket(bucket)
        .key("test.txt")
        .build_request(now())
        .unwrap();
    let output = send(request, GetObjectTaggingFluentBuilder::parse_response).await;
    assert!(output.tag_set.is_empty());

    // 2 つのタグを設定する
    let request = client
        .put_object_tagging()
        .bucket(bucket)
        .key("test.txt")
        .tagging(
            Tagging::builder()
                .tag_set(Tag {
                    key: "env".to_string(),
                    value: "staging".to_string(),
                })
                .tag_set(Tag {
                    key: "team".to_string(),
                    value: "backend".to_string(),
                })
                .build(),
        )
        .build_request(now())
        .unwrap();
    send(request, PutObjectTaggingFluentBuilder::parse_response).await;

    // タグを取得して設定した内容が含まれることを確認する
    let request = client
        .get_object_tagging()
        .bucket(bucket)
        .key("test.txt")
        .build_request(now())
        .unwrap();
    let output = send(request, GetObjectTaggingFluentBuilder::parse_response).await;
    assert_eq!(output.tag_set.len(), 2);
    assert!(
        output
            .tag_set
            .iter()
            .any(|t| t.key == "env" && t.value == "staging")
    );
    assert!(
        output
            .tag_set
            .iter()
            .any(|t| t.key == "team" && t.value == "backend")
    );

    // タグを全削除する
    let request = client
        .delete_object_tagging()
        .bucket(bucket)
        .key("test.txt")
        .build_request(now())
        .unwrap();
    send(request, DeleteObjectTaggingFluentBuilder::parse_response).await;

    // 削除後は空のタグセットが返ることを確認する
    let request = client
        .get_object_tagging()
        .bucket(bucket)
        .key("test.txt")
        .build_request(now())
        .unwrap();
    let output = send(request, GetObjectTaggingFluentBuilder::parse_response).await;
    assert!(output.tag_set.is_empty());
}

/// パブリックアクセスブロック設定の設定 / 取得 / 削除のラウンドトリップを検証する
///
/// パブリックアクセスブロックはバケットへの公開アクセスを制限するセキュリティ機能。
/// MinIO とは異なり、RustFS 0.0.5 では PutPublicAccessBlock が正常に動作する。
///
/// ## 検証項目
/// - PutPublicAccessBlock で全フィールドを true に設定できる
/// - GetPublicAccessBlock で設定値が正しく取得できる
/// - DeletePublicAccessBlock で設定を削除できる
#[tokio::test]
async fn test_public_access_block() {
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-public-access-block";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // 全フィールドを true にしてパブリックアクセスブロックを設定する
    // RustFS 0.0.5 では MinIO と異なり PutPublicAccessBlock が正常に動作する
    let request = client
        .put_public_access_block()
        .bucket(bucket)
        .block_public_acls(true)
        .ignore_public_acls(true)
        .block_public_policy(true)
        .restrict_public_buckets(true)
        .build_request(now())
        .unwrap();
    send(request, PutPublicAccessBlockFluentBuilder::parse_response).await;

    // 設定を取得して全フィールドが true であることを確認する
    let request = client
        .get_public_access_block()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let output = send(request, GetPublicAccessBlockFluentBuilder::parse_response).await;
    assert_eq!(output.block_public_acls, Some(true));
    assert_eq!(output.ignore_public_acls, Some(true));
    assert_eq!(output.block_public_policy, Some(true));
    assert_eq!(output.restrict_public_buckets, Some(true));

    // 設定を削除する
    let request = client
        .delete_public_access_block()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(
        request,
        DeletePublicAccessBlockFluentBuilder::parse_response,
    )
    .await;
}

/// RustFS が DeletePublicAccessBlock 後の GET で 500 を返すことを検証する
///
/// ## 既知の不具合 (RustFS 0.0.5)
/// DeletePublicAccessBlock 後に GetPublicAccessBlock を実行すると、
/// 期待される 404 ではなく 500 が返る。aws-cli では
/// NoSuchPublicAccessBlockConfiguration (404 相当) が返ることを確認済み。
///
/// ## 検証項目
/// - DeletePublicAccessBlock 後の GetPublicAccessBlock が 500 を返す
#[tokio::test]
async fn test_delete_public_access_block_returns_500() {
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-pab-delete-bug";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // パブリックアクセスブロックを設定して削除する
    let request = client
        .put_public_access_block()
        .bucket(bucket)
        .block_public_acls(true)
        .build_request(now())
        .unwrap();
    send(request, PutPublicAccessBlockFluentBuilder::parse_response).await;

    let request = client
        .delete_public_access_block()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(
        request,
        DeletePublicAccessBlockFluentBuilder::parse_response,
    )
    .await;

    // RustFS は削除後の GET で 404 ではなく 500 を返す
    let request = client
        .get_public_access_block()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let response = execute(request).await;
    assert_eq!(
        response.status_code, 500,
        "RustFS should return 500 after DeletePublicAccessBlock (known RustFS 0.0.5 issue)"
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
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-bucket-policy";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
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
        .build_request(now())
        .unwrap();
    send(request, PutBucketPolicyFluentBuilder::parse_response).await;

    // ポリシーを取得して存在することを確認する
    // レスポンスは JSON 文字列として返される
    let request = client
        .get_bucket_policy()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let output = send(request, GetBucketPolicyFluentBuilder::parse_response).await;
    assert!(output.policy.is_some());

    // ポリシーを削除する
    let request = client
        .delete_bucket_policy()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(request, DeleteBucketPolicyFluentBuilder::parse_response).await;

    // 削除後は GetBucketPolicy で 404 が返ることを確認する
    let request = client
        .get_bucket_policy()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let response = execute(request).await;
    assert_eq!(response.status_code, 404);
}

/// バケットライフサイクル設定の Put / Get / Delete を検証する
///
/// ## 検証項目
/// - PutBucketLifecycleConfiguration でルールを設定できる
/// - GetBucketLifecycleConfiguration で設定したルールを取得できる
/// - DeleteBucketLifecycle でルールを削除できる
/// - 削除後に Get するとエラーになる
#[tokio::test]
async fn test_bucket_lifecycle_configuration() {
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-lifecycle-config";

    // バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let _output = send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // ライフサイクルルールを設定する
    let rule = shiguredo_s3::types::LifecycleRule {
        id: Some("expire-after-30-days".to_string()),
        status: shiguredo_s3::types::ExpirationStatus::Enabled,
        filter: Some(shiguredo_s3::types::LifecycleRuleFilter {
            prefix: Some("logs/".to_string()),
            ..Default::default()
        }),
        expiration: Some(shiguredo_s3::types::LifecycleExpiration {
            days: Some(30),
            ..Default::default()
        }),
        transitions: None,
        noncurrent_version_transitions: None,
        noncurrent_version_expiration: None,
        abort_incomplete_multipart_upload: None,
    };

    let request = client
        .put_bucket_lifecycle_configuration()
        .bucket(bucket)
        .rule(rule)
        .build_request(now())
        .unwrap();
    let _output = send(
        request,
        shiguredo_s3::api::PutBucketLifecycleConfigurationFluentBuilder::parse_response,
    )
    .await;

    // 設定したルールを取得する
    let request = client
        .get_bucket_lifecycle_configuration()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetBucketLifecycleConfigurationFluentBuilder::parse_response,
    )
    .await;

    assert_eq!(output.rules.len(), 1);
    assert_eq!(output.rules[0].id.as_deref(), Some("expire-after-30-days"));
    assert_eq!(
        output.rules[0].status,
        shiguredo_s3::types::ExpirationStatus::Enabled
    );
    assert_eq!(output.rules[0].expiration.as_ref().unwrap().days, Some(30));

    // ライフサイクル設定を削除する
    let request = client
        .delete_bucket_lifecycle()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let _output = send(
        request,
        shiguredo_s3::api::DeleteBucketLifecycleFluentBuilder::parse_response,
    )
    .await;

    // 削除後に Get するとエラーになることを確認する
    let request = client
        .get_bucket_lifecycle_configuration()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let response = execute(request).await;
    let result =
        shiguredo_s3::api::GetBucketLifecycleConfigurationFluentBuilder::parse_response(&response);
    assert!(result.is_err(), "lifecycle should not exist after delete");
}

/// バケット暗号化設定の Put → Get → Delete のラウンドトリップを検証する
#[tokio::test]
async fn test_bucket_encryption() {
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-bucket-encryption";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // SSE-S3 (AES256) を設定する
    let request = client
        .put_bucket_encryption()
        .bucket(bucket)
        .server_side_encryption_configuration(
            ServerSideEncryptionConfiguration::builder()
                .rules(
                    ServerSideEncryptionRule::builder()
                        .apply_server_side_encryption_by_default(
                            ServerSideEncryptionByDefault::builder()
                                .sse_algorithm("AES256")
                                .build(),
                        )
                        .bucket_key_enabled(false)
                        .build(),
                )
                .build(),
        )
        .build_request(now())
        .unwrap();
    send(request, PutBucketEncryptionFluentBuilder::parse_response).await;

    // 暗号化設定を取得して AES256 が返ることを確認する
    let request = client
        .get_bucket_encryption()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let output = send(
        request,
        shiguredo_s3::api::GetBucketEncryptionFluentBuilder::parse_response,
    )
    .await;
    let config = output.server_side_encryption_configuration.unwrap();
    assert_eq!(config.rules.len(), 1);
    let rule = &config.rules[0];
    let by_default = rule
        .apply_server_side_encryption_by_default
        .as_ref()
        .unwrap();
    assert_eq!(by_default.sse_algorithm, "AES256");
    assert!(by_default.kms_master_key_id.is_none());

    // 暗号化設定を削除する
    let request = client
        .delete_bucket_encryption()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(request, DeleteBucketEncryptionFluentBuilder::parse_response).await;

    // 削除後は暗号化設定が存在しないことを確認する
    let request = client
        .get_bucket_encryption()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let response = execute(request).await;
    // RustFS は削除後 400 を返す (MinIO は 404)
    assert!(
        response.status_code == 400 || response.status_code == 404,
        "expected 400 or 404, got {}",
        response.status_code
    );
}

/// バケット CORS 設定の Put → Get → Delete のラウンドトリップを検証する
#[tokio::test]
async fn test_bucket_cors() {
    let guard = start_rustfs().await;
    let client = build_client(&guard.host, guard.port);
    let bucket = "test-bucket-cors";

    // テスト用バケットを作成する
    let request = client
        .create_bucket()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(
        request,
        shiguredo_s3::api::CreateBucketFluentBuilder::parse_response,
    )
    .await;

    // CORS 設定を行う
    let request = client
        .put_bucket_cors()
        .bucket(bucket)
        .cors_configuration(
            CorsConfiguration::builder()
                .cors_rules(
                    CorsRule::builder()
                        .allowed_methods("GET")
                        .allowed_methods("PUT")
                        .allowed_origins("https://example.com")
                        .allowed_headers("*")
                        .expose_headers("x-amz-request-id")
                        .max_age_seconds(3600)
                        .build()
                        .expect("CorsRule build failed"),
                )
                .build(),
        )
        .build_request(now())
        .unwrap();
    send(request, PutBucketCorsFluentBuilder::parse_response).await;

    // CORS 設定を取得して検証する
    let request = client
        .get_bucket_cors()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let output = send(request, GetBucketCorsFluentBuilder::parse_response).await;
    let rules = output.cors_rules.unwrap();
    assert_eq!(rules.len(), 1);
    let rule = &rules[0];
    assert_eq!(rule.allowed_methods, vec!["GET", "PUT"]);
    assert_eq!(rule.allowed_origins, vec!["https://example.com"]);
    assert_eq!(
        rule.allowed_headers.as_ref().unwrap(),
        &vec!["*".to_string()]
    );
    assert_eq!(
        rule.expose_headers.as_ref().unwrap(),
        &vec!["x-amz-request-id".to_string()]
    );
    assert_eq!(rule.max_age_seconds, Some(3600));

    // CORS 設定を削除する
    let request = client
        .delete_bucket_cors()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    send(request, DeleteBucketCorsFluentBuilder::parse_response).await;

    // 削除後は GetBucketCors でエラーが返ることを確認する
    let request = client
        .get_bucket_cors()
        .bucket(bucket)
        .build_request(now())
        .unwrap();
    let response = execute(request).await;
    assert!(
        response.status_code == 404 || response.status_code == 400,
        "expected 404 or 400, got {}",
        response.status_code
    );
}
