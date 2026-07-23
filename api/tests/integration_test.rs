use std::path::PathBuf;

use axum::body::Body;
use http_body_util::BodyExt;
use tower::ServiceExt;

use harbour_chat_api::{create_app, AppState, Config};
use harbour_chat_api::domain::ports::{GatewayIdentity, UserRepository};
use harbour_chat_api::error::AppError;

const GENERAL_CHANNEL_ID: &str = "00000000-0000-4000-8000-000000000002";
const BOARD_FEED_TOPIC: &str = "__board__";

async fn test_app() -> axum::Router {
    let state = test_state().await;
    create_app(state)
}

async fn test_state() -> AppState {
    let db_path = PathBuf::from(format!(
        "{}/harbour-chat-test-{}.db",
        std::env::temp_dir().display(),
        uuid::Uuid::new_v4()
    ));
    let config = Config::for_test(db_path);
    AppState::new_for_test(config)
        .await
        .expect("test app state")
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
async fn reply_message_includes_quoted_preview() {
    let app = test_app().await;

    let parent = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/channels/{GENERAL_CHANNEL_ID}/messages"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "content": "parent message" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(parent.status(), 200);
    let parent_msg = body_json(parent.into_body()).await;
    let parent_id = parent_msg["id"].as_str().unwrap();

    let reply = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/channels/{GENERAL_CHANNEL_ID}/messages"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "content": "reply body",
                        "reply_to_message_id": parent_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reply.status(), 200);
    let reply_msg = body_json(reply.into_body()).await;
    assert_eq!(reply_msg["content"], "reply body");
    assert_eq!(reply_msg["reply_to_message_id"], parent_id);
    assert_eq!(reply_msg["reply_to"]["id"], parent_id);
    assert_eq!(reply_msg["reply_to"]["content"], "parent message");
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
                .header("x-harbour-scopes", "app:board")
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

#[tokio::test]
async fn list_dms_empty_then_populated_after_dm_message() {
    let app = test_app().await;

    let empty = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/dms")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(empty.status(), 200);
    let empty_json = body_json(empty.into_body()).await;
    assert!(empty_json.as_array().unwrap().is_empty());

    let dm_open = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/dms/friend-user")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dm_open.status(), 200);
    let channel = body_json(dm_open.into_body()).await;
    let channel_id = channel["id"].as_str().unwrap();

    let post = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/channels/{channel_id}/messages"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "content": "direct hello" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(post.status(), 200);

    let inbox = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/dms")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(inbox.status(), 200);
    let entries = body_json(inbox.into_body()).await;
    let list = entries.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["channelId"], channel_id);
    assert_eq!(list[0]["otherUserId"], "friend-user");
    assert_eq!(list[0]["lastMessagePreview"], "direct hello");
}

#[tokio::test]
async fn board_posts_crud() {
    let app = test_app().await;

    let create = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/board/posts")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "title": "Harbour news",
                        "body": "Check this out",
                        "link_url": "https://example.com/article"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), 200);
    let post = body_json(create.into_body()).await;
    assert_eq!(post["body"], "Check this out");
    assert_eq!(post["score"], 0);
    assert_eq!(post["myVote"], 0);
    let post_id = post["id"].as_str().unwrap();

    let list = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/board/posts?period=day")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), 200);
    let feed = body_json(list.into_body()).await;
    assert_eq!(feed["period"], "day");
    let sections = feed["sections"].as_array().unwrap();
    assert!(!sections.is_empty());
    assert_eq!(sections[0]["kind"], "top");
    assert_eq!(sections[0]["posts"].as_array().unwrap().len(), 1);

    let get = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/board/posts/{post_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get.status(), 200);
}

