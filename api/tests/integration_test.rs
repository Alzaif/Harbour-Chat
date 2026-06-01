use std::path::PathBuf;

use axum::body::Body;
use http_body_util::BodyExt;
use tower::ServiceExt;

use harbour_chat_api::{create_app, AppState, Config};

const GENERAL_CHANNEL_ID: &str = "00000000-0000-4000-8000-000000000002";

async fn test_app() -> axum::Router {
    let db_path = PathBuf::from(format!(
        "{}/harbour-chat-test-{}.db",
        std::env::temp_dir().display(),
        uuid::Uuid::new_v4()
    ));
    let config = Config::for_test(db_path);
    let state = AppState::new_for_test(config)
        .await
        .expect("test app state");
    create_app(state)
}

async fn trusted_headers_app() -> axum::Router {
    let db_path = PathBuf::from(format!(
        "{}/harbour-chat-test-trusted-{}.db",
        std::env::temp_dir().display(),
        uuid::Uuid::new_v4()
    ));
    let mut config = Config::for_test(db_path);
    config.trust_gateway_headers = true;
    config.require_https_forwarded_proto = true;
    config.trusted_proxy_token = Some("proxy-token".into());
    let state = AppState::new_for_test(config)
        .await
        .expect("trusted app state");
    create_app(state)
}

