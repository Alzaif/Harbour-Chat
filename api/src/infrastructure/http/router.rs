use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State, WebSocketUpgrade},
    http::{header, HeaderValue, Request, StatusCode},
    middleware,
    response::Response,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;

use crate::domain::entities::PresenceStatus;
use crate::error::AppError;
use crate::infrastructure::http::gateway::{gateway_identity_middleware, AuthUser};
use crate::infrastructure::state::AppState;

pub fn create_app(state: AppState) -> Router {
    let public = Router::new()
        .route("/health", get(health))
        .route("/version", get(version));

    let api = Router::new()
        .route("/me", get(me))
        .route("/me/settings", get(get_user_settings).patch(update_user_settings))
        .route("/me/avatar", post(upload_avatar))
        .route("/users/search", get(search_users))
        .route("/users/{id}/avatar", get(get_user_avatar))
        .route("/servers", get(list_servers).post(create_server))
        .route(
            "/servers/{id}",
            get(get_server).patch(update_server).delete(delete_server),
        )
        .route(
            "/servers/{id}/channels",
            post(create_channel),
        )
        .route("/servers/{id}/members", get(list_members).post(add_member))
        .route("/servers/{id}/presence", get(list_presence).post(set_presence))
        .route(
            "/channels/{id}/messages",
            get(list_messages).post(send_message),
        )
        .route("/channels/{id}/typing", get(list_typing).post(set_typing))
        .route("/channels/{id}/voice", get(list_voice_participants))
        .route("/channels/{id}/voice/remote-producers", get(list_remote_voice_producers))
        .route("/channels/{id}/voice/join", post(join_voice))
        .route("/channels/{id}/voice/leave", post(leave_voice))
        .route("/channels/{id}/voice/state", post(update_voice_state))
        .route("/channels/{id}/voice/session", post(bootstrap_voice_session))
        .route("/channels/{id}/voice/session/{session_id}", delete(close_voice_session))
        .route("/channels/{id}/voice/transports", post(create_voice_transport))
        .route(
            "/channels/{id}/voice/transports/{transport_id}/connect",
            post(connect_voice_transport),
        )
        .route("/channels/{id}/voice/producers", post(create_voice_producer))
        .route("/channels/{id}/voice/consumers", post(create_voice_consumer))
        .route(
            "/channels/{id}/voice/transports/{transport_id}/ice-candidates",
            post(add_voice_ice_candidate),
        )
        .route(
            "/channels/{id}/voice/transports/{transport_id}/restart-ice",
            post(restart_voice_ice),
        )
        .route("/messages/{id}", patch(edit_message).delete(delete_message))
        .route("/messages/{id}/reactions", post(toggle_reaction))
        .route(
            "/messages/{id}/attachments",
            post(upload_message_attachment),
        )
        .route("/attachments/{id}", get(get_attachment))
        .route("/channels/{id}/read", post(mark_read))
        .route("/board/posts", get(list_board_posts).post(create_board_post))
        .route("/board/posts/{id}/share", post(share_board_post))
        .route("/board/posts/{id}/vote", post(vote_board_post))
        .route(
            "/board/posts/{id}/comments",
            get(list_board_comments).post(create_board_comment),
        )
        .route("/board/posts/{id}", get(get_board_post))
        .route("/board/share-targets", get(list_share_targets))
        .route("/dms", get(list_dms))
        .route("/dm-peers", get(list_dm_peers))
        .route("/dms/{user_id}", post(open_dm))
        .route("/ws", get(ws_handler))
        .route_layer(middleware::from_fn(security_headers_middleware))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            gateway_identity_middleware,
        ));

    Router::new()
        .merge(public)
        .nest("/api", api)
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn version(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": state.config.package_name,
        "version": state.config.package_version,
    }))
}

async fn me(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let avatar_updated_at = state.avatars.meta(&user.id).await?.map(|m| m.updated_at);
    Ok(Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "displayName": user.display_name,
        "avatarUpdatedAt": avatar_updated_at,
    })))
}

async fn list_servers(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let servers = state.chat.list_servers(&user).await?;
    Ok(Json(serde_json::to_value(servers).unwrap()))
}

#[derive(Deserialize)]
struct NameBody {
    name: Option<String>,
    description: Option<String>,
}

async fn create_server(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<NameBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let server = state
        .chat
        .create_server(
            &user,
            body.name.as_deref().unwrap_or(""),
            body.description.as_deref(),
        )
        .await?;
    Ok(Json(serde_json::to_value(server).unwrap()))
}

