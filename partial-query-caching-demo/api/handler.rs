mod cache;
mod graphql;
mod manifest;
mod merge;
mod request;
mod schema;
mod split;

use anyhow::anyhow;
use gql_query::{
    ast::ASTContext,
    schema::{BuildClientSchema, IntrospectionQuery},
};
use graphql::GraphQLInput;
use http::{HeaderMap, Method};
use manifest::Manifest;
use merge::response::{merge_responses, PartialQuery};
use schema::{MANIFEST, SCHEMA};
use split::split;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let res = match handle_with_error(req) {
        Ok(res) => res,
        Err(err) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::Text(format!(
                "Stl8 Internal Server Error: {err}\n{:?}",
                err.source()
            )))?,
    };
    Ok(res)
}

fn handle_with_error(req: Request) -> anyhow::Result<Response<Body>> {
    if req.method() == Method::OPTIONS {
        return Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("access-control-allow-credentials", "true")
            .header("access-control-allow-headers", "*")
            .header("access-control-allow-methods", "GET, POST, OPTIONS")
            .header("access-control-allow-origin", "*")
            .header("access-control-expose-headers", "*")
            .header("access-control-max-age", "3600")
            .body(Body::Empty)?);
    }

    let ctx = ASTContext::new();
    let body_bytes = match req.body() {
        Body::Binary(bytes) => bytes,
        Body::Text(str) => str.as_bytes(),
        Body::Empty => return Err(anyhow!("Missing request body")),
    };
    let input = GraphQLInput::new(&ctx, body_bytes)?;

    let manifest = serde_json::from_slice::<Manifest>(
        req.headers()
            .get("stellate-manifest")
            .map(|v| v.as_bytes())
            .unwrap_or_default(),
    )
    .unwrap_or(serde_json::from_str::<Manifest>(MANIFEST).unwrap());

    let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
    let schema = introspection.build_client_schema(&ctx);
    let splits = split(&input, schema, &manifest)?;

    let mut partial_queries = vec![];
    let mut status = 200;
    let mut headers = HeaderMap::default();
    for (_, document) in splits.iter() {
        let input = GraphQLInput {
            ctx: &ctx,
            document,
            operation_name: input.operation_name.clone(),
            variables: input.variables.clone(),
            extensions: input.extensions.clone(),
        };
        let (response, partial_status, partial_headers) = request::send(&input, req.headers())?;

        partial_queries.push(PartialQuery { input, response });

        // TODO: how to handle if some parts return errors? For now let's just combine it somehow
        status = status.max(partial_status);
        for (key, value) in partial_headers {
            if let Some(key) = key {
                headers.append(key, value);
            }
        }
    }

    let res = merge_responses(&input, partial_queries)?;

    let mut builder = Response::builder().status(status);
    for (key, value) in headers {
        if let Some(key) = key {
            builder = builder.header(key, value);
        }
    }

    Ok(builder.header("gcdn-cache", "PASS").body(res.into())?)
}
