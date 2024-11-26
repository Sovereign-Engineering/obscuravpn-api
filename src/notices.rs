use std::convert::TryFrom;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use itertools::Itertools as _;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct NoticeResp {
    notices: Vec<NoticeRaw>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NoticeRaw {
    message: String,
    version_req: Option<String>,
    warn_at: Option<u64>,
    error_at: Option<u64>,
}

struct Notice {
    message: String,
    version_req: Option<semver::VersionReq>,
    warn_at: Option<SystemTime>,
    error_at: Option<SystemTime>,
}

fn epoch_secs_to_systemtime(epoch_secs: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(epoch_secs)
}

impl TryFrom<NoticeRaw> for Notice {
    type Error = anyhow::Error;

    fn try_from(value: NoticeRaw) -> Result<Self, Self::Error> {
        let version_req: Option<semver::VersionReq> = value.version_req.map(|vr| vr.parse()).transpose()?;
        let warn_at = value.warn_at.map(epoch_secs_to_systemtime);
        let error_at = value.error_at.map(epoch_secs_to_systemtime);
        let rv = Self {
            message: value.message,
            version_req,
            warn_at,
            error_at,
        };
        Ok(rv)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", content = "content")]
pub enum NoticeDisplay {
    Warn(String),
    Error(String),
}

pub struct NoticesClient {
    client: reqwest::Client,
    full_url: String,
}

const NOTICES_PATH_FROM_BASE: &str = "notices";

impl NoticesClient {
    pub fn new(base_url: impl ToString) -> Self {
        let full_url = {
            let mut base_url = base_url.to_string();
            if !base_url.ends_with('/') {
                base_url += "/"
            }
            base_url + NOTICES_PATH_FROM_BASE
        };
        Self {
            client: reqwest::Client::new(),
            full_url,
        }
    }

    pub async fn current_notices(&self, version: &semver::Version) -> anyhow::Result<Vec<NoticeDisplay>> {
        let resp = self.client.get(&self.full_url).send().await?;
        if let Err(error) = resp.error_for_status_ref() {
            if let Some(reqwest::StatusCode::NOT_FOUND) = error.status() {
                // 404 means no notices
                return Ok(vec![]);
            }
            return Err(error.into());
        }
        let notice_resp = resp.json::<NoticeResp>().await?;
        let notices: Vec<Notice> = notice_resp.notices.into_iter().map(|nr| Notice::try_from(nr)).try_collect()?;
        let rv = notices.into_iter().flat_map(|n| n.into_display(version)).collect();
        Ok(rv)
    }
}

impl Notice {
    fn into_display(self, version: &semver::Version) -> Option<NoticeDisplay> {
        if let Some(version_req) = self.version_req {
            if !version_req.matches(version) {
                return None;
            }
        }
        let now = SystemTime::now();
        if let Some(error_at) = self.error_at {
            if error_at <= now {
                return Some(NoticeDisplay::Error(self.message));
            }
        }
        if let Some(warn_at) = self.warn_at {
            if warn_at <= now {
                return Some(NoticeDisplay::Warn(self.message));
            }
        }
        None
    }
}