#[derive(Deserialize)]
struct UpdateServerBody {
    name: Option<String>,
    description: Option<String>,
    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,
    #[serde(rename = "cardColor")]
    card_color: Option<String>,
}

fn patch_optional_text(value: &Option<String>) -> Option<Option<&str>> {
    value.as_ref().map(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

async fn update_server(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<UpdateServerBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let server = state
        .chat
        .update_server(
            &user,
            &id,
            body.name.as_deref().map(str::trim),
            patch_optional_text(&body.description),
            patch_optional_text(&body.icon_url),
            patch_optional_text(&body.card_color),
        )
        .await?;
    Ok(Json(serde_json::to_value(server).unwrap()))
}

async fn delete_server(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.chat.delete_server(&user, &id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
struct UpdateUserSettingsBody {
    #[serde(rename = "pushToTalk")]
    push_to_talk: Option<bool>,
    #[serde(rename = "pushToTalkKey")]
    push_to_talk_key: Option<String>,
}

async fn get_user_settings(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::ports::UserRepository;
    let settings = state.users.get_settings(&user.id).await?;
    Ok(Json(serde_json::to_value(settings).unwrap()))
}

async fn update_user_settings(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<UpdateUserSettingsBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::ports::UserRepository;
    let mut settings = state.users.get_settings(&user.id).await?;
    if let Some(push_to_talk) = body.push_to_talk {
        settings.push_to_talk = push_to_talk;
    }
    if let Some(key) = body.push_to_talk_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            settings.push_to_talk_key = trimmed.to_string();
        }
    }
    let saved = state.users.upsert_settings(&user.id, &settings).await?;
    Ok(Json(serde_json::to_value(saved).unwrap()))
}

async fn get_server(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let detail = state.chat.get_server_detail(&user, &id).await?;
    Ok(Json(serde_json::to_value(detail).unwrap()))
}

#[derive(Deserialize)]
struct CreateChannelBody {
    name: Option<String>,
    #[serde(rename = "type")]
    channel_type: Option<String>,
}

async fn create_channel(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<CreateChannelBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::entities::ChannelType;
    let ty = match body.channel_type.as_deref().unwrap_or("text") {
        "text" => ChannelType::Text,
        "voice" => ChannelType::Voice,
        other => {
            return Err(AppError::Validation(format!(
                "unsupported channel type: {other}"
            )))
        }
    };
    let channel = state
        .chat
        .create_channel(&user, &id, body.name.as_deref().unwrap_or(""), ty)
        .await?;
    Ok(Json(serde_json::to_value(channel).unwrap()))
}

#[derive(Deserialize)]
struct AddMemberBody {
    #[serde(rename = "userId")]
    user_id: String,
}

async fn add_member(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<AddMemberBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::ports::UserRepository;
    if state.users.find_by_id(&body.user_id).await?.is_none() {
        return Err(AppError::NotFound("user not found".into()));
    }
    let member = state.chat.add_member(&user, &id, &body.user_id).await?;
    Ok(Json(serde_json::to_value(member).unwrap()))
}

#[derive(Deserialize)]
struct SearchUsersQuery {
    q: Option<String>,
    #[serde(rename = "excludeServerId")]
    exclude_server_id: Option<String>,
    limit: Option<u32>,
}

async fn search_users(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(q): Query<SearchUsersQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::ports::UserRepository;
    let query = q.q.unwrap_or_default();
    if query.trim().len() < 2 {
        return Err(AppError::Validation(
            "search query must be at least 2 characters".into(),
        ));
    }
    let results = state
        .users
        .search(
            &user.id,
            &query,
            q.exclude_server_id.as_deref(),
            q.limit.unwrap_or(20),
        )
        .await?;
    Ok(Json(serde_json::to_value(results).unwrap()))
}

async fn list_members(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let members = state.chat.list_members(&user, &id).await?;
    Ok(Json(serde_json::to_value(members).unwrap()))
}

#[derive(Deserialize)]
struct MessagesQuery {
    before: Option<String>,
    limit: Option<u32>,
}

async fn list_messages(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Query(q): Query<MessagesQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let messages = state
        .chat
        .list_messages(&user, &id, q.before.as_deref(), q.limit.unwrap_or(50))
        .await?;
    Ok(Json(serde_json::to_value(messages).unwrap()))
}

#[derive(Deserialize)]
struct ContentBody {
    content: Option<String>,
    #[serde(default, alias = "replyToMessageId")]
    reply_to_message_id: Option<String>,
}

async fn send_message(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<ContentBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let message = state
        .chat
        .send_message(
            &user,
            &id,
            body.content.as_deref().unwrap_or(""),
            body.reply_to_message_id.as_deref(),
        )
        .await?;
    Ok(Json(serde_json::to_value(message).unwrap()))
}

async fn edit_message(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<ContentBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let message = state
        .chat
        .edit_message(&user, &id, body.content.as_deref().unwrap_or(""))
        .await?;
    Ok(Json(serde_json::to_value(message).unwrap()))
}

async fn delete_message(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let message = state.chat.delete_message(&user, &id).await?;
    Ok(Json(serde_json::to_value(message).unwrap()))
}

#[derive(Deserialize)]
struct ReactionBody {
    emoji: Option<String>,
}

#[derive(Deserialize)]
struct PresenceBody {
    status: Option<String>,
}

#[derive(Deserialize)]
struct TypingBody {
    #[serde(rename = "isTyping")]
    is_typing: Option<bool>,
}

#[derive(Deserialize)]
struct VoiceStateBody {
    muted: Option<bool>,
    deafened: Option<bool>,
}

#[derive(Deserialize)]
struct VoiceSessionBody {
    #[serde(rename = "requestId")]
    request_id: Option<String>,
}

#[derive(Deserialize)]
struct CreateTransportBody {
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: String,
    direction: Option<String>,
}

#[derive(Deserialize)]
struct ConnectTransportBody {
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "dtlsParameters")]
    dtls_parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct CreateProducerBody {
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "transportId")]
    transport_id: String,
    kind: Option<String>,
    #[serde(rename = "rtpParameters")]
    rtp_parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct CreateConsumerBody {
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "transportId")]
    transport_id: String,
    #[serde(rename = "producerId")]
    producer_id: String,
    #[serde(rename = "rtpCapabilities")]
    rtp_capabilities: serde_json::Value,
}