async fn body_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn health_without_auth() {
    let app = test_app().await;
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let json = body_json(res.into_body()).await;
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn send_message_in_general_after_joining() {
    let app = test_app().await;

    let post = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/channels/{GENERAL_CHANNEL_ID}/messages"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "content": "hello harbour" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(post.status(), 200);
    let msg = body_json(post.into_body()).await;
    assert_eq!(msg["content"], "hello harbour");
    assert_eq!(msg["author_display_name"], "Dev User");

    let list = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/channels/{GENERAL_CHANNEL_ID}/messages"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), 200);
    let messages = body_json(list.into_body()).await;
    assert!(messages.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn me_returns_current_user() {
    let app = test_app().await;
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let json = body_json(res.into_body()).await;
    assert_eq!(json["id"], "dev-user");
    assert_eq!(json["displayName"], "Dev User");
}

#[tokio::test]
async fn members_include_display_name() {
    let app = test_app().await;
    let harbour_home = "00000000-0000-4000-8000-000000000001";
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/servers/{harbour_home}/members"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let members = body_json(res.into_body()).await;
    let arr = members.as_array().unwrap();
    assert!(!arr.is_empty());
    assert!(arr[0]["display_name"].is_string());
}

#[tokio::test]
async fn toggle_reaction_and_unread_after_message() {
    let app = test_app().await;

    let post = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/channels/{GENERAL_CHANNEL_ID}/messages"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "content": "react me" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let msg = body_json(post.into_body()).await;
    let message_id = msg["id"].as_str().unwrap();

    let reaction = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/messages/{message_id}/reactions"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "emoji": "👍" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reaction.status(), 200);

    let list = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/channels/{GENERAL_CHANNEL_ID}/messages"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let messages = body_json(list.into_body()).await;
    let with_reaction = messages
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == message_id)
        .unwrap();
    assert!(with_reaction["reactions"].as_array().unwrap().len() >= 1);

    let harbour_home = "00000000-0000-4000-8000-000000000001";
    let server = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/servers/{harbour_home}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let detail = body_json(server.into_body()).await;
    assert!(detail["unreadByChannelId"].is_object());
}

#[tokio::test]
async fn create_server_and_channel() {
    let app = test_app().await;

    let server_res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/servers")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "name": "Test Guild" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(server_res.status(), 200);
    let server = body_json(server_res.into_body()).await;
    let server_id = server["id"].as_str().unwrap();

    let channel_res = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/servers/{server_id}/channels"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "name": "random", "type": "text" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(channel_res.status(), 200);
    let channel = body_json(channel_res.into_body()).await;
    assert_eq!(channel["name"], "random");
}

#[tokio::test]
async fn harbour_home_member_can_create_voice_channel() {
    let app = test_app().await;
    let harbour_home = "00000000-0000-4000-8000-000000000001";
    let channel_res = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/servers/{harbour_home}/channels"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "name": "live-room", "type": "voice" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(channel_res.status(), 200);
    let channel = body_json(channel_res.into_body()).await;
    assert_eq!(channel["type"], "voice");
}

#[tokio::test]
async fn trusted_header_mode_rejects_non_https_proto() {
    let app = trusted_headers_app().await;
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/me")
                .header("x-forwarded-proto", "http")
                .header("x-harbour-proxy-token", "proxy-token")
                .header("x-harbour-user-id", "u1")
                .header("x-harbour-email", "u1@example.com")
                .header("x-harbour-scopes", "app:chat")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn api_sets_transport_security_headers() {
    let app = test_app().await;
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let headers = res.headers();
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert!(headers.get("strict-transport-security").is_some());
}

#[tokio::test]
async fn attachment_upload_rejects_mime_mismatch() {
    let app = test_app().await;
    let post = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/channels/{GENERAL_CHANNEL_ID}/messages"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "content": "with file" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let msg = body_json(post.into_body()).await;
    let message_id = msg["id"].as_str().unwrap();

    let boundary = "----harbourtest";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"bad.pdf\"\r\nContent-Type: application/pdf\r\n\r\nnot a real pdf\r\n--{boundary}--\r\n"
    );
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/messages/{message_id}/attachments"))
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn presence_typing_and_voice_endpoints_work() {
    let app = test_app().await;
    let harbour_home = "00000000-0000-4000-8000-000000000001";

    let presence = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/servers/{harbour_home}/presence"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "status": "idle" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(presence.status(), 200);

    let typing_start = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/channels/{GENERAL_CHANNEL_ID}/typing"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "isTyping": true }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(typing_start.status(), 200);
    let typing_list = body_json(typing_start.into_body()).await;
    assert!(typing_list.as_array().unwrap().iter().any(|v| v["user_id"] == "dev-user"));

    let server_res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/servers")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "name": "Voice Guild" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(server_res.status(), 200);
    let server = body_json(server_res.into_body()).await;
    let server_id = server["id"].as_str().unwrap();

    let create_voice = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/servers/{server_id}/channels"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "name": "Voice 1", "type": "voice" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_voice.status(), 200);
    let voice = body_json(create_voice.into_body()).await;
    let voice_id = voice["id"].as_str().unwrap();

    let join = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/channels/{voice_id}/voice/join"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "muted": true, "deafened": false }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(join.status(), 200);

    let list = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/channels/{voice_id}/voice"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), 200);
    let participants = body_json(list.into_body()).await;
    assert_eq!(participants.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn voice_signaling_endpoints_work_and_forbid_unauthorized_channel() {
    let app = test_app().await;
    let harbour_home = "00000000-0000-4000-8000-000000000001";

    let create_voice = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/servers/{harbour_home}/channels"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "name": "Voice Signaling", "type": "voice" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_voice.status(), 200);
    let voice = body_json(create_voice.into_body()).await;
    let voice_id = voice["id"].as_str().unwrap();

    let bootstrap = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/channels/{voice_id}/voice/session"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "requestId": "req-1" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bootstrap.status(), 200);
    let boot = body_json(bootstrap.into_body()).await;
    assert_eq!(boot["type"], "signal_response");
    let session_id = boot["payload"]["session_id"].as_str().unwrap();

    let create_transport = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/channels/{voice_id}/voice/transports"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "requestId": "req-2",
                        "sessionId": session_id,
                        "direction": "send"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_transport.status(), 200);

    let forbidden = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/channels/not-a-channel/voice/session")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "requestId": "req-3" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden.status(), 404);
}
