use crate::util::{RequestHelper, TestApp};
use deadpool_diesel::postgres::Pool;
use http::StatusCode;
use std::time::{Duration, Instant};
use tracing::info;

const DB_HEALTHY_TIMEOUT: Duration = Duration::from_millis(2000);

async fn wait_until_healthy(pool: &Pool) {
    info!("Waiting for the database to become healthy…");

    let start_time = Instant::now();
    loop {
        let result = pool.get().await;
        if result.is_ok() {
            info!("Database is healthy now");
            return;
        }

        if start_time.elapsed() < DB_HEALTHY_TIMEOUT {
            info!("Database is not healthy yet, retrying…");
            tokio::time::sleep(Duration::from_millis(100)).await;
        } else {
            info!("Database did not become healthy within the timeout");
            let _ = result.expect("the database did not return healthy");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn http_error_with_unhealthy_database() {
    let (app, anon) = TestApp::init().with_chaos_proxy().empty();

    let response = anon.get::<()>("https://crates.io/api/v1/summary").await;
    assert_eq!(response.status(), StatusCode::OK);

    app.primary_db_chaosproxy().break_networking().unwrap();

    let response = anon.get::<()>("https://crates.io/api/v1/summary").await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    app.primary_db_chaosproxy().restore_networking().unwrap();
    wait_until_healthy(&app.as_inner().primary_database).await;

    let response = anon.get::<()>("https://crates.io/api/v1/summary").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn fallback_to_replica_returns_user_info() {
    const URL: &str = "https://crates.io/api/v1/users/foo";

    let (app, _, owner) = TestApp::init()
        .with_replica()
        .with_chaos_proxy()
        .with_user();
    app.db_new_user("foo");
    app.primary_db_chaosproxy().break_networking().unwrap();

    // When the primary database is down, requests are forwarded to the replica database
    let response = owner.get::<()>(URL).await;
    assert_eq!(response.status(), 200);

    // restore primary database connection
    app.primary_db_chaosproxy().restore_networking().unwrap();
    wait_until_healthy(&app.as_inner().primary_database).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn restored_replica_returns_user_info() {
    const URL: &str = "https://crates.io/api/v1/users/foo";

    let (app, _, owner) = TestApp::init()
        .with_replica()
        .with_chaos_proxy()
        .with_user();
    app.db_new_user("foo");
    app.primary_db_chaosproxy().break_networking().unwrap();
    app.replica_db_chaosproxy().break_networking().unwrap();

    // When both primary and replica database are down, the request returns an error
    let response = owner.get::<()>(URL).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    // Once the replica database is restored, it should serve as a fallback again
    app.replica_db_chaosproxy().restore_networking().unwrap();
    let replica = app
        .as_inner()
        .replica_database
        .as_ref()
        .expect("no replica database configured");
    wait_until_healthy(replica).await;

    let response = owner.get::<()>(URL).await;
    assert_eq!(response.status(), StatusCode::OK);

    // restore connection
    app.primary_db_chaosproxy().restore_networking().unwrap();
    wait_until_healthy(&app.as_inner().primary_database).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn restored_primary_returns_user_info() {
    const URL: &str = "https://crates.io/api/v1/users/foo";

    let (app, _, owner) = TestApp::init()
        .with_replica()
        .with_chaos_proxy()
        .with_user();
    app.db_new_user("foo");
    app.primary_db_chaosproxy().break_networking().unwrap();
    app.replica_db_chaosproxy().break_networking().unwrap();

    // When both primary and replica database are down, the request returns an error
    let response = owner.get::<()>(URL).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    // Once the replica database is restored, it should serve as a fallback again
    app.primary_db_chaosproxy().restore_networking().unwrap();
    wait_until_healthy(&app.as_inner().primary_database).await;

    let response = owner.get::<()>(URL).await;
    assert_eq!(response.status(), StatusCode::OK);
}