#[derive(Deserialize)]
struct AddIceCandidateBody {
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: String,
    candidate: serde_json::Value,
}

#[derive(Deserialize)]
struct RestartIceBody {
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: String,
}

#[derive(Deserialize)]
struct ListRemoteProducersQuery {
    #[serde(rename = "sessionId")]
    session_id: String,
}

async fn toggle_reaction(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<ReactionBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let added = state
        .chat
        .toggle_reaction(&user, &id, body.emoji.as_deref().unwrap_or(""))
        .await?;
    Ok(Json(serde_json::json!({ "added": added })))
}

async fn upload_message_attachment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut filename = String::from("upload");
    let mut mime_type = String::from("application/octet-stream");
    let mut data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            filename = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| filename.clone());
            mime_type = field
                .content_type()
                .map(|s| s.to_string())
                .unwrap_or(mime_type.clone());
            data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(e.to_string()))?
                    .to_vec(),
            );
        }
    }

    let data = data.ok_or_else(|| AppError::Validation("file is required".into()))?;
    let message = state
        .chat
        .upload_attachment(&user, &id, &filename, &mime_type, &data)
        .await?;
    Ok(Json(serde_json::to_value(message).unwrap()))
}

async fn upload_avatar(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut mime_type = String::from("application/octet-stream");
    let mut data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(e.to_string()))?
    {
        if field.name().unwrap_or("") == "file" {
            mime_type = field
                .content_type()
                .map(|s| s.to_string())
                .unwrap_or(mime_type.clone());
            data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(e.to_string()))?
                    .to_vec(),
            );
        }
    }

    let data = data.ok_or_else(|| AppError::Validation("file is required".into()))?;
    let meta = state.avatars.save(&user.id, &mime_type, &data).await?;
    Ok(Json(serde_json::json!({
        "mimeType": meta.mime_type,
        "sizeBytes": meta.size_bytes,
        "avatarUpdatedAt": meta.updated_at,
    })))
}

async fn get_user_avatar(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let (mime_type, bytes) = state
        .avatars
        .read(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("avatar not found".into()))?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CACHE_CONTROL, "private, max-age=300")
        .header("x-content-type-options", "nosniff")
        .body(Body::from(bytes))
        .map_err(|e| AppError::Internal(e.to_string()))?)
}

async fn get_attachment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let (mime_type, bytes) = state.chat.get_attachment(&user, &id).await?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CONTENT_DISPOSITION, "attachment")
        .header("x-content-type-options", "nosniff")
        .body(Body::from(bytes))
        .map_err(|e| AppError::Internal(e.to_string()))?)
}

async fn security_headers_middleware(
    req: Request<Body>,
    next: middleware::Next,
) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "strict-transport-security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    response
}

