use crate::cmd::{parse_response, ApiError, ApiErrorKind, Cmd, ProtocolError};
use crate::token::AcquireToken;
use crate::types::AuthToken;
use anyhow::{anyhow, Context};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug)]
pub struct Client {
    account_id: String,
    base_url: String,
    http: reqwest::Client,
    cached_auth_token: Arc<Mutex<Option<AuthToken>>>,
    acquiring_auth_token: tokio::sync::Mutex<()>,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("API Error: {0}")]
    ApiError(#[from] ApiError),
    /// We got a response but it wasn't the expected format.
    ///
    /// Most likely a response from a proxy or similar.
    #[error("Protocol Error: {0}")]
    ProtocolError(#[from] ProtocolError),
    #[error("request processing error: {:?}", .0)]
    Other(#[from] anyhow::Error),
}

impl Client {
    pub fn new(base_url: impl ToString, account_id: String, user_agent: &str) -> anyhow::Result<Self> {
        let mut base_url = base_url.to_string();
        if !base_url.ends_with('/') {
            base_url += "/"
        }
        Ok(Self {
            account_id,
            base_url,
            cached_auth_token: Arc::new(Mutex::new(None)),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .read_timeout(Duration::from_secs(10))
                .user_agent(user_agent)
                .build()
                .context("failed to initialize HTTP client")?,
            acquiring_auth_token: tokio::sync::Mutex::new(()),
        })
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
        let auth_token: String = parse_response(res).await?;
        let auth_token: AuthToken = auth_token.into();
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

    async fn send_http(&self, request: http::Request<String>) -> anyhow::Result<reqwest::Response> {
        let request = request.try_into().context("could not construct reqwest::Request")?;
        self.http.execute(request).await.context("error executing request")
    }

    pub async fn run<C: Cmd>(&self, cmd: C) -> Result<C::Output, ClientError> {
        for _ in 0..3 {
            let auth_token = self.acquire_auth_token().await?;
            if let Some(output) = self.try_run::<C>(&cmd, &auth_token).await? {
                return Ok(output);
            }
            self.clear_auth_token(auth_token);
        }

        Err(anyhow!("repeatedly acquired invalid auth token").into())
    }

    // Sends the http request and maps expected error codes to client errors.
    // Returns `Ok(None)` if the auth token is invalid, because this error shouldn't bubble up.
    async fn try_run<C: Cmd>(&self, body: &C, auth_token: &AuthToken) -> Result<Option<C::Output>, ClientError> {
        let request = body.to_request(&self.base_url, auth_token)?;
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
}
