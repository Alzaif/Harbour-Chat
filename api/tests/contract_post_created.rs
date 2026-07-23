use harbour_chat_api::contracts::events::{PostCreatedV1, POST_CREATED_V1};

#[test]
fn post_created_contract_serializes_stable_shape() {
    let event = PostCreatedV1::new(
        "post-1".into(),
        "user-1".into(),
        "2026-01-01T00:00:00Z".into(),
    );
    let value = serde_json::to_value(&event).expect("serialize post created");
    assert_eq!(value["schema"], POST_CREATED_V1);
    assert_eq!(value["post_id"], "post-1");
    assert_eq!(value["author_user_id"], "user-1");
}
