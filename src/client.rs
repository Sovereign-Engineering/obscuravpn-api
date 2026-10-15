use crate::cmd::{ApiError, ApiErrorBody, ApiErrorKind, AuthOptional, Cmd, CmdAuth, ETagCmd, ProtocolError, parse_response};
use crate::pow::PowOutput;
use crate::response::Response;
use crate::token::{AcquireToken, AcquireToken2Output};
use crate::types::{AccountId, AuthToken};
use anyhow::{Context, anyhow};
use http::HeaderValue;
use itertools::Itertools;
use reqwest::ClientBuilder;
use reqwest::dns::Resolve;
use rustls::client::WebPkiServerVerifier;
use std::iter::once;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use thiserror::Error;
use url::Url;

pub trait ClientAuth {
    fn as_authenticated(&self) -> Option<&Authenticated>;
}

pub trait CanRun<R: CmdAuth>: ClientAuth {}

#[derive(Debug)]
pub struct Unauthenticated;

impl ClientAuth for Unauthenticated {
    fn as_authenticated(&self) -> Option<&Authenticated> {
        None
    }
}

impl CanRun<AuthOptional> for Unauthenticated {}

#[derive(Debug)]
pub struct Authenticated {
    account_id: AccountId,
    cached_auth_token: Arc<Mutex<Option<AuthToken>>>,
    acquiring_auth_token: tokio::sync::Mutex<()>,
}

impl Authenticated {
    fn clear_auth_token(&self, token: AuthToken) {
        let mut guard = self.cached_auth_token.lock().unwrap();
        if guard.as_ref() == Some(&token) {
            guard.take();
        }
    }

    fn get_auth_token(&self) -> Option<AuthToken> {
        self.cached_auth_token.lock().unwrap().clone()
    }

    fn set_auth_token(&self, token: Option<AuthToken>) {
        *self.cached_auth_token.lock().unwrap() = token;
    }
}

impl ClientAuth for Authenticated {
    fn as_authenticated(&self) -> Option<&Authenticated> {
        Some(self)
    }
}

impl<R: CmdAuth> CanRun<R> for Authenticated {}