#[derive(Deserialize)]
struct MarkReadBody {
    #[serde(rename = "messageId")]
    message_id: String,
}

async fn mark_read(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<MarkReadBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.chat.mark_read(&user, &id, &body.message_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn list_board_posts(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(query): Query<ListPostsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::application::board_service::FeedPeriod;
    let period = FeedPeriod::parse(query.period.as_deref().unwrap_or("day"))?;
    let feed = state
        .board
        .list_feed(&user, period, query.limit.unwrap_or(20))
        .await?;
    Ok(Json(serde_json::to_value(feed).unwrap()))
}

async fn create_board_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreatePostBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let post = state
        .board
        .create_post(&user, body.title.as_deref(), &body.body, body.link_url.as_deref())
        .await?;
    Ok(Json(serde_json::to_value(post).unwrap()))
}

async fn get_board_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let post = state.board.get_post(&user, &id).await?;
    Ok(Json(serde_json::to_value(post).unwrap()))
}

async fn vote_board_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<VotePostBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let post = state.board.vote_post(&user, &id, body.value).await?;
    Ok(Json(serde_json::to_value(post).unwrap()))
}

async fn list_board_comments(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let comments = state.board.list_comments(&user, &id).await?;
    Ok(Json(serde_json::to_value(comments).unwrap()))
}

async fn create_board_comment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<CreateCommentBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let comment = state
        .board
        .create_comment(
            &user,
            &id,
            &body.body,
            body.parent_comment_id.as_deref(),
        )
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(comment).unwrap()),
    ))
}

async fn list_share_targets(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let targets = state.board.list_share_targets(&user).await?;
    Ok(Json(serde_json::to_value(targets).unwrap()))
}

async fn share_board_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<SharePostBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.channel_id.trim().is_empty() {
        return Err(AppError::Validation("channelId is required".into()));
    }
    let message = state
        .board
        .share_post(&user, &id, body.channel_id.trim())
        .await?;
    Ok(Json(serde_json::to_value(message).unwrap()))
}

#[derive(Deserialize)]
struct SharePostBody {
    #[serde(rename = "channelId")]
    channel_id: String,
}

#[derive(Deserialize)]
struct ListPostsQuery {
    period: Option<String>,
    limit: Option<u32>,
}

#[derive(Deserialize)]
struct CreatePostBody {
    title: Option<String>,
    body: String,
    link_url: Option<String>,
}

#[derive(Deserialize)]
struct VotePostBody {
    value: i8,
}

#[derive(Deserialize)]
struct CreateCommentBody {
    body: String,
    #[serde(rename = "parentCommentId")]
    parent_comment_id: Option<String>,
}

async fn list_dms(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let inbox = state.chat.list_dms(&user).await?;
    Ok(Json(serde_json::to_value(inbox).unwrap()))
}

async fn list_dm_peers(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let peers = state.chat.list_dm_peers(&user).await?;
    Ok(Json(serde_json::to_value(peers).unwrap()))
}

async fn open_dm(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::ports::{GatewayIdentity, UserRepository};
    if state.users.find_by_id(&user_id).await?.is_none() {
        state
            .users
            .upsert_from_gateway(GatewayIdentity {
                id: user_id.clone(),
                email: format!("{user_id}@harbour.local"),
                display_name: None,
            })
            .await?;
    }
    let channel = state.chat.open_dm(&user, &user_id).await?;
    Ok(Json(serde_json::to_value(channel).unwrap()))
}

async fn list_presence(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let presence = state.chat.list_presence(&user, &id).await?;
    Ok(Json(serde_json::to_value(presence).unwrap()))
}

async fn set_presence(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<PresenceBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let status = match body.status.as_deref().unwrap_or("online") {
        "online" => PresenceStatus::Online,
        "idle" => PresenceStatus::Idle,
        "dnd" => PresenceStatus::Dnd,
        "offline" => PresenceStatus::Offline,
        _ => return Err(AppError::Validation("invalid presence status".into())),
    };
    let presence = state.chat.set_presence(&user, &id, status).await?;
    Ok(Json(serde_json::to_value(presence).unwrap()))
}

async fn list_typing(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let indicators = state.chat.list_typing(&user, &id).await?;
    Ok(Json(serde_json::to_value(indicators).unwrap()))
}

async fn set_typing(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<TypingBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let indicators = state
        .chat
        .set_typing(&user, &id, body.is_typing.unwrap_or(true))
        .await?;
    Ok(Json(serde_json::to_value(indicators).unwrap()))
}

