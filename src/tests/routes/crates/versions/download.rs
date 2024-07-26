use crate::builders::{CrateBuilder, VersionBuilder};
use crate::util::{RequestHelper, TestApp};

#[tokio::test(flavor = "multi_thread")]
async fn test_redirects() {
    let (app, anon, user) = TestApp::init().with_user();

    app.db(|conn| {
        CrateBuilder::new("foo-download", user.as_model().id)
            .version(VersionBuilder::new("1.0.0"))
            .expect_build(conn);
    });

    // Any redirect to an existing crate and version works correctly.
    anon.get::<()>("https://crates.io/api/v1/crates/foo-download/1.0.0/download")
        .await
        .assert_redirect_ends_with("/crates/foo-download/foo-download-1.0.0.crate");

    // Redirects to crates with wrong capitalization are performed unconditionally.
    anon.get::<()>("https://crates.io/api/v1/crates/Foo_downloaD/1.0.0/download")
        .await
        .assert_redirect_ends_with("/crates/Foo_downloaD/Foo_downloaD-1.0.0.crate");

    // Redirects to missing versions are performed unconditionally.
    anon.get::<()>("https://crates.io/api/v1/crates/foo-download/2.0.0/download")
        .await
        .assert_redirect_ends_with("/crates/foo-download/foo-download-2.0.0.crate");

    // Redirects to missing crates are performed unconditionally.
    anon.get::<()>("https://crates.io/api/v1/crates/bar-download/1.0.0/download")
        .await
        .assert_redirect_ends_with("/crates/bar-download/bar-download-1.0.0.crate");
}

#[tokio::test(flavor = "multi_thread")]
async fn download_with_build_metadata() {
    let (app, anon, user) = TestApp::init().with_user();
    let user = user.as_model();

    app.db(|conn| {
        CrateBuilder::new("foo", user.id)
            .version(VersionBuilder::new("1.0.0+bar"))
            .expect_build(conn);
    });

    anon.get::<()>("https://crates.io/api/v1/crates/foo/1.0.0+bar/download")
        .await
        .assert_redirect_ends_with("/crates/foo/foo-1.0.0%2Bbar.crate");

    anon.get::<()>("https://crates.io/api/v1/crates/foo/1.0.0+bar/readme")
        .await
        .assert_redirect_ends_with("/readmes/foo/foo-1.0.0%2Bbar.html");
}
