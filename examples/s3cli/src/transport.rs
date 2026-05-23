// -------------------------------------------------------
// I/O 層
// -------------------------------------------------------

use std::sync::{Arc, Mutex};

use rustls::pki_types::ServerName;
use shiguredo_http11::{HttpHead, ResponseDecoder};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsConnector;

use shiguredo_s3::{S3Request, S3Response};

/// rustls の ClientConfig を構築する
pub(crate) fn build_tls_config(ignore_cert_check: bool) -> rustls::ClientConfig {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    if ignore_cert_check {
        eprintln!("WARNING: TLS certificate verification is disabled (S3CLI_IGNORE_CERT_CHECK=1)");
        rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .expect("failed to set TLS protocol versions")
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth()
    } else {
        let verifier = rustls_platform_verifier::Verifier::new(Arc::clone(&provider))
            .expect("failed to create platform verifier");
        rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .expect("failed to set TLS protocol versions")
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(verifier))
            .with_no_client_auth()
    }
}

/// 証明書検証を無視する検証器 (テスト環境向け)
#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::aws_lc_rs::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

/// S3Request を HTTP/1.1 リクエストにエンコードする
pub(crate) fn encode_request(
    s3_request: &S3Request,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut request = shiguredo_http11::Request::new(&s3_request.method, &s3_request.uri)?;
    for (name, value) in &s3_request.headers {
        request.add_header(name, value)?;
    }
    if !s3_request.body.is_empty() {
        request.set_body(s3_request.body.clone());
    }
    Ok(request.encode()?)
}

/// S3Request を送信して S3Response を返す
pub(crate) async fn execute(
    tls_config: &Arc<rustls::ClientConfig>,
    s3_request: S3Request,
) -> Result<S3Response, Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("{}:{}", s3_request.host, s3_request.port);
    let tcp = tokio::net::TcpStream::connect(&addr).await?;
    tcp.set_nodelay(true)?;

    let encoded = encode_request(&s3_request)?;

    let mut decoder = ResponseDecoder::new();
    decoder.set_request_method(&s3_request.method);

    if s3_request.https {
        let server_name: ServerName<'_> = ServerName::try_from(s3_request.host.clone())
            .map_err(|_| format!("invalid DNS name: {}", s3_request.host))?;
        let connector = TlsConnector::from(Arc::clone(tls_config));
        let mut tls = connector.connect(server_name, tcp).await?;

        tls.write_all(&encoded).await?;
        read_response(&mut tls, &mut decoder).await
    } else {
        let (mut reader, mut writer) = tokio::io::split(tcp);
        writer.write_all(&encoded).await?;
        read_response(&mut reader, &mut decoder).await
    }
}

/// ストリームからレスポンスを読み取る
pub(crate) async fn read_response<R: AsyncReadExt + Unpin>(
    reader: &mut R,
    decoder: &mut ResponseDecoder,
) -> Result<S3Response, Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = [0u8; 65536];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            decoder.mark_eof();
            if let Some(response) = decoder.decode()? {
                return Ok(into_s3_response(response));
            }
            return Err("unexpected EOF".into());
        }
        decoder.feed(&buf[..n])?;
        if let Some(response) = decoder.decode()? {
            return Ok(into_s3_response(response));
        }
    }
}

/// shiguredo_http11::Response を S3Response に変換する
fn into_s3_response(response: shiguredo_http11::Response) -> S3Response {
    S3Response {
        status_code: response.status_code(),
        headers: response.headers().to_vec(),
        body: response.body_bytes().unwrap_or_default().to_vec(),
    }
}

/// build_request + execute + parse_response をまとめた便利関数
pub(crate) async fn send<T>(
    tls_config: &Arc<rustls::ClientConfig>,
    request: S3Request,
    parse: impl FnOnce(&S3Response) -> Result<T, shiguredo_s3::Error>,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
    let response = execute(tls_config, request).await?;
    Ok(parse(&response)?)
}

// -------------------------------------------------------
// 接続プール
// -------------------------------------------------------

/// プールされた接続
pub(crate) enum PooledStream {
    Tls(Box<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>),
    Plain(tokio::net::TcpStream),
}

/// S3 エンドポイントへの接続プール
///
/// マルチパートアップロード時に TLS ハンドシェイクのコストを削減する。
pub(crate) struct ConnectionPool {
    connections: Mutex<Vec<PooledStream>>,
    tls_config: Arc<rustls::ClientConfig>,
    host: String,
    port: u16,
    https: bool,
}

impl ConnectionPool {
    pub(crate) fn new(
        tls_config: &Arc<rustls::ClientConfig>,
        host: &str,
        port: u16,
        https: bool,
    ) -> Self {
        Self {
            connections: Mutex::new(Vec::new()),
            tls_config: Arc::clone(tls_config),
            host: host.to_string(),
            port,
            https,
        }
    }

    /// プールから接続を取得する (空なら新規作成する)
    pub(crate) async fn acquire(
        &self,
    ) -> Result<PooledStream, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(conn) = self.connections.lock().unwrap().pop() {
            return Ok(conn);
        }
        self.create_new().await
    }

    /// 新規接続を作成する
    async fn create_new(&self) -> Result<PooledStream, Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("{}:{}", self.host, self.port);
        let tcp = tokio::net::TcpStream::connect(&addr).await?;
        tcp.set_nodelay(true)?;
        if self.https {
            let server_name = ServerName::try_from(self.host.clone())
                .map_err(|_| format!("invalid DNS name: {}", self.host))?;
            let connector = TlsConnector::from(Arc::clone(&self.tls_config));
            let tls = connector.connect(server_name, tcp).await?;
            Ok(PooledStream::Tls(Box::new(tls)))
        } else {
            Ok(PooledStream::Plain(tcp))
        }
    }

    /// 接続をプールに返却する
    pub(crate) fn release(&self, conn: PooledStream) {
        self.connections.lock().unwrap().push(conn);
    }
}

/// PooledStream 上でリクエストを実行する
pub(crate) async fn execute_on_stream(
    stream: &mut PooledStream,
    encoded: &[u8],
    request_method: &str,
) -> Result<S3Response, Box<dyn std::error::Error + Send + Sync>> {
    let mut decoder = ResponseDecoder::new();
    decoder.set_request_method(request_method);
    match stream {
        PooledStream::Tls(tls) => {
            tls.write_all(encoded).await?;
            read_response(tls, &mut decoder).await
        }
        PooledStream::Plain(tcp) => {
            tcp.write_all(encoded).await?;
            read_response(tcp, &mut decoder).await
        }
    }
}

/// 接続プールを使用してリクエストを実行する
///
/// エラー時は接続を破棄する (サーバー側で切断された可能性があるため)。
pub(crate) async fn execute_pooled(
    pool: &ConnectionPool,
    s3_request: S3Request,
) -> Result<S3Response, Box<dyn std::error::Error + Send + Sync>> {
    let encoded = encode_request(&s3_request)?;
    let mut stream = pool.acquire().await?;
    let result = execute_on_stream(&mut stream, &encoded, &s3_request.method).await;
    if result.is_ok() {
        pool.release(stream);
    }
    result
}

/// 接続プールを使用して send する便利関数
pub(crate) async fn send_pooled<T>(
    pool: &ConnectionPool,
    request: S3Request,
    parse: impl FnOnce(&S3Response) -> Result<T, shiguredo_s3::Error>,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
    let response = execute_pooled(pool, request).await?;
    Ok(parse(&response)?)
}
