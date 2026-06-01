use harbour_chat_api::contracts::events::{MessageSentV1, MESSAGE_SENT_V1};

#[test]
fn message_sent_v1_round_trip() {
    let event = MessageSentV1::new(
        "msg-1".into(),
        "chan-1".into(),
        "srv-1".into(),
        "user-1".into(),
        "2026-05-27T12:00:00Z".into(),
    );
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(MESSAGE_SENT_V1));
    let parsed: MessageSentV1 = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, event);
}
