use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceStatus {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceStatusOutput {
    pub free_trial: FreeTrialStatus,
    pub nnn: NnnStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FreeTrialStatus {
    pub available: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NnnStatus {
    pub claimed_months_target: u64,
    pub claimed_months_total: u64,
    pub submissions_open: bool,
}

impl Cmd for ServiceStatus {
    type Output = ServiceStatusOutput;
    const METHOD: http::Method = http::Method::GET;
    const PATH: &'static str = "status";
}

#[test]
fn test_json() {
    crate::cmd::check_cmd_json::<ServiceStatus>(
        None,
        Some(
            r#"
            {
                "free_trial": {
                    "available": true
                },
                "nnn": {
                    "claimed_months_target": 100000,
                    "claimed_months_total": 1234,
                    "submissions_open": true
                }
            }
            "#,
        ),
    );
}
