use axum::extract::DefaultBodyLimit;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;
use http::{Method, StatusCode};

use crate::app::AppState;
use crate::controllers::*;
use crate::util::errors::not_found;
use crate::Env;

const MAX_PUBLISH_CONTENT_LENGTH: usize = 128 * 1024 * 1024; // 128 MB

pub fn build_axum_router(state: AppState) -> Router<()> {
    let mut router = Router::new()
        // Route used by both `cargo search` and the frontend
        .route("https://crates.io/api/v1/crates", get(krate::search::search))
        // Routes used by `cargo`
        .route(
            "https://crates.io/api/v1/crates/new",
            put(krate::publish::publish)
                .layer(DefaultBodyLimit::max(MAX_PUBLISH_CONTENT_LENGTH))
                .get(krate::metadata::show_new),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/owners",
            get(krate::owners::owners)
                .put(krate::owners::add_owners)
                .delete(krate::owners::remove_owners),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/:version/yank",
            delete(version::yank::yank),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/:version/unyank",
            put(version::yank::unyank),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/:version/download",
            get(version::downloads::download),
        )
        // Routes used by the frontend
        .route("https://crates.io/api/v1/crates/:crate_id", get(krate::metadata::show))
        .route(
            "https://crates.io/api/v1/crates/:crate_id/:version",
            get(version::metadata::show),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/:version/readme",
            get(krate::metadata::readme),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/:version/dependencies",
            get(version::metadata::dependencies),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/:version/downloads",
            get(version::downloads::downloads),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/:version/authors",
            get(version::metadata::authors),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/downloads",
            get(krate::downloads::downloads),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/versions",
            get(krate::versions::versions),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/follow",
            put(krate::follow::follow).delete(krate::follow::unfollow),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/following",
            get(krate::follow::following),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/owner_team",
            get(krate::owners::owner_team),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/owner_user",
            get(krate::owners::owner_user),
        )
        .route(
            "https://crates.io/api/v1/crates/:crate_id/reverse_dependencies",
            get(krate::metadata::reverse_dependencies),
        )
        .route("https://crates.io/api/v1/keywords", get(keyword::index))
        .route("https://crates.io/api/v1/keywords/:keyword_id", get(keyword::show))
        .route("https://crates.io/api/v1/categories", get(category::index))
        .route("https://crates.io/api/v1/categories/:category_id", get(category::show))
        .route("https://crates.io/api/v1/category_slugs", get(category::slugs))
        .route(
            "https://crates.io/api/v1/users/:user_id",
            get(user::other::show).put(user::me::update_user),
        )
        .route("https://crates.io/api/v1/users/:user_id/stats", get(user::other::stats))
        .route("https://crates.io/api/v1/teams/:team_id", get(team::show_team))
        .route("https://crates.io/api/v1/me", get(user::me::me))
        .route("https://crates.io/api/v1/me/updates", get(user::me::updates))
        .route("https://crates.io/api/v1/me/tokens", get(token::list).put(token::new))
        .route(
            "https://crates.io/api/v1/me/tokens/:id",
            get(token::show).delete(token::revoke),
        )
        .route("https://crates.io/api/v1/tokens/current", delete(token::revoke_current))
        .route(
            "https://crates.io/api/v1/me/crate_owner_invitations",
            get(crate_owner_invitation::list),
        )
        .route(
            "https://crates.io/api/v1/me/crate_owner_invitations/:crate_id",
            put(crate_owner_invitation::handle_invite),
        )
        .route(
            "https://crates.io/api/v1/me/crate_owner_invitations/accept/:token",
            put(crate_owner_invitation::handle_invite_with_token),
        )
        .route(
            "https://crates.io/api/v1/me/email_notifications",
            put(user::me::update_email_notifications),
        )
        .route("https://crates.io/api/v1/summary", get(summary::summary))
        .route(
            "https://crates.io/api/v1/confirm/:email_token",
            put(user::me::confirm_user_email),
        )
        .route(
            "https://crates.io/api/v1/users/:user_id/resend",
            put(user::me::regenerate_token_and_send),
        )
        .route(
            "https://crates.io/api/v1/site_metadata",
            get(site_metadata::show_deployed_sha),
        )
        // Session management
        .route("https://crates.io/api/private/session/begin", get(user::session::begin))
        .route(
            "https://crates.io/api/private/session/authorize",
            get(user::session::authorize),
        )
        .route("https://crates.io/api/private/session", delete(user::session::logout))
        // Metrics
        .route("https://crates.io/api/private/metrics/:kind", get(metrics::prometheus))
        // Crate ownership invitations management in the frontend
        .route(
            "https://crates.io/api/private/crate_owner_invitations",
            get(crate_owner_invitation::private_list),
        )
        // Alerts from GitHub scanning for exposed API tokens
        .route(
            "https://crates.io/api/github/secret-scanning/verify",
            post(github::secret_scanning::verify),
        );

    // Only serve the local checkout of the git index in development mode.
    // In production, for crates.io, cargo gets the index from
    // https://github.com/rust-lang/crates.io-index directly
    // or from the sparse index CDN https://index.crates.io.
    if state.config.env() == Env::Development {
        router = router.route(
            "/git/index/*path",
            get(git::http_backend).post(git::http_backend),
        );
    }

    router
        .fallback(|method: Method| async move {
            match method {
                Method::HEAD => StatusCode::NOT_FOUND.into_response(),
                _ => not_found().into_response(),
            }
        })
        .with_state(state)
}
