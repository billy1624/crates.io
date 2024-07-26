use crate::builders::{CrateBuilder, VersionBuilder};
use crate::util::{RequestHelper, TestApp};
use http::StatusCode;

#[tokio::test(flavor = "multi_thread")]
async fn test_non_blocked_download_route() {
    let (app, anon, user) = TestApp::init()
        .with_config(|config| {
            config.blocked_routes.clear();
        })
        .with_user();

    app.db(|conn| {
        CrateBuilder::new("foo", user.as_model().id)
            .version(VersionBuilder::new("1.0.0"))
            .expect_build(conn);
    });

    let status = anon
        .get::<()>("https://crates.io/api/v1/crates/foo/1.0.0/download")
        .await
        .status();
    assert_eq!(status, StatusCode::FOUND);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_blocked_download_route() {
    let (app, anon, user) = TestApp::init()
        .with_config(|config| {
            config.blocked_routes.clear();
            config
                .blocked_routes
                .insert("https://crates.io/api/v1/crates/:crate_id/:version/download".into());
        })
        .with_user();

    app.db(|conn| {
        CrateBuilder::new("foo", user.as_model().id)
            .version(VersionBuilder::new("1.0.0"))
            .expect_build(conn);
    });

    let status = anon
        .get::<()>("https://crates.io/api/v1/crates/foo/1.0.0/download")
        .await
        .status();
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}
