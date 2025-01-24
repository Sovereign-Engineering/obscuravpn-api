use serde::{Deserialize, Serialize};

use super::Cmd;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewsletterUnsubscribe {
    pub id: u64,
    pub unsubscribe_secret: String,
}

impl Cmd for NewsletterUnsubscribe {
    type Output = ();
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "newsletter/unsubscribe";
}

#[test]
fn test_json() {
    crate::cmd::check_cmd_json::<NewsletterUnsubscribe>(
        Some(
            r#"
            {
                "id": 37,
                "unsubscribe_secret": "29dcf763-962f-4f6f-8aa0-98cd751bd208"
            }
            "#,
        ),
        None,
    );
}
