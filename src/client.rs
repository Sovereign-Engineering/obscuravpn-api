use crate::cmd::{parse_response, ApiError, ApiErrorKind, Cmd, ETagCmd, ProtocolError};
use crate::response::Response;
use crate::token::AcquireToken;
use crate::types::{AccountId, AuthToken};
use anyhow::{anyhow, Context};
use http::HeaderValue;
use reqwest::ClientBuilder;
use rustls::client::WebPkiServerVerifier;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Debug)]
pub struct Client {
    account_id: AccountId,
    base_url: String,
    alternative_hosts: Vec<String>,
    http: reqwest::Client,
    http_no_sni: reqwest::Client,
    cached_auth_token: Arc<Mutex<Option<AuthToken>>>,
    acquiring_auth_token: tokio::sync::Mutex<()>,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("API Error: {0}")]
    ApiError(#[from] ApiError),
    #[error("invalid header value")]
    InvalidHeaderValue,
    #[error("request processing error: {0}")]
    Other(#[from] anyhow::Error),
    /// We got a response but it wasn't the expected format.
    ///
    /// Most likely a response from a proxy or similar.
    #[error("Protocol Error: {0}")]
    ProtocolError(#[from] ProtocolError),
    #[error("error executing request: {0}")]
    RequestExecError(#[from] reqwest::Error),
}

impl Client {
    pub fn new(base_url: impl ToString, alternative_hosts: Vec<String>, account_id: AccountId, user_agent: &str) -> anyhow::Result<Self> {
        let mut base_url = base_url.to_string();
        if !base_url.ends_with('/') {
            base_url += "/"
        }
        let server_name_for_host_verification = Url::parse(&base_url)
            .context("can't parse base url")?
            .host_str()
            .context("base url does not contain host")?
            .to_string();
        dbg!(&server_name_for_host_verification);
        let mut rustls_config = Self::rustls_config(server_name_for_host_verification)?;
        let http = Self::http_client_builder(user_agent, rustls_config.clone())?;
        rustls_config.enable_sni = false;
        let http_no_sni = Self::http_client_builder(user_agent, rustls_config)?;

        Ok(Self {
            account_id,
            base_url,
            cached_auth_token: Arc::new(Mutex::new(None)),
            http,
            http_no_sni,
            acquiring_auth_token: tokio::sync::Mutex::new(()),
            alternative_hosts,
        })
    }

    fn http_client_builder(user_agent: &str, rustls_config: rustls::ClientConfig) -> anyhow::Result<reqwest::Client> {
        ClientBuilder::new()
            .timeout(Duration::from_secs(60))
            .read_timeout(Duration::from_secs(10))
            .user_agent(user_agent)
            .use_preconfigured_tls(rustls_config)
            .build()
            .context("failed to initialize HTTP client")
    }

    fn clear_auth_token(&self, token: AuthToken) {
        let mut guard = self.cached_auth_token.lock().unwrap();
        if guard.as_ref() == Some(&token) {
            guard.take();
        }
    }

    pub async fn acquire_auth_token(&self) -> Result<AuthToken, ClientError> {
        if let Some(auth_token) = self.get_auth_token() {
            return Ok(auth_token);
        }

        let acquiring_auth_token = self.acquiring_auth_token.lock().await;

        if let Some(auth_token) = self.get_auth_token() {
            return Ok(auth_token);
        }
        let account_id = self.account_id.clone();
        let request = AcquireToken { account_id }.to_request(&self.base_url)?;
        let res = self.send_http(request).await?;
        let res = parse_response::<String>(res).await?;
        let auth_token: AuthToken = res.into_body().context("No auth token in response")?.into();
        self.set_auth_token(Some(auth_token.clone()));

        drop(acquiring_auth_token);
        Ok(auth_token)
    }

    pub fn get_auth_token(&self) -> Option<AuthToken> {
        self.cached_auth_token.lock().unwrap().clone()
    }

    pub fn set_auth_token(&self, token: Option<AuthToken>) {
        *self.cached_auth_token.lock().unwrap() = token
    }

    async fn send_http(&self, request: http::Request<String>) -> Result<reqwest::Response, ClientError> {
        let reqwest_request: reqwest::Request = request.clone().try_into().context("could not construct reqwest::Request")?;
        let first_error = match self.http.execute(reqwest_request).await {
            Ok(resp) => return Ok(resp),
            Err(error) => {
                tracing::error!(
                    message_id = "XfTLkg6w",
                    ?error,
                    "error executing request, maybe blocked, trying again with alternative hosts",
                );
                error
            }
        };

        for host in &self.alternative_hosts {
            let mut reqwest_request: reqwest::Request = request.clone().try_into().context("could not construct reqwest::Request")?;
            if let Err(error) = reqwest_request.url_mut().set_host(Some(host.as_str())) {
                tracing::error!(message_id = "6mXOeRSL", host, ?error, "failed to set alternative host on request");
                continue;
            };
            match self.http.execute(reqwest_request).await {
                Ok(resp) => return Ok(resp),
                Err(error) => tracing::error!(
                    message_id = "m6JLZaYN",
                    host,
                    ?error,
                    "error executing request with alternative host, maybe blocked"
                ),
            }
        }

        tracing::error!("all attempts to evade blocks using alternative hosts failed, trying again without SNI");
        let reqwest_request: reqwest::Request = request.clone().try_into().context("could not construct reqwest::Request")?;
        match self.http_no_sni.execute(reqwest_request).await {
            Ok(resp) => return Ok(resp),
            Err(error) => tracing::error!(
                message_id = "CttGsdTj",
                ?error,
                "error executing request without SNI, maybe blocked, trying again with alternative hosts"
            ),
        }

        tracing::error!(message_id = "QmAdyhxm", "all attempts to evade blocks failed, returning original error");
        Err(first_error.into())
    }

    pub async fn run<C: Cmd>(&self, cmd: C) -> Result<C::Output, ClientError> {
        self.run_impl(cmd, None)
            .await?
            .into_body()
            .ok_or(anyhow::Error::msg("Non-conditional request has no body.").into())
    }

    pub async fn run_with_etag<C: ETagCmd>(&self, cmd: C, etag: Option<&[u8]>) -> Result<Response<C::Output>, ClientError> {
        self.run_impl(cmd, etag).await
    }

    async fn run_impl<C: Cmd>(&self, cmd: C, etag: Option<&[u8]>) -> Result<Response<C::Output>, ClientError> {
        let etag = etag
            .map(HeaderValue::from_bytes)
            .transpose()
            .map_err(|_| ClientError::InvalidHeaderValue)?;
        for _ in 0..3 {
            let auth_token = self.acquire_auth_token().await?;
            if let Some(output) = self.run_once::<C>(&cmd, &auth_token, etag.clone()).await? {
                return Ok(output);
            }
            self.clear_auth_token(auth_token);
        }
        Err(anyhow!("repeatedly acquired invalid auth token").into())
    }

    // Sends the http request and maps expected error codes to client errors.
    // Returns `Ok(None)` if the auth token is invalid, because this error shouldn't bubble up.
    async fn run_once<C: Cmd>(
        &self,
        body: &C,
        auth_token: &AuthToken,
        etag: Option<HeaderValue>,
    ) -> Result<Option<Response<C::Output>>, ClientError> {
        let request = body.to_request(&self.base_url, auth_token, etag)?;
        let res = self.send_http(request).await?;
        match parse_response(res).await {
            Ok(output) => Ok(Some(output)),
            Err(ClientError::ApiError(error)) => match error.body.error {
                ApiErrorKind::MissingOrInvalidAuthToken {} => Ok(None),
                _ => Err(ClientError::ApiError(error)),
            },
            Err(err) => Err(err),
        }
    }

    fn rustls_config(server_name_for_cert_verification: String) -> anyhow::Result<rustls::ClientConfig> {
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(VerifyApiServerCert::new(server_name_for_cert_verification)?)
            .with_no_client_auth();
        Ok(crypto)
    }
}

#[derive(Debug)]
struct VerifyApiServerCert {
    server_name: rustls::pki_types::ServerName<'static>,
    web_pki_server_verifier: Arc<dyn rustls::client::danger::ServerCertVerifier>,
}

impl VerifyApiServerCert {
    fn new(server_name: String) -> anyhow::Result<Arc<Self>> {
        let roots = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        let web_pki_server_verifier = WebPkiServerVerifier::builder(roots.into()).build()?;
        let server_name = rustls::pki_types::ServerName::try_from(server_name)?;
        Ok(Arc::new(Self {
            server_name,
            web_pki_server_verifier,
        }))
    }
}

impl rustls::client::danger::ServerCertVerifier for VerifyApiServerCert {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::pki_types::CertificateDer<'_>,
        intermediates: &[rustls::pki_types::CertificateDer<'_>],
        server_name: &rustls::pki_types::ServerName<'_>,
        ocsp_response: &[u8],
        now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        if server_name.to_str() != self.server_name.to_str() {
            tracing::info!(
                message_id = "jx7gpGq8",
                verify_server_name = %self.server_name.to_str(),
                request_server_name = %server_name.to_str(),
                "verifying server certificate with different server name",
            );
        }
        self.web_pki_server_verifier
            .verify_server_cert(end_entity, intermediates, &self.server_name, ocsp_response, now)
    }
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.web_pki_server_verifier.verify_tls12_signature(message, cert, dss)
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.web_pki_server_verifier.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.web_pki_server_verifier.supported_verify_schemes()
    }
}
