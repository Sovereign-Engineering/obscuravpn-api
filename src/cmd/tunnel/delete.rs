use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteTunnel {
    pub id: String,
}
impl Cmd for DeleteTunnel {
    type Output = ();
    const METHOD: http::Method = http::Method::DELETE;
    const PATH: &'static str = super::PATH;
}

#[test]
fn test_json() {
    let cmd_json = r#"
    {
        "id": "2c4ca7c0-90f0-4e28-aa0c-c656a5127189"
    }
    "#;
    crate::cmd::check_cmd_json::<DeleteTunnel>(Some(cmd_json), None);
}