async fn list_voice_participants(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let participants = state.chat.list_voice_participants(&user, &id).await?;
    Ok(Json(serde_json::to_value(participants).unwrap()))
}

async fn join_voice(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<VoiceStateBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let participant = state
        .chat
        .join_voice(
            &user,
            &id,
            body.muted.unwrap_or(false),
            body.deafened.unwrap_or(false),
        )
        .await?;
    Ok(Json(serde_json::to_value(participant).unwrap()))
}

async fn leave_voice(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.chat.leave_voice(&user, &id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn update_voice_state(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<VoiceStateBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let participant = state
        .chat
        .update_voice_state(
            &user,
            &id,
            body.muted.unwrap_or(false),
            body.deafened.unwrap_or(false),
        )
        .await?;
    Ok(Json(serde_json::to_value(participant).unwrap()))
}

async fn bootstrap_voice_session(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<VoiceSessionBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let request_id = body
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let session = state.chat.bootstrap_voice_session(&user, &id).await?;
    Ok(Json(signal_ok(
        request_id,
        "session_bootstrap",
        serde_json::to_value(session).unwrap(),
    )))
}

async fn close_voice_session(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, session_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    state
        .chat
        .close_voice_session(&user, &id, &session_id)
        .await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn create_voice_transport(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<CreateTransportBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let request_id = body
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let transport = state
        .chat
        .create_voice_transport(
            &user,
            &id,
            &body.session_id,
            body.direction.as_deref().unwrap_or("send"),
        )
        .await?;
    Ok(Json(signal_ok(
        request_id,
        "create_transport",
        serde_json::to_value(transport).unwrap(),
    )))
}

async fn connect_voice_transport(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, transport_id)): Path<(String, String)>,
    Json(body): Json<ConnectTransportBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let request_id = body
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    state
        .chat
        .connect_voice_transport(
            &user,
            &id,
            &body.session_id,
            &transport_id,
            body.dtls_parameters,
        )
        .await?;
    Ok(Json(signal_ok(
        request_id,
        "connect_transport",
        serde_json::json!({ "ok": true }),
    )))
}

async fn create_voice_producer(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<CreateProducerBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let request_id = body
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let producer = state
        .chat
        .create_voice_producer(
            &user,
            &id,
            &body.session_id,
            &body.transport_id,
            body.kind.as_deref().unwrap_or("audio"),
            body.rtp_parameters,
        )
        .await?;
    Ok(Json(signal_ok(
        request_id,
        "create_producer",
        serde_json::to_value(producer).unwrap(),
    )))
}

async fn create_voice_consumer(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Json(body): Json<CreateConsumerBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let request_id = body
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let consumer = state
        .chat
        .create_voice_consumer(
            &user,
            &id,
            &body.session_id,
            &body.transport_id,
            &body.producer_id,
            body.rtp_capabilities,
        )
        .await?;
    Ok(Json(signal_ok(
        request_id,
        "create_consumer",
        serde_json::to_value(consumer).unwrap(),
    )))
}

async fn add_voice_ice_candidate(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, transport_id)): Path<(String, String)>,
    Json(body): Json<AddIceCandidateBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let request_id = body
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    state
        .chat
        .add_voice_ice_candidate(
            &user,
            &id,
            &body.session_id,
            &transport_id,
            body.candidate,
        )
        .await?;
    Ok(Json(signal_ok(
        request_id,
        "add_ice_candidate",
        serde_json::json!({ "ok": true }),
    )))
}

async fn restart_voice_ice(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, transport_id)): Path<(String, String)>,
    Json(body): Json<RestartIceBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let request_id = body
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let ice = state
        .chat
        .restart_voice_ice(&user, &id, &body.session_id, &transport_id)
        .await?;
    Ok(Json(signal_ok(request_id, "restart_ice", ice)))
}

async fn list_remote_voice_producers(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<String>,
    Query(q): Query<ListRemoteProducersQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let producers = state
        .chat
        .list_remote_voice_producers(&user, &id, &q.session_id)
        .await?;
    Ok(Json(serde_json::json!({ "producers": producers })))
}

fn signal_ok(request_id: String, kind: &str, payload: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "type": "signal_response",
        "requestId": request_id,
        "kind": kind,
        "ok": true,
        "payload": payload
    })
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let realtime = state.realtime.clone();
    let chat = state.chat.clone();
    Ok(ws.on_upgrade(move |socket| async move {
        realtime.handle_socket(socket, user, chat).await;
    }))
}
