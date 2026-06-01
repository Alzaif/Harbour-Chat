use harbour_chat_api::contracts::events::{
    PresenceChangedV1, TypingStartedV1, VoiceConsumerCreatedV1, VoiceSessionCreatedV1,
    VoiceTransportCreatedV1, VoiceParticipantUpdatedV1, PRESENCE_CHANGED_V1, TYPING_STARTED_V1,
    VOICE_CONSUMER_CREATED_V1, VOICE_PARTICIPANT_UPDATED_V1, VOICE_SESSION_CREATED_V1,
    VOICE_TRANSPORT_CREATED_V1,
};
use harbour_chat_api::contracts::voice_signaling::{VoiceSignalEnvelope, VoiceSignalKind, VoiceSignalResponse};

#[test]
fn typing_started_contract_serializes_stable_shape() {
    let event = TypingStartedV1 {
        schema: TYPING_STARTED_V1.to_string(),
        channel_id: "c1".into(),
        user_id: "u1".into(),
        occurred_at: "2026-01-01T00:00:00Z".into(),
        expires_at: "2026-01-01T00:00:08Z".into(),
    };
    let value = serde_json::to_value(event).expect("serialize typing");
    assert_eq!(value["schema"], TYPING_STARTED_V1);
    assert!(value["channel_id"].is_string());
    assert!(value["user_id"].is_string());
    assert!(value["expires_at"].is_string());
}

#[test]
fn presence_changed_contract_serializes_stable_shape() {
    let event = PresenceChangedV1 {
        schema: PRESENCE_CHANGED_V1.to_string(),
        server_id: "s1".into(),
        user_id: "u1".into(),
        status: "online".into(),
        occurred_at: "2026-01-01T00:00:00Z".into(),
    };
    let value = serde_json::to_value(event).expect("serialize presence");
    assert_eq!(value["schema"], PRESENCE_CHANGED_V1);
    assert_eq!(value["status"], "online");
}

#[test]
fn voice_updated_contract_serializes_stable_shape() {
    let event = VoiceParticipantUpdatedV1 {
        schema: VOICE_PARTICIPANT_UPDATED_V1.to_string(),
        channel_id: "c1".into(),
        user_id: "u1".into(),
        connected: true,
        muted: false,
        deafened: true,
        occurred_at: "2026-01-01T00:00:00Z".into(),
    };
    let value = serde_json::to_value(event).expect("serialize voice");
    assert_eq!(value["schema"], VOICE_PARTICIPANT_UPDATED_V1);
    assert_eq!(value["connected"], true);
    assert_eq!(value["deafened"], true);
}

#[test]
fn voice_session_contract_serializes_stable_shape() {
    let event = VoiceSessionCreatedV1 {
        schema: VOICE_SESSION_CREATED_V1.to_string(),
        session_id: "sess-1".into(),
        channel_id: "c1".into(),
        user_id: "u1".into(),
        occurred_at: "2026-01-01T00:00:00Z".into(),
    };
    let value = serde_json::to_value(event).expect("serialize session");
    assert_eq!(value["schema"], VOICE_SESSION_CREATED_V1);
    assert!(value["session_id"].is_string());
}

#[test]
fn voice_transport_contract_serializes_stable_shape() {
    let event = VoiceTransportCreatedV1 {
        schema: VOICE_TRANSPORT_CREATED_V1.to_string(),
        session_id: "sess-1".into(),
        transport_id: "transport-1".into(),
        direction: "send".into(),
        occurred_at: "2026-01-01T00:00:00Z".into(),
    };
    let value = serde_json::to_value(event).expect("serialize transport");
    assert_eq!(value["schema"], VOICE_TRANSPORT_CREATED_V1);
    assert_eq!(value["direction"], "send");
}

#[test]
fn voice_consumer_contract_serializes_stable_shape() {
    let event = VoiceConsumerCreatedV1 {
        schema: VOICE_CONSUMER_CREATED_V1.to_string(),
        session_id: "sess-1".into(),
        consumer_id: "consumer-1".into(),
        producer_id: "producer-1".into(),
        transport_id: "transport-1".into(),
        kind: "audio".into(),
        occurred_at: "2026-01-01T00:00:00Z".into(),
    };
    let value = serde_json::to_value(event).expect("serialize consumer");
    assert_eq!(value["schema"], VOICE_CONSUMER_CREATED_V1);
    assert_eq!(value["kind"], "audio");
}

#[test]
fn signaling_envelope_and_response_shapes_are_stable() {
    let req = VoiceSignalEnvelope {
        request_id: "req-1".into(),
        kind: VoiceSignalKind::CreateTransport,
        payload: serde_json::json!({ "sessionId": "sess-1" }),
    };
    let req_value = serde_json::to_value(req).expect("serialize request envelope");
    assert_eq!(req_value["request_id"], "req-1");
    assert_eq!(req_value["kind"], "create_transport");

    let res = VoiceSignalResponse::ok(
        "req-1".into(),
        VoiceSignalKind::CreateTransport,
        serde_json::json!({ "transportId": "t1" }),
    );
    let res_value = serde_json::to_value(res).expect("serialize signal response");
    assert_eq!(res_value["type"], "signal_response");
    assert_eq!(res_value["request_id"], "req-1");
    assert_eq!(res_value["ok"], true);

    let err = VoiceSignalResponse::err(
        "req-2".into(),
        VoiceSignalKind::CreateConsumer,
        "producer missing".into(),
    );
    let err_value = serde_json::to_value(err).expect("serialize error signal response");
    assert_eq!(err_value["type"], "signal_response");
    assert_eq!(err_value["ok"], false);
    assert_eq!(err_value["error"], "producer missing");
}
