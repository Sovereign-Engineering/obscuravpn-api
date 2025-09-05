use crate::types::SubscriptionTarget;
use serde::{Deserialize, Serialize};

use super::Cmd;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewsletterSubscribe {
    /// DEPRECATED, use `target`.
    pub email: Option<String>,

    /// Required, optional for migration only.
    pub target: Option<SubscriptionTarget>,

    /// Set of subscriptions to subscribe to, they are identified by strings.
    ///
    /// This is required, for legacy reasons if not provided or empty it defaults to the general newsletter.
    #[serde(default)]
    pub subscriptions: Vec<String>,
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
                "email": null,
                "subscriptions": [
                    "blog"
                ],
                "target": {
                    "type": "email",
                    "addr": "me@example"
                }
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

    crate::cmd::check_cmd_json::<NewsletterSubscribe>(
        Some(
            r#"
            {
                "email": null,
                "subscriptions": [
                    "platform-windows",
                    "platform-android"
                ],
                "target": {
                    "type": "nostr",
                    "addr": "me@example"
                }
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

    crate::cmd::check_cmd_json::<NewsletterSubscribe>(
        Some(
            r#"
            {
                "email": "me@example",
                "subscriptions": [],
                "target": null
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