#[tokio::test]
async fn board_vote_toggle_flip_clear() {
    let app = test_app().await;

    let create = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/board/posts")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "body": "Vote me" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let post_id = body_json(create.into_body()).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let upvote = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/board/posts/{post_id}/vote"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "value": 1 }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upvote.status(), 200);
    let up = body_json(upvote.into_body()).await;
    assert_eq!(up["score"], 1);
    assert_eq!(up["upvotes"], 1);
    assert_eq!(up["myVote"], 1);

    // Same vote again clears
    let clear = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/board/posts/{post_id}/vote"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "value": 1 }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let cleared = body_json(clear.into_body()).await;
    assert_eq!(cleared["score"], 0);
    assert_eq!(cleared["myVote"], 0);

    // Flip downvote then upvote
    let down = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/board/posts/{post_id}/vote"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "value": -1 }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let downed = body_json(down.into_body()).await;
    assert_eq!(downed["score"], -1);
    assert_eq!(downed["myVote"], -1);

    let flip = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/board/posts/{post_id}/vote"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "value": 1 }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let flipped = body_json(flip.into_body()).await;
    assert_eq!(flipped["score"], 1);
    assert_eq!(flipped["myVote"], 1);

    let explicit_clear = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/board/posts/{post_id}/vote"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "value": 0 }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let zeroed = body_json(explicit_clear.into_body()).await;
    assert_eq!(zeroed["score"], 0);
    assert_eq!(zeroed["myVote"], 0);
}

#[tokio::test]
async fn board_comments_nest_under_parent() {
    let app = test_app().await;

    let create = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/board/posts")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "body": "Thread root" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let post_id = body_json(create.into_body()).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let root = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/board/posts/{post_id}/comments"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "body": "top level" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(root.status(), 201);
    let root_c = body_json(root.into_body()).await;
    let root_id = root_c["id"].as_str().unwrap();

    let reply = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/board/posts/{post_id}/comments"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "body": "nested reply",
                        "parentCommentId": root_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reply.status(), 201);

    let list = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/board/posts/{post_id}/comments"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), 200);
    let tree = body_json(list.into_body()).await;
    let arr = tree.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["body"], "top level");
    assert_eq!(arr[0]["replies"].as_array().unwrap().len(), 1);
    assert_eq!(arr[0]["replies"][0]["body"], "nested reply");
}

#[tokio::test]
async fn board_period_top_excludes_old_from_first_section() {
    let state = test_state().await;
    let db_path = state.config.db_path.clone();
    let app = create_app(state);

    let create_fresh = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/board/posts")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "body": "fresh post" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let fresh_id = body_json(create_fresh.into_body()).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let create_old = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/board/posts")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "body": "stale post" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let old_id = body_json(create_old.into_body()).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Age the second post beyond the 1-hour window and give it a high score
    let old_ms = chrono::Utc::now().timestamp_millis() - 3_600_000 * 3;
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&format!("sqlite:{}?mode=rw", db_path.display()))
        .await
        .expect("open test db");
    sqlx::query(
        "UPDATE posts SET created_at = ?, updated_at = ?, upvotes = 5, downvotes = 0, score = 5 WHERE id = ?",
    )
    .bind(old_ms)
    .bind(old_ms)
    .bind(&old_id)
    .execute(&pool)
    .await
    .expect("age old post");

    let list = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/board/posts?period=hour&limit=20")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), 200);
    let feed = body_json(list.into_body()).await;
    let sections = feed["sections"].as_array().unwrap();
    assert_eq!(sections[0]["kind"], "top");
    let top_ids: Vec<&str> = sections[0]["posts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert!(top_ids.contains(&fresh_id.as_str()));
    assert!(!top_ids.contains(&old_id.as_str()));

    let older = sections
        .iter()
        .find(|s| s["kind"] == "older")
        .expect("older section");
    assert!(older["label"]
        .as_str()
        .unwrap()
        .contains("older than 1 hour"));
    let older_ids: Vec<&str> = older["posts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert!(older_ids.contains(&old_id.as_str()));
}

#[tokio::test]
async fn board_soft_deleted_comment_shows_placeholder() {
    let state = test_state().await;
    let db_path = state.config.db_path.clone();
    let app = create_app(state);

    let create = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/board/posts")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "body": "post" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let post_id = body_json(create.into_body()).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let root = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/board/posts/{post_id}/comments"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "body": "will delete" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let root_id = body_json(root.into_body()).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let _ = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/board/posts/{post_id}/comments"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "body": "child stays",
                        "parentCommentId": root_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&format!("sqlite:{}?mode=rw", db_path.display()))
        .await
        .expect("open test db");
    sqlx::query("UPDATE post_comments SET deleted_at = ? WHERE id = ?")
        .bind(chrono::Utc::now().timestamp_millis())
        .bind(&root_id)
        .execute(&pool)
        .await
        .expect("soft-delete comment");

    let list = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/board/posts/{post_id}/comments"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let tree = body_json(list.into_body()).await;
    assert_eq!(tree[0]["body"], "[deleted]");
    assert_eq!(tree[0]["replies"][0]["body"], "child stays");
}

