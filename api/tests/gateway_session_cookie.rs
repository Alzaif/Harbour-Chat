use harbour_chat_api::infrastructure::http::gateway::session_cookie_value;

#[test]
fn session_cookie_value_extracts_harbour_session() {
    let cookie = "foo=bar; harbour_session=signed-value; other=baz";
    assert_eq!(session_cookie_value(Some(cookie)), Some("signed-value"));
}

#[test]
fn session_cookie_value_returns_none_when_missing() {
    assert_eq!(session_cookie_value(Some("foo=bar")), None);
    assert_eq!(session_cookie_value(None), None);
}
