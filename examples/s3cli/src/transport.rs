// -------------------------------------------------------
// I/O 層
// -------------------------------------------------------

use std::sync::{Arc, Mutex};

use rustls::pki_types::ServerName;
use shiguredo_http11::ResponseDecoder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsConnector;

use shiguredo_s3::{S3Request, S3Response};

/// rustls の ClientConfig を構築する
pub(crate) fn build_tls_config(ignore_cert_check: bool) -> rustls::ClientConfig {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let config = if ignore_cert_check {
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
    };
    // feature h2 有効時のみ ALPN に h2 と http/1.1 を載せる
    // サーバーが HTTP/2 に対応していれば h2 が合意され、そうでなければ http/1.1 にフォールバックする
    #[cfg(feature = "h2")]
    let config = {
        let mut config = config;
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        config
    };
    config
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
    let mut request = shiguredo_http11::Request::new(&s3_request.method, &s3_request.uri);
    for (name, value) in &s3_request.headers {
        request.add_header(name, value);
    }
    if !s3_request.body.is_empty() {
        request.body = s3_request.body.clone();
    }
    Ok(request.try_encode()?)
}

/// S3Request を送信して S3Response を返す
pub(crate) async fn execute(
    tls_config: &Arc<rustls::ClientConfig>,
    s3_request: S3Request,
) -> Result<S3Response, Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("{}:{}", s3_request.host, s3_request.port);
    let tcp = tokio::net::TcpStream::connect(&addr).await?;
    tcp.set_nodelay(true)?;

    if s3_request.https {
        let server_name: ServerName<'_> = ServerName::try_from(s3_request.host.clone())
            .map_err(|_| format!("invalid DNS name: {}", s3_request.host))?;
        let connector = TlsConnector::from(Arc::clone(tls_config));
        let mut tls = connector.connect(server_name, tcp).await?;

        // feature h2 有効時は ALPN の合意結果が h2 なら HTTP/2 で送信する
        #[cfg(feature = "h2")]
        if tls.get_ref().1.alpn_protocol() == Some(b"h2") {
            return h2::execute_h2(tls, s3_request).await;
        }

        let encoded = encode_request(&s3_request)?;
        let mut decoder = ResponseDecoder::new();
        if s3_request.expect_no_body {
            decoder.set_expect_no_body(true);
        }
        tls.write_all(&encoded).await?;
        read_response(&mut tls, &mut decoder).await
    } else {
        let encoded = encode_request(&s3_request)?;
        let mut decoder = ResponseDecoder::new();
        if s3_request.expect_no_body {
            decoder.set_expect_no_body(true);
        }
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
        status_code: response.status_code,
        headers: response.headers,
        body: response.body,
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
///
/// feature h2 有効時は接続プールを迂回するためバリアントは使用されない
#[cfg_attr(feature = "h2", allow(dead_code))]
pub(crate) enum PooledStream {
    Tls(Box<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>),
    Plain(tokio::net::TcpStream),
}

/// S3 エンドポイントへの接続プール
///
/// マルチパートアップロード時に TLS ハンドシェイクのコストを削減する。
/// feature h2 有効時は tls_config のみ使用し、他のフィールドは使用されない
pub(crate) struct ConnectionPool {
    #[cfg_attr(feature = "h2", allow(dead_code))]
    connections: Mutex<Vec<PooledStream>>,
    tls_config: Arc<rustls::ClientConfig>,
    #[cfg_attr(feature = "h2", allow(dead_code))]
    host: String,
    #[cfg_attr(feature = "h2", allow(dead_code))]
    port: u16,
    #[cfg_attr(feature = "h2", allow(dead_code))]
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
    #[cfg_attr(feature = "h2", allow(dead_code))]
    pub(crate) async fn acquire(
        &self,
    ) -> Result<PooledStream, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(conn) = self.connections.lock().unwrap().pop() {
            return Ok(conn);
        }
        self.create_new().await
    }

    /// 新規接続を作成する
    #[cfg_attr(feature = "h2", allow(dead_code))]
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
    #[cfg_attr(feature = "h2", allow(dead_code))]
    pub(crate) fn release(&self, conn: PooledStream) {
        self.connections.lock().unwrap().push(conn);
    }
}

