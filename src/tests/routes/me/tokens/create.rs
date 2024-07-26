use crate::util::insta::{self, assert_json_snapshot};
use crate::util::{RequestHelper, TestApp};
use crates_io::models::token::{CrateScope, EndpointScope};
use crates_io::models::ApiToken;
use diesel::prelude::*;
use googletest::prelude::*;
use http::StatusCode;
use serde_json::Value;

static NEW_BAR: &[u8] = br#"{ "api_token": { "name": "bar" } }"#;

#[tokio::test(flavor = "multi_thread")]
async fn create_token_logged_out() {
    let (_, anon) = TestApp::init().empty();
    anon.put("https://crates.io/api/v1/me/tokens", NEW_BAR)
        .await
        .assert_forbidden();
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_invalid_request() {
    let (_, _, user) = TestApp::init().with_user();
    let invalid: &[u8] = br#"{ "name": "" }"#;
    let response = user.put::<()>("https://crates.io/api/v1/me/tokens", invalid).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "invalid new token request: Error(\"missing field `api_token`\", line: 1, column: 14)" }] })
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_no_name() {
    let (_, _, user) = TestApp::init().with_user();
    let empty_name: &[u8] = br#"{ "api_token": { "name": "" } }"#;
    let response = user.put::<()>("https://crates.io/api/v1/me/tokens", empty_name).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "name must have a value" }] })
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_exceeded_tokens_per_user() {
    let (app, _, user) = TestApp::init().with_user();
    let id = user.as_model().id;
    app.db(|conn| {
        for i in 0..1000 {
            assert_ok!(ApiToken::insert(conn, id, &format!("token {i}")));
        }
    });
    let response = user.put::<()>("https://crates.io/api/v1/me/tokens", NEW_BAR).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "maximum tokens per user is: 500" }] })
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_success() {
    let (app, _, user) = TestApp::init().with_user();

    let response = user.put::<()>("https://crates.io/api/v1/me/tokens", NEW_BAR).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json(), {
        ".api_token.id" => insta::any_id_redaction(),
        ".api_token.created_at" => "[datetime]",
        ".api_token.last_used_at" => "[datetime]",
        ".api_token.token" => insta::api_token_redaction(),
    });

    let tokens: Vec<ApiToken> = app.db(|conn| {
        assert_ok!(ApiToken::belonging_to(user.as_model())
            .select(ApiToken::as_select())
            .load(conn))
    });
    assert_that!(tokens, len(eq(1)));
    assert_eq!(tokens[0].name, "bar");
    assert!(!tokens[0].revoked);
    assert_eq!(tokens[0].last_used_at, None);
    assert_eq!(tokens[0].crate_scopes, None);
    assert_eq!(tokens[0].endpoint_scopes, None);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_multiple_have_different_values() {
    let (_, _, user) = TestApp::init().with_user();
    let first: Value = user.put("https://crates.io/api/v1/me/tokens", NEW_BAR).await.good();
    let second: Value = user.put("https://crates.io/api/v1/me/tokens", NEW_BAR).await.good();

    assert_eq!(first["api_token"]["name"], second["api_token"]["name"]);
    assert_ne!(first["api_token"]["token"], second["api_token"]["token"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_multiple_users_have_different_values() {
    let (app, _, user1) = TestApp::init().with_user();
    let first: Value = user1.put("https://crates.io/api/v1/me/tokens", NEW_BAR).await.good();

    let user2 = app.db_new_user("bar");
    let second: Value = user2.put("https://crates.io/api/v1/me/tokens", NEW_BAR).await.good();

    assert_ne!(first["api_token"]["token"], second["api_token"]["token"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn cannot_create_token_with_token() {
    let (_, _, _, token) = TestApp::init().with_token();
    let response = token
        .put::<()>(
            "https://crates.io/api/v1/me/tokens",
            br#"{ "api_token": { "name": "baz" } }"# as &[u8],
        )
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "cannot use an API token to create a new API token" }] })
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_with_scopes() {
    let (app, _, user) = TestApp::init().with_user();

    let json = json!({
        "api_token": {
            "name": "bar",
            "crate_scopes": ["tokio", "tokio-*"],
            "endpoint_scopes": ["publish-update"],
        }
    });

    let response = user
        .put::<()>("https://crates.io/api/v1/me/tokens", serde_json::to_vec(&json).unwrap())
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json(), {
        ".api_token.id" => insta::any_id_redaction(),
        ".api_token.created_at" => "[datetime]",
        ".api_token.last_used_at" => "[datetime]",
        ".api_token.token" => insta::api_token_redaction(),
    });

    let tokens: Vec<ApiToken> = app.db(|conn| {
        assert_ok!(ApiToken::belonging_to(user.as_model())
            .select(ApiToken::as_select())
            .load(conn))
    });
    assert_that!(tokens, len(eq(1)));
    assert_eq!(tokens[0].name, "bar");
    assert!(!tokens[0].revoked);
    assert_eq!(tokens[0].last_used_at, None);
    assert_eq!(
        tokens[0].crate_scopes,
        Some(vec![
            CrateScope::try_from("tokio").unwrap(),
            CrateScope::try_from("tokio-*").unwrap()
        ])
    );
    assert_eq!(
        tokens[0].endpoint_scopes,
        Some(vec![EndpointScope::PublishUpdate])
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_with_null_scopes() {
    let (app, _, user) = TestApp::init().with_user();

    let json = json!({
        "api_token": {
            "name": "bar",
            "crate_scopes": null,
            "endpoint_scopes": null,
        }
    });

    let response = user
        .put::<()>("https://crates.io/api/v1/me/tokens", serde_json::to_vec(&json).unwrap())
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json(), {
        ".api_token.id" => insta::any_id_redaction(),
        ".api_token.created_at" => "[datetime]",
        ".api_token.last_used_at" => "[datetime]",
        ".api_token.token" => insta::api_token_redaction(),
    });

    let tokens: Vec<ApiToken> = app.db(|conn| {
        assert_ok!(ApiToken::belonging_to(user.as_model())
            .select(ApiToken::as_select())
            .load(conn))
    });
    assert_that!(tokens, len(eq(1)));
    assert_eq!(tokens[0].name, "bar");
    assert!(!tokens[0].revoked);
    assert_eq!(tokens[0].last_used_at, None);
    assert_eq!(tokens[0].crate_scopes, None);
    assert_eq!(tokens[0].endpoint_scopes, None);
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_with_empty_crate_scope() {
    let (_, _, user) = TestApp::init().with_user();

    let json = json!({
        "api_token": {
            "name": "bar",
            "crate_scopes": ["", "tokio-*"],
            "endpoint_scopes": ["publish-update"],
        }
    });

    let response = user
        .put::<()>("https://crates.io/api/v1/me/tokens", serde_json::to_vec(&json).unwrap())
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "invalid crate scope" }] })
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_with_invalid_endpoint_scope() {
    let (_, _, user) = TestApp::init().with_user();

    let json = json!({
        "api_token": {
            "name": "bar",
            "crate_scopes": ["tokio", "tokio-*"],
            "endpoint_scopes": ["crash"],
        }
    });

    let response = user
        .put::<()>("https://crates.io/api/v1/me/tokens", serde_json::to_vec(&json).unwrap())
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "invalid endpoint scope" }] })
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_token_with_expiry_date() {
    let (_app, _, user) = TestApp::init().with_user();

    let json = json!({
        "api_token": {
            "name": "bar",
            "crate_scopes": null,
            "endpoint_scopes": null,
            "expired_at": "2024-12-24T12:34:56+05:00",
        }
    });

    let response = user
        .put::<()>("https://crates.io/api/v1/me/tokens", serde_json::to_vec(&json).unwrap())
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json(), {
        ".api_token.id" => insta::any_id_redaction(),
        ".api_token.created_at" => "[datetime]",
        ".api_token.last_used_at" => "[datetime]",
        ".api_token.token" => insta::api_token_redaction(),
    });
}
