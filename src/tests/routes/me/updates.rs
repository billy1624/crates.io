use crate::builders::{CrateBuilder, VersionBuilder};
use crate::util::{RequestHelper, TestApp};
use crate::OkBool;
use crates_io::schema::versions;
use crates_io::views::EncodableVersion;
use diesel::prelude::*;
use diesel::update;
use googletest::prelude::*;
use http::StatusCode;

#[tokio::test(flavor = "multi_thread")]
async fn api_token_cannot_get_user_updates() {
    let (_, _, _, token) = TestApp::init().with_token();
    token.get("https://crates.io/api/v1/me/updates").await.assert_forbidden();
}

#[tokio::test(flavor = "multi_thread")]
async fn following() {
    #[derive(Deserialize)]
    struct R {
        versions: Vec<EncodableVersion>,
        meta: Meta,
    }
    #[derive(Deserialize)]
    struct Meta {
        more: bool,
    }

    let (app, _, user) = TestApp::init().with_user();
    let user_model = user.as_model();
    let user_id = user_model.id;
    app.db(|conn| {
        CrateBuilder::new("foo_fighters", user_id)
            .version(VersionBuilder::new("1.0.0"))
            .expect_build(conn);

        // Make foo_fighters's version mimic a version published before we started recording who
        // published versions
        let none: Option<i32> = None;
        update(versions::table)
            .set(versions::published_by.eq(none))
            .execute(conn)
            .unwrap();

        CrateBuilder::new("bar_fighters", user_id)
            .version(VersionBuilder::new("1.0.0"))
            .expect_build(conn);
    });

    let r: R = user.get("https://crates.io/api/v1/me/updates").await.good();
    assert_that!(r.versions, empty());
    assert!(!r.meta.more);

    user.put::<OkBool>("https://crates.io/api/v1/crates/foo_fighters/follow", b"" as &[u8])
        .await
        .good();
    user.put::<OkBool>("https://crates.io/api/v1/crates/bar_fighters/follow", b"" as &[u8])
        .await
        .good();

    let r: R = user.get("https://crates.io/api/v1/me/updates").await.good();
    assert_that!(r.versions, len(eq(2)));
    assert!(!r.meta.more);
    let foo_version = r
        .versions
        .iter()
        .find(|v| v.krate == "foo_fighters")
        .unwrap();
    assert_none!(&foo_version.published_by);
    let bar_version = r
        .versions
        .iter()
        .find(|v| v.krate == "bar_fighters")
        .unwrap();
    assert_eq!(
        bar_version.published_by.as_ref().unwrap().login,
        user_model.gh_login
    );

    let r: R = user
        .get_with_query("https://crates.io/api/v1/me/updates", "per_page=1")
        .await
        .good();
    assert_that!(r.versions, len(eq(1)));
    assert!(r.meta.more);

    user.delete::<OkBool>("https://crates.io/api/v1/crates/bar_fighters/follow")
        .await
        .good();
    let r: R = user
        .get_with_query("https://crates.io/api/v1/me/updates", "page=2&per_page=1")
        .await
        .good();
    assert_that!(r.versions, empty());
    assert!(!r.meta.more);

    let response = user
        .get_with_query::<()>("https://crates.io/api/v1/me/updates", "page=0")
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "page indexing starts from 1, page 0 is invalid" }] })
    );
}
