//! 統合テスト共通ヘルパー
//!
//! MinIO / RustFS / kikyo-local の各統合テストで共有する HTTP/1.1 送信層と
//! テスト用 Client 構築ヘルパーを提供する。
//!
//! コンテナ固有の起動処理 (`start_rustfs` 等) とアクセスキー / シークレットキーは
//! 各テストファイルに置き、このモジュールにはサーバーに依存しない共通処理のみを置く。

use shiguredo_http11::{HeaderName, HttpHead, Method, ResponseDecoder};
use shiguredo_s3::{Client, Config, Credentials, S3Request, S3Response};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// `SystemTime::now()` をテスト本体から呼び出すためのヘルパ
///
/// shiguredo_s3 は Sans I/O のため `build_request` / `presigned` に
/// 現在時刻を引数で渡す。テスト側で副作用を 1 箇所に閉じ込める目的。
pub fn now() -> std::time::SystemTime {
    std::time::SystemTime::now()
}

/// テスト用の Client を構築する
///
/// - region: us-east-1 (CreateBucket で LocationConstraint を省略できる)
/// - endpoint: 127.0.0.1 のホストポートに接続する HTTP エンドポイント
/// - force_path_style: true (S3 互換サーバーはパススタイルが必要)
pub fn build_client(port: u16, access_key: &str, secret_key: &str) -> Client {
    let config = Config::builder()
        .region("us-east-1")
        .credentials_provider(Credentials::new(access_key, secret_key, None, None, "test"))
        .endpoint(format!("http://127.0.0.1:{port}"))
        // 仮想ホストスタイルに対応していない S3 互換サーバーが多いためパススタイルを使う
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
pub async fn execute(s3_request: S3Request) -> S3Response {
    let addr = format!("{}:{}", s3_request.host, s3_request.port);
    let encoded = encode_request(&s3_request);
    send_encoded(&s3_request.method, &addr, &encoded).await
}

/// build_request + execute + parse_response をまとめた便利関数
///
/// レスポンスのパースに失敗した場合はパニックする。
/// エラーレスポンスを直接検査したい場合は execute() を直接使うこと。
pub async fn send<T>(
    request: S3Request,
    parse: impl FnOnce(&S3Response) -> Result<T, shiguredo_s3::Error>,
) -> T {
    let response = execute(request).await;
    parse(&response).expect("failed to parse response")
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

/// エンコード済み HTTP リクエストを TCP で送信してレスポンスを 1 つ読み切る
///
/// 接続失敗・プロトコルエラーは expect でパニックする (テストヘルパーであるため)。
async fn send_encoded(method: &str, addr: &str, encoded: &[u8]) -> S3Response {
    let tcp = tokio::net::TcpStream::connect(addr)
        .await
        .expect("failed to connect");

    let (mut reader, mut writer) = tokio::io::split(tcp);
    writer
        .write_all(encoded)
        .await
        .expect("failed to write request");

    let mut decoder = ResponseDecoder::new();
    decoder.set_request_method(method);

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