#[tokio::test]
async fn share_board_post_to_channel() {
    let app = test_app().await;

    let create = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/board/posts")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "body": "Share me" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let post = body_json(create.into_body()).await;
    let post_id = post["id"].as_str().unwrap();

    let share = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/board/posts/{post_id}/share"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "channelId": GENERAL_CHANNEL_ID }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(share.status(), 200);
    let message = body_json(share.into_body()).await;
    assert!(message["content"]
        .as_str()
        .unwrap()
        .contains("/board/feed/"));

    let messages = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/channels/{GENERAL_CHANNEL_ID}/messages"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(messages.status(), 200);
    let list = body_json(messages.into_body()).await;
    assert!(list
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["content"].as_str().unwrap_or("").contains("Shared from Board")));
}

#[tokio::test]
async fn search_users_and_add_member_to_group() {
    let app = test_app().await;

    let dm_open = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/dms/friend-user")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dm_open.status(), 200);

    let search = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/users/search?q=friend")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(search.status(), 200);
    let users = body_json(search.into_body()).await;
    assert!(users
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["id"] == "friend-user"));

    let create_server = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/servers")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "name": "Weekend crew" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_server.status(), 200);
    let server = body_json(create_server.into_body()).await;
    let server_id = server["id"].as_str().unwrap();

    let add = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/servers/{server_id}/members"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "userId": "friend-user" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(add.status(), 200);

    let members = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/servers/{server_id}/members"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(members.status(), 200);
    let member_list = body_json(members.into_body()).await;
    assert!(member_list
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["user_id"] == "friend-user"));

    let exclude_search = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!(
                    "/api/users/search?q=friend&excludeServerId={server_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(exclude_search.status(), 200);
    let excluded = body_json(exclude_search.into_body()).await;
    assert!(excluded.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn search_users_requires_min_query_length() {
    let app = test_app().await;
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/users/search?q=a")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn update_server_appearance_and_delete() {
    let app = test_app().await;

    let server_res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/servers")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "name": "Styled Guild", "description": "A test guild" })
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(server_res.status(), 200);
    let server = body_json(server_res.into_body()).await;
    let server_id = server["id"].as_str().unwrap();

    let patch_res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/servers/{server_id}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "cardColor": "#ff4455",
                        "iconUrl": "https://example.com/icon.png"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch_res.status(), 200);
    let updated = body_json(patch_res.into_body()).await;
    assert_eq!(updated["cardColor"], "#ff4455");
    assert_eq!(updated["icon_url"], "https://example.com/icon.png");
    assert_eq!(updated["description"], "A test guild");

    let delete_res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/servers/{server_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_res.status(), 200);

    let get_res = app
        .oneshot(
            axum::http::Request::builder()
                .uri(format!("/api/servers/{server_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_res.status(), 404);
}

#[tokio::test]
async fn delete_harbour_home_is_forbidden() {
    let app = test_app().await;
    let harbour_home = "00000000-0000-4000-8000-000000000001";
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/servers/{harbour_home}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn user_settings_push_to_talk() {
    let app = test_app().await;

    let get_res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/me/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_res.status(), 200);
    let defaults = body_json(get_res.into_body()).await;
    assert_eq!(defaults["pushToTalk"], false);
    assert_eq!(defaults["pushToTalkKey"], "Space");

    let patch_res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri("/api/me/settings")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "pushToTalk": true, "pushToTalkKey": "KeyV" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch_res.status(), 200);
    let saved = body_json(patch_res.into_body()).await;
    assert_eq!(saved["pushToTalk"], true);
    assert_eq!(saved["pushToTalkKey"], "KeyV");

    let get_again = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/me/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_again.status(), 200);
    let settings = body_json(get_again.into_body()).await;
    assert_eq!(settings["pushToTalk"], true);
}

#[tokio::test]
async fn gateway_mode_rejects_startup_without_proxy_token() {
    let db_path = PathBuf::from(format!(
        "{}/harbour-chat-test-invalid-{}.db",
        std::env::temp_dir().display(),
        uuid::Uuid::new_v4()
    ));
    let mut config = Config::for_test(db_path);
    config.trust_gateway_headers = true;
    config.trusted_proxy_token = None;

    let result = AppState::new_for_test(config).await;
    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn authorize_realtime_subscribe_enforces_membership() {
    let state = test_state().await;
    let dev = state
        .users
        .upsert_from_gateway(GatewayIdentity {
            id: "dev-user".into(),
            email: "dev@harbour.local".into(),
            display_name: Some("Dev User".into()),
        })
        .await
        .unwrap();
    let outsider = state
        .users
        .upsert_from_gateway(GatewayIdentity {
            id: "outsider".into(),
            email: "outsider@example.com".into(),
            display_name: Some("Outsider".into()),
        })
        .await
        .unwrap();

    state
        .chat
        .authorize_realtime_subscribe(&dev, BOARD_FEED_TOPIC)
        .await
        .expect("board feed allowed for authenticated user");
    state
        .chat
        .authorize_realtime_subscribe(&dev, GENERAL_CHANNEL_ID)
        .await
        .expect("harbour home general allowed for member");

    let server = state
        .chat
        .create_server(&dev, "Private Guild", None)
        .await
        .expect("create server");
    let detail = state
        .chat
        .get_server_detail(&dev, &server.id)
        .await
        .expect("server detail");
    let channel_id = detail
        .channels
        .first()
        .expect("default channel")
        .id
        .clone();

    state
        .chat
        .authorize_realtime_subscribe(&dev, &channel_id)
        .await
        .expect("owner can subscribe");
    assert!(
        state
            .chat
            .authorize_realtime_subscribe(&outsider, &channel_id)
            .await
            .is_err(),
        "non-member must not subscribe to private server channel"
    );
}

/// Minimal valid 1x1 PNG so `infer` detects `image/png`.
fn sample_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x62, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

fn multipart_body(boundary: &str, filename: &str, content_type: &str, bytes: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

#[tokio::test]
async fn avatar_upload_and_serve_roundtrip() {
    let app = test_app().await;

    let missing = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/users/dev-user/avatar")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), 404);

    let png = sample_png();
    let boundary = "----harbouravatar";
    let upload = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/me/avatar")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(multipart_body(boundary, "me.png", "image/png", &png)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload.status(), 200);
    let meta = body_json(upload.into_body()).await;
    assert_eq!(meta["mimeType"], "image/png");
    assert!(meta["avatarUpdatedAt"].as_i64().is_some());

    let served = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/users/dev-user/avatar")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(served.status(), 200);
    assert_eq!(
        served
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("image/png")
    );
    let served_bytes = served.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(served_bytes.as_ref(), png.as_slice());

    let me = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let me_json = body_json(me.into_body()).await;
    assert!(me_json["avatarUpdatedAt"].as_i64().is_some());
}

#[tokio::test]
async fn avatar_upload_rejects_non_image() {
    let app = test_app().await;
    let boundary = "----harbouravatarbad";
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/me/avatar")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(multipart_body(
                    boundary,
                    "note.pdf",
                    "application/pdf",
                    b"not really a pdf",
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}
