use serde::{Deserialize, Serialize};

use super::Cmd;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewsletterSubscribe {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewsletterSubscribeResponse {
    pub id: u64,
    pub unsubscribe_secret: String,
}

impl Cmd for NewsletterSubscribe {
    type Output = NewsletterSubscribeResponse;
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "newsletter/subscribe";
}

#[test]
fn test_json() {
    crate::cmd::check_cmd_json::<NewsletterSubscribe>(
        Some(
            r#"
            {
                "email": "me@example"
            }
            "#,
        ),
        Some(
            r#"
            {
                "id": 37,
                "unsubscribe_secret": "29dcf763-962f-4f6f-8aa0-98cd751bd208"
            }
            "#,
        ),
    );
}
