use crate::credential::Credentials;
use crate::signing::uri_encode_path;

// -------------------------------------------------------
// 設定参照 (ライフタイム付き、Sans I/O で使用)
// -------------------------------------------------------

/// `Client` の設定への参照
pub(crate) struct ClientConfig<'a> {
    pub(crate) region: &'a str,
    pub(crate) credentials: &'a Credentials,
    /// スキームを除去したホスト名 (ポート含む場合あり)
    pub(crate) endpoint: Option<&'a str>,
    pub(crate) force_path_style: bool,
    /// HTTPS を使用するかどうか (endpoint のスキームから判定)
    pub(crate) https: bool,
    /// TLS 証明書の検証を無視する
    pub(crate) ignore_cert_check: bool,
}

// -------------------------------------------------------
// ホスト・パス計算
// -------------------------------------------------------

/// endpoint 文字列からスキームを解析する
///
/// - `"http://localhost:9000"` → `(false, "localhost:9000")`
/// - `"https://minio.example.com"` → `(true, "minio.example.com")`
/// - `"minio.example.com"` → `(true, "minio.example.com")` (スキーム省略時は HTTPS)
pub(super) fn parse_endpoint_scheme(endpoint: &str) -> (bool, &str) {
    if let Some(rest) = endpoint.strip_prefix("http://") {
        (false, rest)
    } else if let Some(rest) = endpoint.strip_prefix("https://") {
        (true, rest)
    } else {
        (true, endpoint)
    }
}

pub(super) fn service_host(config: &ClientConfig<'_>) -> String {
    config
        .endpoint
        .map(String::from)
        .unwrap_or_else(|| format!("s3.{}.amazonaws.com", config.region))
}

/// HTTPS かつバケット名にドットを含む場合は path-style にフォールバックが必要
fn use_path_style_for_bucket(config: &ClientConfig<'_>, bucket: &str) -> bool {
    config.force_path_style || (config.https && bucket.contains('.'))
}

pub(super) fn host_for_bucket(config: &ClientConfig<'_>, bucket: &str) -> String {
    let base = service_host(config);
    if use_path_style_for_bucket(config, bucket) {
        base
    } else {
        format!("{bucket}.{base}")
    }
}

/// ホスト文字列からポート部分を除去して接続先ホスト名を返す
///
/// IPv6 アドレス (`[::1]:9000` や `[::1]`) を正しく処理する
pub(super) fn extract_connect_host(host: &str) -> String {
    if let Some(end) = host.find(']') {
        // IPv6: `[::1]:9000` → `[::1]`, `[::1]` → `[::1]`
        host[..=end].to_string()
    } else if let Some(pos) = host.rfind(':') {
        // IPv4 / ホスト名でポート付き: `localhost:9000` → `localhost`
        // ただしコロンが含まれていてもポート部分が数値でなければホスト名全体を返す
        if host[pos + 1..].parse::<u16>().is_ok() {
            host[..pos].to_string()
        } else {
            host.to_string()
        }
    } else {
        host.to_string()
    }
}

/// endpoint からポートを抽出する (明示的なポートがない場合は HTTPS なら 443、HTTP なら 80)
///
/// IPv6 アドレス (`[::1]:9000`) を正しく処理する
pub(super) fn extract_port(config: &ClientConfig<'_>) -> u16 {
    if let Some(endpoint) = config.endpoint
        && let Some(port) = parse_port_from_authority(endpoint)
    {
        return port;
    }
    if config.https { 443 } else { 80 }
}

/// authority 文字列からポートを解析する
///
/// `[::1]:9000` → `Some(9000)`, `localhost:9000` → `Some(9000)`, `[::1]` → `None`
fn parse_port_from_authority(authority: &str) -> Option<u16> {
    if authority.starts_with('[') {
        // IPv6: `]` の後に `:port` があればポート
        let after_bracket = authority.find(']')?;
        let rest = &authority[after_bracket + 1..];
        let port_str = rest.strip_prefix(':')?;
        port_str.parse().ok()
    } else {
        // IPv4 / ホスト名: 最後の `:` 以降がポート
        let port_str = authority.rsplit(':').next()?;
        port_str.parse().ok()
    }
}

