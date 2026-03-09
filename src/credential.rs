/// AWS クレデンシャル
///
/// `Debug` 実装ではシークレットアクセスキーとセッショントークンをマスクする。
#[derive(Clone)]
pub struct Credential {
    /// アクセスキー ID
    pub(crate) access_key_id: String,
    /// シークレットアクセスキー
    pub(crate) secret_access_key: String,
    /// セッショントークン (一時クレデンシャル用)
    pub(crate) session_token: Option<String>,
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credential")
            .field("access_key_id", &self.access_key_id)
            .field("secret_access_key", &"**REDACTED**")
            .field(
                "session_token",
                &self.session_token.as_ref().map(|_| "**REDACTED**"),
            )
            .finish()
    }
}

impl Credential {
    /// 長期クレデンシャルを作成する
    pub fn new(access_key_id: impl Into<String>, secret_access_key: impl Into<String>) -> Self {
        Self {
            access_key_id: access_key_id.into(),
            secret_access_key: secret_access_key.into(),
            session_token: None,
        }
    }

    /// 一時クレデンシャルを作成する (STS / IAM ロール用)
    pub fn with_session_token(
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        session_token: impl Into<String>,
    ) -> Self {
        Self {
            access_key_id: access_key_id.into(),
            secret_access_key: secret_access_key.into(),
            session_token: Some(session_token.into()),
        }
    }

    /// アクセスキー ID を返す
    pub fn access_key_id(&self) -> &str {
        &self.access_key_id
    }

    /// シークレットアクセスキーを返す
    pub fn secret_access_key(&self) -> &str {
        &self.secret_access_key
    }

    /// セッショントークンを返す
    pub fn session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }
}