/// PooledStream 上でリクエストを実行する
#[cfg_attr(feature = "h2", allow(dead_code))]
pub(crate) async fn execute_on_stream(
    stream: &mut PooledStream,
    encoded: &[u8],
    expect_no_body: bool,
) -> Result<S3Response, Box<dyn std::error::Error + Send + Sync>> {
    let mut decoder = ResponseDecoder::new();
    if expect_no_body {
        decoder.set_expect_no_body(true);
    }
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
///
/// feature h2 有効時は接続プールを使用せず `execute` に委譲する。
/// ALPN の合意結果 (h2 または http/1.1) は `execute` 内で判定する。
/// マルチパートの接続再利用は現状諦めており、HTTP/2 専用プールは今後の課題。
pub(crate) async fn execute_pooled(
    pool: &ConnectionPool,
    s3_request: S3Request,
) -> Result<S3Response, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(feature = "h2")]
    {
        return execute(&pool.tls_config, s3_request).await;
    }
    #[cfg(not(feature = "h2"))]
    {
        let encoded = encode_request(&s3_request)?;
        let mut stream = pool.acquire().await?;
        let result = execute_on_stream(&mut stream, &encoded, s3_request.expect_no_body).await;
        if result.is_ok() {
            pool.release(stream);
        }
        result
    }
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

// -------------------------------------------------------
// HTTP/2 送信パス (feature h2 有効時のみ)
// -------------------------------------------------------

/// HTTP/2 送信パス
///
/// shiguredo_http2 (Sans I/O) を tokio の TlsStream 上で駆動する。
/// MVP として 1 コネクションにつき 1 ストリームで使い捨てる。
#[cfg(feature = "h2")]
mod h2 {
    use shiguredo_http2::{Connection, Event, HeaderField, Limits};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio_rustls::client::TlsStream;

    use shiguredo_s3::{S3Request, S3Response};

    /// HTTPS TLS ストリーム上で HTTP/2 リクエストを 1 往復する
    pub(super) async fn execute_h2(
        mut tls: TlsStream<TcpStream>,
        s3_request: S3Request,
    ) -> Result<S3Response, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = Connection::client(Limits::default());

        // コネクションプリフェイスを直接送信する (Sans I/O のため I/O は呼び出し側で行う)
        tls.write_all(shiguredo_http2::CONNECTION_PREFACE).await?;
        tls.flush().await?;
        conn.mark_preface_sent();

        // 初期 SETTINGS を送信する
        conn.send_settings()?;
        flush(&mut tls, &mut conn).await?;

        // 擬似ヘッダとリクエストヘッダを組み立てる
        let headers = build_request_headers(&s3_request);

        // リクエストを送信する
        let stream_id = if s3_request.body.is_empty() {
            conn.start_stream(headers, true)?
        } else {
            let sid = conn.start_stream(headers, false)?;
            conn.send_data(sid, s3_request.body.clone(), true)?;
            sid
        };
        flush(&mut tls, &mut conn).await?;

        // レスポンスを受信する
        let mut status_code: Option<u16> = None;
        let mut response_headers: Vec<(String, String)> = Vec::new();
        let mut body: Vec<u8> = Vec::new();
        let mut buf = vec![0u8; 16384];

