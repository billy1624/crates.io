use crate::builders::CrateBuilder;
use crate::util::{RequestHelper, TestApp};
use http::status::StatusCode;
use ipnetwork::IpNetwork;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn pagination_blocks_ip_from_cidr_block_list() {
    let (app, anon, user) = TestApp::init()
        .with_config(|config| {
            config.max_allowed_page_offset = 1;
            config.page_offset_cidr_blocklist = vec!["127.0.0.1/24".parse::<IpNetwork>().unwrap()];
        })
        .with_user();
    let user = user.as_model();

    app.db(|conn| {
        CrateBuilder::new("pagination_links_1", user.id).expect_build(conn);
        CrateBuilder::new("pagination_links_2", user.id).expect_build(conn);
        CrateBuilder::new("pagination_links_3", user.id).expect_build(conn);
    });

    let response = anon
        .get_with_query::<()>("https://crates.io/api/v1/crates", "page=2&per_page=1")
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "Page 2 is unavailable for performance reasons. Please take a look at https://crates.io/data-access for alternatives." }] })
    );
}
