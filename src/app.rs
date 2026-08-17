use axum::{Json, Router, middleware, response::Redirect, routing::get};
use utoipa::{Modify, OpenApi};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};
use worker::Env;

use crate::auth::ensure_authorized;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "turborepo-cf-workers",
        description = "We have Turborepo at home."
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "cache", description = "Cache operations.")
    ),
    external_docs(
        url = "https://turborepo.com/docs/core-concepts/remote-caching#self-hosting",
        description = "The manual"
    ),
    servers(
        (url = "/", description = "Yes.")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};

        openapi
            .components
            .get_or_insert_default()
            .add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
    }
}

fn protected_router() -> OpenApiRouter<Env> {
    OpenApiRouter::default()
        .routes(routes!(crate::routes::status::caching_status))
        .routes(routes!(crate::routes::events::record_events))
        .routes(routes!(
            crate::routes::artifacts::get_artifact,
            crate::routes::artifacts::head_artifact,
            crate::routes::artifacts::put_artifact,
        ))
}

fn preflight_router() -> OpenApiRouter<Env> {
    OpenApiRouter::default()
        .routes(routes!(crate::routes::preflight::preflight_events))
        .routes(routes!(crate::routes::preflight::preflight_artifact,))
}

fn api_router(env: &Env) -> OpenApiRouter<Env> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(
            protected_router().route_layer(middleware::from_fn_with_state(
                env.clone(),
                ensure_authorized,
            )),
        )
        .merge(preflight_router())
}

pub(crate) fn router(env: Env) -> Router {
    let (router, openapi) = api_router(&env).split_for_parts();
    let openapi_json = openapi.clone();

    router
        .route("/", get(|| async { Redirect::temporary("/docs") }))
        .route(
            "/openapi.json",
            get(move || {
                let openapi = openapi_json.clone();
                async move { Json(openapi) }
            }),
        )
        .merge(Scalar::with_url("/docs", openapi))
        .with_state(env)
}

#[cfg(test)]
mod tests {
    use utoipa::OpenApi;
    use utoipa_axum::router::OpenApiRouter;
    use worker::Env;

    use super::{ApiDoc, preflight_router, protected_router};

    #[test]
    fn generates_openapi_document() {
        let document = OpenApiRouter::<Env>::with_openapi(ApiDoc::openapi())
            .merge(protected_router())
            .merge(preflight_router())
            .into_openapi();
        assert!(document.paths.paths.contains_key("/v8/artifacts/{hash}"));
        assert!(
            document
                .components
                .expect("OpenAPI components")
                .security_schemes
                .contains_key("bearerAuth")
        );
    }
}