        loop {
            let event = next_event(&mut tls, &mut conn, &mut buf).await?;
            match event {
                Event::HeadersReceived {
                    stream_id: sid,
                    headers: hs,
                    end_stream,
                    ..
                } if sid == stream_id => {
                    for h in hs {
                        if h.name == b":status" {
                            let s = std::str::from_utf8(&h.value)
                                .map_err(|e| format!("invalid :status utf-8: {e}"))?;
                            status_code = Some(
                                s.parse()
                                    .map_err(|e| format!("invalid :status value: {e}"))?,
                            );
                        } else if !h.name.starts_with(b":") {
                            let name = String::from_utf8(h.name)
                                .map_err(|e| format!("invalid header name: {e}"))?;
                            let value = String::from_utf8(h.value)
                                .map_err(|e| format!("invalid header value: {e}"))?;
                            response_headers.push((name, value));
                        }
                    }
                    if end_stream {
                        break;
                    }
                }
                Event::DataReceived {
                    stream_id: sid,
                    data,
                    end_stream,
                } if sid == stream_id => {
                    body.extend_from_slice(&data);
                    if end_stream {
                        break;
                    }
                }
                Event::StreamClosed { stream_id: sid } if sid == stream_id => break,
                Event::GoawayReceived {
                    error_code,
                    debug_data,
                    ..
                } => {
                    return Err(format!(
                        "HTTP/2 GOAWAY received: {error_code} ({})",
                        String::from_utf8_lossy(&debug_data)
                    )
                    .into());
                }
                Event::ConnectionError { error_code, reason } => {
                    return Err(format!("HTTP/2 connection error: {error_code} ({reason})").into());
                }
                _ => {}
            }
        }

        let status_code = status_code.ok_or("HTTP/2 response missing :status pseudo header")?;
        Ok(S3Response {
            status_code,
            headers: response_headers,
            body,
        })
    }

    /// HTTP/2 形式のヘッダリストを組み立てる
    ///
    /// RFC 9113 Section 8.2.1: ヘッダ名は全て小文字で送信する。
    /// RFC 9113 Section 8.2.2: connection-specific ヘッダは送信してはならない。
    fn build_request_headers(req: &S3Request) -> Vec<HeaderField> {
        let scheme = if req.https { "https" } else { "http" };
        let authority = format!("{}:{}", req.host, req.port);

        let mut headers = Vec::with_capacity(req.headers.len() + 4);
        headers.push(HeaderField::from_str(":method", &req.method));
        headers.push(HeaderField::from_str(":path", &req.uri));
        headers.push(HeaderField::from_str(":scheme", scheme));
        headers.push(HeaderField::from_str(":authority", &authority));

        for (name, value) in &req.headers {
            let lower = name.to_ascii_lowercase();
            if is_connection_specific(&lower) {
                continue;
            }
            headers.push(HeaderField::new(
                lower.into_bytes(),
                value.as_bytes().to_vec(),
            ));
        }
        headers
    }

    /// RFC 9113 Section 8.2.2: HTTP/2 で送信してはならない connection-specific ヘッダかどうか
    fn is_connection_specific(name_lower: &str) -> bool {
        matches!(
            name_lower,
            "connection"
                | "keep-alive"
                | "proxy-connection"
                | "transfer-encoding"
                | "upgrade"
                | "host"
        )
    }

    /// 出力キューのバイト列を全て書き出してフラッシュする
    async fn flush(
        stream: &mut TlsStream<TcpStream>,
        conn: &mut Connection,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while let Some(out) = conn.poll_output() {
            stream.write_all(&out).await?;
        }
        stream.flush().await?;
        Ok(())
    }

    /// 次のイベントが取り出せるまで送受信をドライブする
    async fn next_event(
        stream: &mut TlsStream<TcpStream>,
        conn: &mut Connection,
        buf: &mut [u8],
    ) -> Result<Event, Box<dyn std::error::Error + Send + Sync>> {
        loop {
            flush(stream, conn).await?;
            if let Some(event) = conn.poll_event() {
                return Ok(event);
            }
            let n = stream.read(buf).await?;
            if n == 0 {
                return Err("unexpected EOF while reading HTTP/2 frames".into());
            }
            let _ = conn.feed(&buf[..n])?;
            conn.process()?;
        }
    }
}