pub(super) fn path_for_key(config: &ClientConfig<'_>, bucket: &str, key: &str) -> String {
    let encoded_key = uri_encode_path(key);
    if use_path_style_for_bucket(config, bucket) {
        if encoded_key.is_empty() {
            format!("/{bucket}")
        } else {
            format!("/{bucket}/{encoded_key}")
        }
    } else if encoded_key.is_empty() {
        "/".to_string()
    } else {
        format!("/{encoded_key}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential::Credentials;

    /// テスト用のクレデンシャルを生成する
    fn test_credentials() -> Credentials {
        Credentials::new(
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            None,
            None,
            "test",
        )
    }

    /// テスト用の設定参照を生成する
    fn test_config<'a>(
        region: &'a str,
        credentials: &'a Credentials,
        endpoint: Option<&'a str>,
        force_path_style: bool,
        https: bool,
    ) -> ClientConfig<'a> {
        ClientConfig {
            region,
            credentials,
            endpoint,
            force_path_style,
            https,
            ignore_cert_check: false,
        }
    }

    /// endpoint のスキーム解析 (http / https / スキーム省略)
    #[test]
    fn test_parse_endpoint_scheme() {
        assert_eq!(
            parse_endpoint_scheme("http://localhost:9000"),
            (false, "localhost:9000")
        );
        assert_eq!(
            parse_endpoint_scheme("https://minio.example.com"),
            (true, "minio.example.com")
        );
        assert_eq!(
            parse_endpoint_scheme("minio.example.com"),
            (true, "minio.example.com")
        );
    }

    /// HTTPS かつバケット名にドットを含む場合は path-style にフォールバックする
    #[test]
    fn test_use_path_style_for_bucket() {
        let credentials = test_credentials();
        let config = test_config("us-east-1", &credentials, None, false, true);
        assert!(use_path_style_for_bucket(&config, "example.bucket"));
        assert!(!use_path_style_for_bucket(&config, "examplebucket"));

        let force_path_style = test_config("us-east-1", &credentials, None, true, true);
        assert!(use_path_style_for_bucket(
            &force_path_style,
            "examplebucket"
        ));

        let http = test_config("us-east-1", &credentials, None, false, false);
        assert!(!use_path_style_for_bucket(&http, "example.bucket"));
    }
    /// バケットホストの構築 (仮想ホスト形式と path-style)
    #[test]
    fn test_host_for_bucket() {
        let credentials = test_credentials();

        let virtual_host = test_config("us-east-1", &credentials, None, false, true);
        assert_eq!(
            host_for_bucket(&virtual_host, "examplebucket"),
            "examplebucket.s3.us-east-1.amazonaws.com"
        );

        let path_style = test_config("us-east-1", &credentials, None, true, true);
        assert_eq!(
            host_for_bucket(&path_style, "examplebucket"),
            "s3.us-east-1.amazonaws.com"
        );
    }

    /// カスタム endpoint 指定時はバケットホストが endpoint を使う
    #[test]
    fn test_host_for_bucket_custom_endpoint() {
        let credentials = test_credentials();

        let custom = test_config(
            "us-east-1",
            &credentials,
            Some("minio.example.com"),
            false,
            true,
        );
        assert_eq!(
            host_for_bucket(&custom, "examplebucket"),
            "examplebucket.minio.example.com"
        );

        let path_style = test_config(
            "us-east-1",
            &credentials,
            Some("minio.example.com"),
            true,
            true,
        );
        assert_eq!(
            host_for_bucket(&path_style, "examplebucket"),
            "minio.example.com"
        );
    }

    /// ホスト文字列からポート部分を除去する (IPv6 と IPv4 / ホスト名)
    #[test]
    fn test_extract_connect_host() {
        assert_eq!(extract_connect_host("[::1]:9000"), "[::1]");
        assert_eq!(extract_connect_host("[::1]"), "[::1]");
        assert_eq!(extract_connect_host("localhost:9000"), "localhost");
        assert_eq!(extract_connect_host("example.com"), "example.com");
        // ポート部分が数値でなければホスト名全体を返す
        assert_eq!(extract_connect_host("localhost:abc"), "localhost:abc");
    }

    /// endpoint のポート抽出 (明示ポート / 既定値 443 / 80)
    #[test]
    fn test_extract_port() {
        let credentials = test_credentials();
        let explicit = test_config(
            "us-east-1",
            &credentials,
            Some("localhost:9000"),
            false,
            true,
        );
        assert_eq!(extract_port(&explicit), 9000);

        let https = test_config("us-east-1", &credentials, None, false, true);
        assert_eq!(extract_port(&https), 443);

        let http = test_config("us-east-1", &credentials, None, false, false);
        assert_eq!(extract_port(&http), 80);
    }

    /// endpoint にポートが含まれない場合は既定値 (HTTPS 443 / HTTP 80) にフォールバックする
    #[test]
    fn test_extract_port_no_port_fallback() {
        let credentials = test_credentials();

        let https = test_config(
            "us-east-1",
            &credentials,
            Some("s3.example.com"),
            false,
            true,
        );
        assert_eq!(extract_port(&https), 443);

        let http = test_config(
            "us-east-1",
            &credentials,
            Some("s3.example.com"),
            false,
            false,
        );
        assert_eq!(extract_port(&http), 80);
    }

    /// authority 文字列からのポート解析 (IPv6 と IPv4 / ホスト名)
    #[test]
    fn test_parse_port_from_authority() {
        assert_eq!(parse_port_from_authority("[::1]:9000"), Some(9000));
        assert_eq!(parse_port_from_authority("[::1]"), None);
        assert_eq!(parse_port_from_authority("localhost:9000"), Some(9000));
        assert_eq!(parse_port_from_authority("localhost"), None);
    }

    /// キーのパス構築 (path-style と仮想ホスト形式)
    #[test]
    fn test_path_for_key() {
        let credentials = test_credentials();
        let path_style = test_config("us-east-1", &credentials, None, true, true);
        assert_eq!(
            path_for_key(&path_style, "examplebucket", "test.txt"),
            "/examplebucket/test.txt"
        );
        assert_eq!(
            path_for_key(&path_style, "examplebucket", ""),
            "/examplebucket"
        );

        let virtual_host = test_config("us-east-1", &credentials, None, false, true);
        assert_eq!(
            path_for_key(&virtual_host, "examplebucket", "test.txt"),
            "/test.txt"
        );
        assert_eq!(path_for_key(&virtual_host, "examplebucket", ""), "/");
    }
}
