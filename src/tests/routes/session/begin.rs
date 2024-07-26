use crate::util::{RequestHelper, TestApp};

#[derive(Deserialize)]
struct AuthResponse {
    url: String,
    state: String,
}

#[tokio::test(flavor = "multi_thread")]
async fn auth_gives_a_token() {
    let (_, anon) = TestApp::init().empty();
    let json: AuthResponse = anon.get("https://crates.io/api/private/session/begin").await.good();
    assert!(json.url.contains(&json.state));
}