#[derive(Debug)]
pub struct Client<A = Authenticated> {
    auth: A,
    base_url: String,
    alternative_hosts: Vec<String>,
    http: reqwest::Client,
    http_no_sni: reqwest::Client,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("API Error: {0}")]
    ApiError(#[from] ApiError),
    #[error("invalid header value")]
    InvalidHeaderValue,
    #[error("response too large")]
    ResponseTooLarge,
    #[error("request processing error: {0}")]
    Other(#[from] anyhow::Error),
    /// We got a response, but it wasn't the expected format.
    ///
    /// Most likely a response from a proxy or similar.
    #[error("Protocol Error: {0}")]
    ProtocolError(#[from] ProtocolError),
    #[error("error executing request: {0}")]
    RequestExecError(#[from] reqwest::Error),
    #[error("proof of work timed out")]
    ProofOfWorkTimeout,
    #[error("proof of work failed: {0}")]
    ProofOfWork(argon2::Error),
}

const POW_SOLVE_TIMEOUT: Duration = Duration::from_secs(10);

impl Client<Unauthenticated> {
    pub fn new_unauthenticated(
        base_url: impl ToString,
        alternative_hosts: Vec<String>,
        user_agent: &str,
        #[cfg(not(any(target_os = "android", target_os = "windows")))] network_interface: Option<&str>,
        #[cfg(any(target_os = "android", target_os = "windows"))] network_interface: Option<std::net::IpAddr>,
        #[cfg(target_os = "linux")] so_mark: Option<u32>,
        resolver: Option<Arc<dyn Resolve>>,
    ) -> anyhow::Result<Self> {
        let mut base_url = base_url.to_string();
        if !base_url.ends_with('/') {
            base_url += "/"
        }
        let primary_host = Url::parse(&base_url)
            .context("can't parse base url")?
            .host_str()
            .context("base url does not contain host")?
            .to_string();
        let server_names = once(primary_host).chain(alternative_hosts.iter().cloned());

        let mut rustls_config = Self::rustls_config(server_names)?;
        let http = Self::http_client_builder(
            user_agent,
            rustls_config.clone(),
            network_interface,
            #[cfg(target_os = "linux")]
            so_mark,
            resolver.clone(),
        )?;
        rustls_config.enable_sni = false;
        let http_no_sni = Self::http_client_builder(
            user_agent,
            rustls_config,
            network_interface,
            #[cfg(target_os = "linux")]
            so_mark,
            resolver,
        )?;

        Ok(Self {
            auth: Unauthenticated,
            base_url,
            alternative_hosts,
            http,
            http_no_sni,
        })
    }

    pub fn into_authenticated(self, account_id: AccountId) -> Client<Authenticated> {
        let Self {
            auth: Unauthenticated,
            base_url,
            alternative_hosts,
            http,
            http_no_sni,
        } = self;
        Client {
            auth: Authenticated {
                account_id,
                cached_auth_token: Arc::new(Mutex::new(None)),
                acquiring_auth_token: tokio::sync::Mutex::new(()),
            },
            base_url,
            alternative_hosts,
            http,
            http_no_sni,
        }
    }
}

impl Client<Authenticated> {
    pub fn new(
        base_url: impl ToString,
        alternative_hosts: Vec<String>,
        account_id: AccountId,
        user_agent: &str,
        #[cfg(not(any(target_os = "android", target_os = "windows")))] network_interface: Option<&str>,
        #[cfg(any(target_os = "android", target_os = "windows"))] network_interface: Option<std::net::IpAddr>,
        #[cfg(target_os = "linux")] so_mark: Option<u32>,
        resolver: Option<Arc<dyn Resolve>>,
    ) -> anyhow::Result<Self> {
        Ok(Client::new_unauthenticated(
            base_url,
            alternative_hosts,
            user_agent,
            network_interface,
            #[cfg(target_os = "linux")]
            so_mark,
            resolver,
        )?
        .into_authenticated(account_id))
    }

    pub async fn acquire_auth_token(&self) -> Result<AcquireToken2Output, ClientError> {
        self.acquire_auth_token_for(&self.auth).await
    }

    pub fn get_auth_token(&self) -> Option<AuthToken> {
        self.auth.get_auth_token()
    }

    pub fn set_auth_token(&self, token: Option<AuthToken>) {
        self.auth.set_auth_token(token);
    }
}

impl<A> Client<A> {
    pub async fn run<C>(&self, cmd: C) -> Result<C::Output, ClientError>
    where
        C: Cmd,
        A: CanRun<C::Auth>,
    {
        self.run_impl(cmd, None)
            .await?
            .into_body()
            .ok_or(anyhow::Error::msg("Non-conditional request has no body.").into())
    }

    pub async fn run_with_etag<C>(&self, cmd: C, etag: Option<&[u8]>) -> Result<Response<C::Output>, ClientError>
    where
        C: ETagCmd,
        A: CanRun<C::Auth>,
    {
        self.run_impl(cmd, etag).await
    }

    async fn run_impl<C>(&self, cmd: C, etag: Option<&[u8]>) -> Result<Response<C::Output>, ClientError>
    where
        C: Cmd,
        A: CanRun<C::Auth>,
    {
        let etag = etag
            .map(HeaderValue::from_bytes)
            .transpose()
            .map_err(|_| ClientError::InvalidHeaderValue)?;
        let request = self.to_request(&cmd, etag)?;
        let Some(auth) = self.auth.as_authenticated() else {
            let res = self.send_http(request).await?;
            return parse_response(res).await;
        };
        for _ in 0..3 {
            let auth_token = self.acquire_auth_token_for(auth).await?.auth_token;
            if let Some(output) = self.run_once::<C>(request.clone(), &auth_token).await? {
                return Ok(output);
            }
            auth.clear_auth_token(auth_token);
        }
        Err(anyhow!("repeatedly acquired invalid auth token").into())
    }

    // Sends the http request and maps expected error codes to client errors.
    // Returns `Ok(None)` if the auth token is invalid, because this error shouldn't bubble up.
    async fn run_once<C: Cmd>(&self, mut request: http::Request<String>, auth_token: &AuthToken) -> Result<Option<Response<C::Output>>, ClientError> {
        request.headers_mut().insert(
            http::header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", auth_token.as_str())).context("invalid auth token header")?,
        );
        let res = self.send_http(request).await?;
        match parse_response(res).await {
            Ok(output) => Ok(Some(output)),
            Err(ClientError::ApiError(error)) => match error.body.error {
                ApiErrorKind::MissingOrInvalidAuthToken {} => {
                    tracing::warn!(
                        message_id = "Oroo1Xah",
                        http.response.status_code = error.status.as_u16(),
                        detail = error.body.detail,
                        error = error.body.msg,
                        error.type = "MissingOrInvalidAuthToken",
                        "Auth token rejected."
                    );
                    Ok(None)
                }
                _ => Err(ClientError::ApiError(error)),
            },
            Err(err) => Err(err),
        }
    }

    fn to_request<C: Cmd>(&self, cmd: &C, etag: Option<HeaderValue>) -> anyhow::Result<http::Request<String>> {
        let url = Url::parse(&self.base_url)?.join(C::PATH)?;
        let mut request = http::Request::builder()
            .method(C::METHOD)
            .uri(url.as_str())
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(String::new())?;
        if let Some(etag) = etag {
            request.headers_mut().insert(http::header::IF_NONE_MATCH, etag);
        }
        if C::METHOD == http::Method::GET {
            // TODO: Support GET parameters: https://linear.app/soveng/issue/OBS-2002/support-get-query-parameters-in-rust-api-client
        } else {
            *request.body_mut() = serde_json::to_string(cmd)?;
        }
        Ok(request)
    }

    async fn acquire_auth_token_for(&self, auth: &Authenticated) -> Result<AcquireToken2Output, ClientError> {
        if let Some(auth_token) = auth.get_auth_token() {
            return Ok(AcquireToken2Output {
                auth_token,
                url_override: None,
            });
        }

        let acquiring_auth_token = auth.acquiring_auth_token.lock().await;

        if let Some(auth_token) = auth.get_auth_token() {
            return Ok(AcquireToken2Output {
                auth_token,
                url_override: None,
            });
        }

        tracing::info!(message_id = "xah8Usoo", "Acquiring auth token.");

        let res = self.request_token(&auth.account_id).await?;
        let body = res.into_body().context("No auth token in response")?;
        auth.set_auth_token(Some(body.auth_token.clone()));

        drop(acquiring_auth_token);
        Ok(body)
    }

    async fn request_token(&self, account_id: &AccountId) -> Result<Response<AcquireToken2Output>, ClientError> {
        let run_request = async |pow: Option<PowOutput>| {
            let request = AcquireToken {
                account_id: account_id.clone(),
                pow,
            }
            .to_request2(&self.base_url)?;
            let res = self.send_http(request).await?;
            parse_response::<AcquireToken2Output>(res).await
        };
        let mut pow = None;
        for _ in 0..2 {
            match run_request(pow).await {
                Err(ClientError::ApiError(ApiError {
                    status: _,
                    body:
                        ApiErrorBody {
                            error:
                                ApiErrorKind::RateLimitExceeded {
                                    pow_challenge: Some(pow_challenge),
                                }
                                | ApiErrorKind::SignupLimitExceeded {
                                    pow_challenge: Some(pow_challenge),
                                },
                            msg: _,
                            detail: _,
                        },
                })) => {
                    tracing::warn!(
                        message_id = "Kp9xZ2mW",
                        puzzles = pow_challenge.puzzles,
                        threshold = %hex::encode(pow_challenge.threshold.0),
                        "rate limited, solving proof of work",
                    );
                    let solved = tokio::time::timeout(POW_SOLVE_TIMEOUT, pow_challenge.solve())
                        .await
                        .map_err(|_| ClientError::ProofOfWorkTimeout)?
                        .map_err(ClientError::ProofOfWork)?;
                    tracing::info!(message_id = "Tg4nB7vC", "proof of work solved");
                    pow = Some(solved);
                }
                res => return res,
            }
        }
        run_request(pow).await
    }

    fn http_client_builder(
        user_agent: &str,
        rustls_config: rustls::ClientConfig,
        #[cfg(not(any(target_os = "android", target_os = "windows")))] network_interface: Option<&str>,
        #[cfg(any(target_os = "android", target_os = "windows"))] network_interface: Option<std::net::IpAddr>,
        #[cfg(target_os = "linux")] so_mark: Option<u32>,
        resolver: Option<Arc<dyn Resolve>>,
    ) -> anyhow::Result<reqwest::Client> {
        let builder = ClientBuilder::new()
            .timeout(Duration::from_secs(60))
            .read_timeout(Duration::from_secs(10))
            .user_agent(user_agent)
            .use_preconfigured_tls(rustls_config);
        let builder = match resolver {
            None => builder,
            Some(resolver) => builder.dns_resolver(resolver),
        };
        let builder = match network_interface {
            None => builder,
            #[cfg(not(any(target_os = "android", target_os = "windows")))]
            Some(network_interface) => builder.interface(network_interface),
            #[cfg(any(target_os = "android", target_os = "windows"))]
            Some(network_interface) => builder.local_address(network_interface),
        };
        #[cfg(target_os = "linux")]
        let builder = match so_mark {
            None => builder,
            Some(so_mark) => builder.so_mark(so_mark),
        };
        builder.build().context("failed to initialize HTTP client")
    }

    async fn send_http(&self, request: http::Request<String>) -> Result<reqwest::Response, ClientError> {
        let reqwest_request: reqwest::Request = request.clone().try_into().context("could not construct reqwest::Request")?;
        let first_error = match self.http.execute(reqwest_request).await {
            Ok(resp) => return Ok(resp),
            Err(error) if error.is_connect() || error.is_timeout() => {
                tracing::error!(
                    message_id = "XfTLkg6w",
                    ?error,
                    "error executing request, maybe blocked, trying again with alternative hosts",
                );
                error
            }
            Err(error) => return Err(error.into()),
        };

        for host in &self.alternative_hosts {
            let mut reqwest_request: reqwest::Request = request.clone().try_into().context("could not construct reqwest::Request")?;
            if let Err(error) = reqwest_request.url_mut().set_host(Some(host.as_str())) {
                tracing::error!(message_id = "6mXOeRSL", host, ?error, "failed to set alternative host on request");
                continue;
            };
            match self.http.execute(reqwest_request).await {
                Ok(resp) => return Ok(resp),
                Err(error) if error.is_connect() || error.is_timeout() => tracing::error!(
                    message_id = "m6JLZaYN",
                    host,
                    ?error,
                    "error executing request with alternative host, maybe blocked"
                ),
                Err(error) => return Err(error.into()),
            }
        }

        tracing::error!("all attempts to evade blocks using alternative hosts failed, trying again without SNI");
        let reqwest_request: reqwest::Request = request.clone().try_into().context("could not construct reqwest::Request")?;
        match self.http_no_sni.execute(reqwest_request).await {
            Ok(resp) => return Ok(resp),
            Err(error) if error.is_connect() => tracing::error!(
                message_id = "CttGsdTj",
                ?error,
                "error executing request without SNI, maybe blocked, trying again with alternative hosts"
            ),
            Err(error) => return Err(error.into()),
        }

        tracing::error!(message_id = "QmAdyhxm", "all attempts to evade blocks failed, returning original error");
        Err(first_error.into())
    }

    fn rustls_config(server_name_for_cert_verification: impl IntoIterator<Item = String>) -> anyhow::Result<rustls::ClientConfig> {
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(VerifyApiServerCert::new(server_name_for_cert_verification)?)
            .with_no_client_auth();
        Ok(crypto)
    }
}

#[derive(Debug)]
struct VerifyApiServerCert {
    server_names: Vec<rustls::pki_types::ServerName<'static>>,
    web_pki_server_verifier: Arc<dyn rustls::client::danger::ServerCertVerifier>,
}

impl VerifyApiServerCert {
    fn new(server_names: impl IntoIterator<Item = String>) -> anyhow::Result<Arc<Self>> {
        let roots = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        let web_pki_server_verifier = WebPkiServerVerifier::builder(roots.into()).build()?;
        let server_names = server_names.into_iter().map(rustls::pki_types::ServerName::try_from).try_collect()?;
        Ok(Arc::new(Self {
            server_names,
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
        use rustls::CertificateError::{NotValidForName, NotValidForNameContext};
        use rustls::Error::InvalidCertificate;
        use x509_parser::prelude::*;

        let mut presented_server_names: Option<Vec<String>> = None;
        if let Ok((_, cert)) = X509Certificate::from_der(end_entity) {
            if let Ok(Some(sa_names)) = cert.subject_alternative_name() {
                for sa_name in &sa_names.value.general_names {
                    if let GeneralName::DNSName(dns_name) = sa_name {
                        presented_server_names.get_or_insert_default().push(dns_name.to_string());
                    }
                }
            };
        }

        let mut first_non_server_name_error = None;
        for verify_server_name in self.server_names.iter() {
            let result = self
                .web_pki_server_verifier
                .verify_server_cert(end_entity, intermediates, verify_server_name, ocsp_response, now);
            match result {
                Ok(result) => {
                    tracing::info!(
                        message_id = "BN7vczBq",
                        verify_server_name = &*verify_server_name.to_str(),
                        request_server_name = &*server_name.to_str(),
                        presented_server_names = ?presented_server_names,
                        "certificate valid",
                    );
                    return Ok(result);
                }
                Err(InvalidCertificate(NotValidForName | NotValidForNameContext { .. })) => {
                    tracing::info!(
                        message_id = "UZEx21nI",
                        verify_server_name = &*verify_server_name.to_str(),
                        request_server_name = &*server_name.to_str(),
                        presented_server_names = ?presented_server_names,
                        "certificate not valid for server name",
                    );
                }
                Err(error) => {
                    tracing::error!(
                        message_id = "eB8okNs3",
                        ?error,
                        verify_server_name = &*verify_server_name.to_str(),
                        request_server_name = &*server_name.to_str(),
                        presented_server_names = ?presented_server_names,
                    "failed to verify server certificate",
                    );
                    first_non_server_name_error = Some(error);
                }
            }
        }
        Err(first_non_server_name_error.unwrap_or(InvalidCertificate(NotValidForName)))
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
