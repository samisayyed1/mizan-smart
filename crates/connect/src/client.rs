//! HTTP client for Mizan Connect cloud API.
//!
//! This module provides a shared HTTP client for communicating with the
//! Mizan Connect cloud service. Both Tauri and server implementations
//! should use this client to ensure consistency.

use async_trait::async_trait;
use log::{debug, info};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::broker::{
    BrokerAccount, BrokerBrokerage, BrokerConnection, BrokerConnectionBrokerage,
    BrokerHoldingsResponse, PaginatedUniversalActivity, PlansResponse, UserInfo, UserTeam,
};
use mizan_core::errors::{Error, Result};

use super::broker::BrokerApiClient;

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default base URL for Mizan Connect cloud service.
pub const DEFAULT_CLOUD_API_URL: &str = "https://api.mizan.app";

// ─────────────────────────────────────────────────────────────────────────────
// API Response Types (internal, for parsing cloud API responses)
// ─────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct ApiConnection {
    id: String,
    authorization_id: Option<String>,
    brokerage_name: Option<String>,
    brokerage_slug: Option<String>,
    brokerage: Option<ApiBrokerage>,
    disabled: Option<bool>,
    updated_at: Option<String>,
    name: Option<String>,
    status: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct ApiBrokerage {
    id: Option<String>,
    name: Option<String>,
    display_name: Option<String>,
    slug: Option<String>,
    aws_s3_logo_url: Option<String>,
    aws_s3_square_logo_url: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiUser {
    id: String,
    email: Option<String>,
    full_name: Option<String>,
    avatar_url: Option<String>,
    locale: Option<String>,
    week_starts_on_monday: Option<bool>,
    timezone: Option<String>,
    timezone_auto_sync: Option<bool>,
    time_format: Option<i32>,
    date_format: Option<String>,
    team_id: Option<String>,
    team_role: Option<String>,
    team: Option<ApiTeam>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiTeam {
    id: String,
    name: Option<String>,
    logo_url: Option<String>,
    plan: Option<String>,
    #[serde(default)]
    subscription_status: Option<String>,
    #[serde(default)]
    subscription_current_period_end: Option<String>,
    #[serde(default)]
    subscription_cancel_at_period_end: Option<bool>,
    #[serde(default)]
    canceled_at: Option<String>,
    #[serde(default)]
    country_code: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct ApiErrorResponse {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

/// Response of `POST /api/v1/sync/brokerage/login-portal`.
///
/// `url` is a one-time SnapTrade Connection Portal URL (5-minute TTL on
/// SnapTrade's side); `expires_at` is RFC 3339 and surfaces the same TTL
/// to callers so they can pre-empt expiry rather than guessing.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LoginPortalResponse {
    pub url: String,
    pub expires_at: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Connect API Client
// ─────────────────────────────────────────────────────────────────────────────

/// HTTP client for the Mizan Connect cloud API.
///
/// This client provides methods for:
/// - Fetching broker connections, accounts, and activities
/// - Getting subscription plans
/// - Getting user information
///
/// # Example
///
/// ```ignore
/// let client = ConnectApiClient::new("https://api.mizan.app", "your-token")?;
/// let connections = client.list_connections().await?;
/// ```
#[derive(Debug, Clone)]
pub struct ConnectApiClient {
    client: reqwest::Client,
    base_url: String,
    auth_header: HeaderValue,
}

impl ConnectApiClient {
    /// Create a new Connect API client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the cloud API (e.g., "https://api.mizan.app")
    /// * `access_token` - A valid JWT access token
    ///
    /// # Errors
    ///
    /// Returns an error if the access token format is invalid or the HTTP client
    /// cannot be initialized.
    pub fn new(base_url: &str, access_token: &str) -> Result<Self> {
        let auth_header = HeaderValue::from_str(&format!("Bearer {}", access_token))
            .map_err(|e| Error::Unexpected(format!("Invalid access token format: {}", e)))?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(|e| Error::Unexpected(format!("Failed to initialize HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_header,
        })
    }

    /// Create default headers for API requests.
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, self.auth_header.clone());
        headers
    }

    /// Make a GET request and parse the response.
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| Error::Unexpected(format!("Request failed: {}", e)))?;

        self.parse_response(response).await
    }

    /// Parse an HTTP response, handling errors appropriately.
    async fn parse_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| Error::Unexpected(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            // Try to parse error response for a better message
            if let Ok(err) = serde_json::from_str::<ApiErrorResponse>(&body) {
                let msg = err
                    .message
                    .or(err.error)
                    .unwrap_or_else(|| format!("HTTP {}", status));
                return Err(Error::Unexpected(format!("API error: {}", msg)));
            }
            return Err(Error::Unexpected(format!(
                "API error {}: {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }

        serde_json::from_str(&body)
            .map_err(|e| Error::Unexpected(format!("Failed to parse response: {} - {}", e, body)))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Brokerage Endpoints
    // ─────────────────────────────────────────────────────────────────────────

    /// Fetch account activities with pagination.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The broker account ID (provider's ID)
    /// * `start_date` - Optional start date filter (YYYY-MM-DD)
    /// * `end_date` - Optional end date filter (YYYY-MM-DD)
    /// * `offset` - Pagination offset
    /// * `limit` - Maximum number of results per page
    pub async fn get_account_activities(
        &self,
        account_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<PaginatedUniversalActivity> {
        let mut url = format!(
            "{}/api/v1/sync/brokerage/accounts/{}/activities",
            self.base_url, account_id
        );

        // Build query parameters
        let mut params = Vec::new();
        if let Some(v) = offset {
            params.push(format!("offset={}", v));
        }
        if let Some(v) = limit {
            params.push(format!("limit={}", v));
        }
        if let Some(v) = start_date {
            params.push(format!("start_date={}", v));
        }
        if let Some(v) = end_date {
            params.push(format!("end_date={}", v));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        debug!("[ConnectApi] Fetching activities from: {}", url);

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| Error::Unexpected(format!("Failed to fetch activities: {}", e)))?;

        self.parse_response(response).await
    }

    /// Delete (disconnect) a broker authorization.
    ///
    /// Calls the Mizan Connect backend's `DELETE /api/v1/sync/brokerage/connections/{id}`.
    /// The backend revokes the upstream SnapTrade authorization and soft-deletes
    /// the local broker_connections row. Idempotent — repeat deletes return 200.
    ///
    /// # Arguments
    ///
    /// * `connection_id` - SnapTrade authorization (connection) UUID, the
    ///   value of `BrokerConnection.id` returned by `list_connections`.
    pub async fn delete_connection(&self, connection_id: &str) -> Result<()> {
        let url = format!(
            "{}/api/v1/sync/brokerage/connections/{}",
            self.base_url, connection_id
        );

        debug!("[ConnectApi] Disconnecting broker: {}", url);

        let response = self
            .client
            .delete(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| Error::Unexpected(format!("Failed to disconnect broker: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Unexpected(format!(
                "Disconnect failed {}: {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }
        Ok(())
    }

    /// Fetch current holdings for a broker account.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The broker account ID (provider's ID)
    pub async fn get_account_holdings(&self, account_id: &str) -> Result<BrokerHoldingsResponse> {
        let url = format!(
            "{}/api/v1/sync/brokerage/accounts/{}/holdings",
            self.base_url, account_id
        );

        debug!("[ConnectApi] Fetching holdings from: {}", url);

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| Error::Unexpected(format!("Failed to fetch holdings: {}", e)))?;

        self.parse_response(response).await
    }

    // ─────────────────────────────────────────────────────────────────────────
    // User & Subscription Endpoints
    // ─────────────────────────────────────────────────────────────────────────

    /// Get the current user's information.
    pub async fn get_user_info(&self) -> Result<UserInfo> {
        let api_user: Option<ApiUser> = self.get("/api/v1/user/me").await?;

        let user =
            api_user.ok_or_else(|| Error::Unexpected("No user info returned".to_string()))?;

        Ok(UserInfo {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            locale: user.locale,
            week_starts_on_monday: user.week_starts_on_monday,
            timezone: user.timezone,
            timezone_auto_sync: user.timezone_auto_sync,
            time_format: user.time_format,
            date_format: user.date_format,
            team_id: user.team_id,
            team_role: user.team_role,
            team: user.team.map(|t| UserTeam {
                id: t.id,
                name: t.name.unwrap_or_default(),
                logo_url: t.logo_url,
                plan: t.plan,
                subscription_status: t.subscription_status,
                subscription_current_period_end: t.subscription_current_period_end,
                subscription_cancel_at_period_end: t.subscription_cancel_at_period_end,
                canceled_at: t.canceled_at,
                country_code: t.country_code,
                created_at: t.created_at,
            }),
        })
    }

    /// Get available subscription plans (authenticated).
    pub async fn get_subscription_plans(&self) -> Result<PlansResponse> {
        self.get("/api/v1/subscription/plans").await
    }

    /// Create a one-time SnapTrade Connection Portal URL for the current user.
    ///
    /// `POST /api/v1/sync/brokerage/login-portal`. The backend lazily
    /// registers the user with SnapTrade on first call and returns a
    /// portal URL with an embedded signed `state` token bound to the
    /// caller's local user id. The user opens the URL in their default
    /// browser, authenticates with their broker, and SnapTrade redirects
    /// back to the backend's callback handler — no further desktop
    /// involvement until the connection lands in
    /// `list_brokerage_connections`.
    ///
    /// The backend rate-limits this endpoint at 10 requests per hour per
    /// user (a 429 is mapped to [`Error::Unexpected`] like any other
    /// upstream failure).
    ///
    /// # Arguments
    /// * `broker` — optional broker slug (e.g. `"ROBINHOOD"`) to deep-link
    ///   straight into a single broker. `None` shows SnapTrade's broker
    ///   picker.
    pub async fn create_login_portal(&self, broker: Option<&str>) -> Result<LoginPortalResponse> {
        let url = format!("{}/api/v1/sync/brokerage/login-portal", self.base_url);
        let body = serde_json::json!({ "broker": broker });

        debug!("[ConnectApi] POST {}", url);
        let response = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Unexpected(format!("Login-portal request failed: {}", e)))?;

        self.parse_response(response).await
    }

    /// Check if the current user's plan includes broker sync.
    ///
    /// Returns true when the user has an active subscription AND their plan
    /// is not "basic" (basic plan only includes device sync).
    ///
    /// **Chunk-3 dev escape hatch**: when `CONNECT_BYPASS_PLAN_CHECK=true`
    /// is set, this short-circuits to `Ok(true)` without hitting
    /// `/api/v1/user/me`. The Mizan Connect backend's `/v1/me` does not
    /// yet return a `team` field — that lands with Chunk 4 (Stripe). The
    /// flag lets the broker UI work end-to-end against Chunk-3 backends
    /// in dev, and Chunk 4 will remove it once `team.plan` is populated.
    pub async fn has_broker_sync(&self) -> Result<bool> {
        // TODO(chunk-4): drop this bypass once /api/v1/user/me returns team.plan.
        //
        // The bypass needs to work in production .dmg/.msi/.AppImage installs
        // where there is no shell env at runtime (apps launched from Finder
        // or Start Menu inherit no `CONNECT_BYPASS_*`). So we accept BOTH:
        //   - compile-time value via `option_env!()` — set by CI's env block
        //     so the GitHub Actions Variable bakes into release builds.
        //   - runtime value via `std::env::var()` — used by tests and by
        //     `pnpm tauri dev` shells that export the var locally.
        const COMPILE_TIME_BYPASS: Option<&str> = option_env!("CONNECT_BYPASS_PLAN_CHECK");
        let runtime_bypass = std::env::var("CONNECT_BYPASS_PLAN_CHECK").ok();
        if COMPILE_TIME_BYPASS == Some("true") || runtime_bypass.as_deref() == Some("true") {
            debug!("[ConnectApi] CONNECT_BYPASS_PLAN_CHECK=true — skipping plan gate");
            return Ok(true);
        }

        let user_info = self.get_user_info().await?;

        let team = match &user_info.team {
            Some(t) => t,
            None => {
                debug!("[ConnectApi] No team info, broker sync not available");
                return Ok(false);
            }
        };

        let is_active = matches!(
            team.subscription_status.as_deref(),
            Some("active") | Some("trialing")
        );
        if !is_active {
            debug!("[ConnectApi] No active subscription, broker sync not available");
            return Ok(false);
        }

        let plan = match team.plan.as_deref() {
            Some(p) => p,
            None => {
                debug!("[ConnectApi] No plan metadata, broker sync not available");
                return Ok(false);
            }
        };
        if plan == "basic" {
            debug!("[ConnectApi] Basic plan, broker sync not available");
            return Ok(false);
        }

        debug!("[ConnectApi] Broker sync is available");
        Ok(true)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BrokerApiClient Trait Implementation
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl BrokerApiClient for ConnectApiClient {
    /// Fetch all broker connections for the user.
    async fn list_connections(&self) -> Result<Vec<BrokerConnection>> {
        // First, get the raw response to log it
        let url = format!("{}/api/v1/sync/brokerage/connections", self.base_url);
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| Error::Unexpected(format!("Request failed: {}", e)))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| Error::Unexpected(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Unexpected(format!(
                "API error {}: {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }

        // The Mizan Connect backend serves /api/v1/sync/brokerage/connections
        // as a top-level JSON array (`Json(Vec<BrokerConnectionDto>)`), not a
        // `{ "connections": [...] }` envelope. Earlier client versions targeted
        // a wrapper shape; mismatch silently broke the connection refresh path.
        let raw: Vec<ApiConnection> = serde_json::from_str(&body).map_err(|e| {
            Error::Unexpected(format!("Failed to parse connections: {} - {}", e, body))
        })?;

        let connections: Vec<BrokerConnection> = raw
            .into_iter()
            .map(|c| {
                // Use brokerage object if present, otherwise use top-level fields
                let brokerage = c.brokerage.map(|b| BrokerConnectionBrokerage {
                    id: b.id,
                    slug: b.slug,
                    name: b.name.clone(),
                    display_name: b.display_name.or(b.name),
                    aws_s3_logo_url: b.aws_s3_logo_url,
                    aws_s3_square_logo_url: b.aws_s3_square_logo_url,
                });

                let brokerage = brokerage.or_else(|| {
                    if c.brokerage_name.is_some() || c.brokerage_slug.is_some() {
                        Some(BrokerConnectionBrokerage {
                            id: None,
                            slug: c.brokerage_slug,
                            name: c.brokerage_name.clone(),
                            display_name: c.brokerage_name,
                            aws_s3_logo_url: None,
                            aws_s3_square_logo_url: None,
                        })
                    } else {
                        None
                    }
                });

                BrokerConnection {
                    id: c.authorization_id.unwrap_or(c.id),
                    brokerage,
                    connection_type: None,
                    status: c.status,
                    disabled: c.disabled.unwrap_or(false),
                    disabled_date: None,
                    updated_at: c.updated_at,
                    name: c.name,
                }
            })
            .collect();

        let count = connections.len();
        info!("[ConnectApi] Fetched {} broker connections", count);
        Ok(connections)
    }

    /// Fetch all broker accounts for the user.
    async fn list_accounts(
        &self,
        _authorization_ids: Option<Vec<String>>,
    ) -> Result<Vec<BrokerAccount>> {
        // First, get the raw response to log it
        let url = format!("{}/api/v1/sync/brokerage/accounts", self.base_url);
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| Error::Unexpected(format!("Request failed: {}", e)))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| Error::Unexpected(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Unexpected(format!(
                "API error {}: {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }

        // Same envelope shape as /connections: backend returns a top-level
        // JSON array (`Json(Vec<BrokerAccountDto>)`), not `{ "accounts": [...] }`.
        let accounts: Vec<BrokerAccount> = serde_json::from_str(&body).map_err(|e| {
            Error::Unexpected(format!("Failed to parse accounts: {} - {}", e, body))
        })?;

        Ok(accounts)
    }

    /// Fetch all available brokerages (not implemented in REST API).
    async fn list_brokerages(&self) -> Result<Vec<BrokerBrokerage>> {
        // The REST API doesn't have a separate brokerages endpoint
        // Brokerages are embedded in connections
        Ok(vec![])
    }

    /// Fetch account activities with pagination.
    async fn get_account_activities(
        &self,
        account_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<PaginatedUniversalActivity> {
        // Delegate to the inherent method
        ConnectApiClient::get_account_activities(
            self, account_id, start_date, end_date, offset, limit,
        )
        .await
    }

    /// Fetch current holdings for a broker account.
    async fn get_account_holdings(&self, account_id: &str) -> Result<BrokerHoldingsResponse> {
        // Delegate to the inherent method
        ConnectApiClient::get_account_holdings(self, account_id).await
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public (Unauthenticated) API Functions
// ─────────────────────────────────────────────────────────────────────────────

/// Fetch subscription plans without authentication.
///
/// This function is used to display pricing information before the user logs in.
/// The plans endpoint is public and does not require authentication.
///
/// # Arguments
///
/// * `base_url` - The base URL of the cloud API (e.g., "https://api.mizan.app")
///
/// # Example
///
/// ```ignore
/// let plans = fetch_subscription_plans_public("https://api.mizan.app").await?;
/// ```
pub async fn fetch_subscription_plans_public(base_url: &str) -> Result<PlansResponse> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .map_err(|e| Error::Unexpected(format!("Failed to initialize HTTP client: {}", e)))?;

    let base_url = base_url.trim_end_matches('/');
    let url = format!("{}/api/v1/subscription/plans", base_url);

    let response = client
        .get(&url)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .map_err(|e| Error::Unexpected(format!("Request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| Error::Unexpected(format!("Failed to read response: {}", e)))?;

    if !status.is_success() {
        return Err(Error::Unexpected(format!(
            "API error {}: {}",
            status,
            body.chars().take(200).collect::<String>()
        )));
    }

    serde_json::from_str(&body)
        .map_err(|e| Error::Unexpected(format!("Failed to parse plans response: {} - {}", e, body)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_client_creation() {
        let client = ConnectApiClient::new("https://api.mizan.app", "test-token");
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_url_normalization() {
        let client = ConnectApiClient::new("https://api.mizan.app/", "test-token").unwrap();
        assert_eq!(client.base_url, "https://api.mizan.app");
    }

    /// Verifies `create_login_portal`:
    /// - hits POST `/api/v1/sync/brokerage/login-portal`
    /// - sends the user's Bearer access token
    /// - sends `Content-Type: application/json`
    /// - sends `{ "broker": <slug or null> }` as the body
    /// - parses `{ url, expires_at }` into [`LoginPortalResponse`]
    #[tokio::test]
    async fn create_login_portal_signs_request_and_parses_response() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/sync/brokerage/login-portal"))
            .and(header("authorization", "Bearer mizan-test-jwt"))
            .and(header("content-type", "application/json"))
            .and(body_json(serde_json::json!({ "broker": null })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "url": "https://app.snaptrade.com/snapTrade/redeemToken?token=abc",
                "expires_at": "2026-05-05T12:34:56Z",
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = ConnectApiClient::new(&server.uri(), "mizan-test-jwt").unwrap();
        let resp = client
            .create_login_portal(None)
            .await
            .expect("login-portal call");
        assert_eq!(
            resp.url,
            "https://app.snaptrade.com/snapTrade/redeemToken?token=abc"
        );
        assert_eq!(resp.expires_at, "2026-05-05T12:34:56Z");
    }

    /// Same mock, but with `broker = Some("ROBINHOOD")` — confirms the
    /// caller-specified broker slug is passed through verbatim.
    #[tokio::test]
    async fn create_login_portal_passes_broker_slug() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/sync/brokerage/login-portal"))
            .and(body_json(serde_json::json!({ "broker": "ROBINHOOD" })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "url": "https://app.snaptrade.com/x",
                "expires_at": "2026-05-05T12:34:56Z",
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = ConnectApiClient::new(&server.uri(), "mizan-test-jwt").unwrap();
        client
            .create_login_portal(Some("ROBINHOOD"))
            .await
            .expect("login-portal call with broker");
    }

    /// 429 from upstream → `Error::Unexpected` (the standard error wrapper
    /// in this crate). The desktop's adapter layer turns this into a toast.
    #[tokio::test]
    async fn create_login_portal_propagates_rate_limit_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/v1/sync/brokerage/login-portal"))
            .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
                "error": {
                    "code": "too_many_requests",
                    "message": "too many login-portal requests; try again later",
                    "request_id": "abc",
                }
            })))
            .mount(&server)
            .await;

        let client = ConnectApiClient::new(&server.uri(), "mizan-test-jwt").unwrap();
        let err = client.create_login_portal(None).await.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("API error"), "got: {msg}");
    }

    /// REPRO for the bug: the live Mizan Connect backend serves
    /// `GET /api/v1/sync/brokerage/connections` as a *bare* JSON array
    /// (`Json(Vec<BrokerConnectionDto>)` in `mizan-connect/src/snaptrade/handlers.rs`).
    /// The current desktop client expects a wrapper (`{ "connections": [...] }`)
    /// and fails to deserialize, so the IPC command returns Err and the React
    /// Query hook's data falls through to `[]` → "No broker connections yet"
    /// forever in the UI even though the row exists in Postgres.
    #[tokio::test]
    async fn list_connections_parses_bare_array_response() {
        let server = MockServer::start().await;

        // Exact shape returned by mizan-connect's snaptrade handler:
        // a top-level JSON array of BrokerConnectionDto.
        Mock::given(method("GET"))
            .and(path("/api/v1/sync/brokerage/connections"))
            .and(header("authorization", "Bearer mizan-test-jwt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": "auth-alpaca-1",
                    "name": "Alpaca Paper",
                    "type": "trade",
                    "disabled": false,
                    "status": "connected",
                    "brokerage": {
                        "id": "brk-alpaca",
                        "slug": "ALPACA-PAPER",
                        "name": "Alpaca Paper",
                        "display_name": "Alpaca Paper",
                        "aws_s3_logo_url": null,
                        "aws_s3_square_logo_url": null
                    },
                    "created_date": "2026-05-05T10:00:00Z",
                    "updated_date": "2026-05-05T10:00:01Z"
                }
            ])))
            .expect(1)
            .mount(&server)
            .await;

        let client = ConnectApiClient::new(&server.uri(), "mizan-test-jwt").unwrap();
        let connections = client
            .list_connections()
            .await
            .expect("list_connections should accept a bare-array response");

        assert_eq!(connections.len(), 1);
        let only = &connections[0];
        assert_eq!(only.id, "auth-alpaca-1");
        assert_eq!(only.status.as_deref(), Some("connected"));
        assert!(!only.disabled);
        let brk = only.brokerage.as_ref().expect("brokerage present");
        assert_eq!(brk.slug.as_deref(), Some("ALPACA-PAPER"));
        assert_eq!(brk.display_name.as_deref(), Some("Alpaca Paper"));
    }

    /// REPRO partner for `/accounts`: same bug pattern. Backend serves
    /// `Json(Vec<BrokerAccountDto>)` (bare array); current client expects
    /// `{ "accounts": [...] }`.
    #[tokio::test]
    async fn list_accounts_parses_bare_array_response() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/sync/brokerage/accounts"))
            .and(header("authorization", "Bearer mizan-test-jwt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": "acct-1",
                    "brokerage_authorization": "auth-alpaca-1",
                    "name": "Paper Trading",
                    "number": "P12345",
                    "institution_name": "Alpaca Paper",
                    "status": "open",
                    "is_paper": true
                }
            ])))
            .expect(1)
            .mount(&server)
            .await;

        let client = ConnectApiClient::new(&server.uri(), "mizan-test-jwt").unwrap();
        let accounts = client
            .list_accounts(None)
            .await
            .expect("list_accounts should accept a bare-array response");
        assert_eq!(accounts.len(), 1);
    }

    /// `CONNECT_BYPASS_PLAN_CHECK=true` short-circuits without an HTTP
    /// call. The mock server has no expectations registered, so a request
    /// to it would 404 — we use a deliberately-broken base URL to prove
    /// no network call happens.
    #[tokio::test]
    #[serial_test::serial]
    async fn has_broker_sync_bypass_flag_short_circuits() {
        std::env::set_var("CONNECT_BYPASS_PLAN_CHECK", "true");
        let client = ConnectApiClient::new("http://127.0.0.1:1/never-reachable", "mizan-test-jwt")
            .expect("client");
        let result = client.has_broker_sync().await.expect("bypass returns Ok");
        assert!(result, "bypass returns true");
        std::env::remove_var("CONNECT_BYPASS_PLAN_CHECK");
    }
}
