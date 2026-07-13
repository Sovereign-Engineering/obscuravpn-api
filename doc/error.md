# Obscura API Errors

When something goes wrong the API returns an error following HTTP conventions. 4xx for client errors and 5xx for server errors.

The error body will contain an [`ApiErrorBody`](crate::cmd::ApiErrorBody) structure.

In general 4xx errors should not be retried without taking corrective action. 5xx errors can be retried with exponential backoff and jitter. Each error code has documentation about acceptable retry behaviour.
