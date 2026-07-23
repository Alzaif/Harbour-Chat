use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::contracts::gateway_headers::{self, APP_BOARD_SCOPE};
use crate::domain::ports::{GatewayIdentity, UserRepository};
use crate::error::AppError;
use crate::infrastructure::state::AppState;

pub const SESSION_COOKIE_NAME: &str = "harbour_session";

#[derive(Clone)]
pub struct AuthUser(pub crate::domain::entities::User);

pub async fn gateway_identity_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    match gateway_identity_inner(&state, &mut req).await {
        Ok(user) => {
            req.extensions_mut().insert(AuthUser(user));
            next.run(req).await
        }
        Err(err) => err.into_response(),
    }
}

async fn gateway_identity_inner(
    state: &AppState,
    req: &mut Request<Body>,
) -> Result<crate::domain::entities::User, AppError> {
    let config = &state.config;

    let (user_id, email, display_name) = if config.trust_gateway_headers {
        if config.require_https_forwarded_proto {
            let proto = req
                .headers()
                .get("x-forwarded-proto")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default();
            if proto != "https" {
                return Err(AppError::Unauthorized);
            }
        }

        if let Some(expected) = &config.trusted_proxy_token {
            let received = req
                .headers()
                .get("x-harbour-proxy-token")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default();
            if received != expected {
                return Err(AppError::Unauthorized);
            }
        }

        let has_gateway_headers = req.headers().contains_key(gateway_headers::USER_ID);
        if has_gateway_headers {
            let user_id = req
                .headers()
                .get(gateway_headers::USER_ID)
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or(AppError::Unauthorized)?
                .to_string();

            let email = req
                .headers()
                .get(gateway_headers::EMAIL)
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or(AppError::Unauthorized)?
                .to_string();

            let scopes = req
                .headers()
                .get(gateway_headers::SCOPES)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<_>>();

            if !scopes.contains(&APP_BOARD_SCOPE) {
                return Err(AppError::Unauthorized);
            }

            let display_name = req
                .headers()
                .get(gateway_headers::DISPLAY_NAME)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);

            (user_id, email, display_name)
        } else if req.uri().path() == "/api/ws" {
            let cookie = session_cookie_value(
                req.headers()
                    .get("cookie")
                    .and_then(|value| value.to_str().ok()),
            )
            .ok_or(AppError::Unauthorized)?;
            resolve_identity_from_portcullis(state, cookie).await?
        } else {
            return Err(AppError::Unauthorized);
        }
    } else {
        let user_id = config.dev_user_id.clone().ok_or(AppError::Unauthorized)?;
        let email = config
            .dev_user_email
            .clone()
            .unwrap_or_else(|| format!("{user_id}@dev.local"));
        let display_name = config.dev_user_display_name.clone();
        (user_id, email, display_name)
    };

    state
        .users
        .upsert_from_gateway(GatewayIdentity {
            id: user_id,
            email,
            display_name,
        })
        .await
}

async fn resolve_identity_from_portcullis(
    state: &AppState,
    session_cookie: &str,
) -> Result<(String, String, Option<String>), AppError> {
    let response = reqwest::Client::new()
        .get(&state.config.portcullis_forward_url)
        .header(
            "cookie",
            format!("{SESSION_COOKIE_NAME}={session_cookie}"),
        )
        .header("x-forwarded-proto", "https")
        .header("host", &state.config.forward_auth_host)
        .header("x-forwarded-host", &state.config.forward_auth_host)
        .header("x-forwarded-uri", &state.config.forward_auth_uri)
        .send()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::Unauthorized);
    }

    let user_id = response
        .headers()
        .get(gateway_headers::USER_ID)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(AppError::Unauthorized)?
        .to_string();

    let email = response
        .headers()
        .get(gateway_headers::EMAIL)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(AppError::Unauthorized)?
        .to_string();

    let scopes = response
        .headers()
        .get(gateway_headers::SCOPES)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>();

    if !scopes.contains(&APP_BOARD_SCOPE) {
        return Err(AppError::Unauthorized);
    }

    let display_name = response
        .headers()
        .get(gateway_headers::DISPLAY_NAME)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    Ok((user_id, email, display_name))
}

pub fn session_cookie_value(cookie_header: Option<&str>) -> Option<&str> {
    let cookie_header = cookie_header?;
    let prefix = format!("{SESSION_COOKIE_NAME}=");
    cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix(prefix.as_str()))
}

#[allow(dead_code)]
pub fn unauthorized_response() -> Response {
    (StatusCode::UNAUTHORIZED, "unauthorized").into_response()
}
