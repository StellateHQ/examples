use crate::graphql::GraphQLInput;
use http::HeaderMap;

pub fn send(input: &GraphQLInput, headers: &HeaderMap) -> anyhow::Result<(String, u16, HeaderMap)> {
    let mut headers = headers.clone();

    // Encoding is a bitch, just don't do it for now
    headers.remove("accept-encoding");

    let client = reqwest::blocking::Client::new();
    let res = client
        .post("https://partial-query-caching-demo.vercel.app/")
        .headers(headers)
        .body(input.to_body()?)
        .send()?;

    let status = res.status().as_u16();
    let headers = res.headers().clone();
    let text = res.text()?;
    Ok((text, status, headers))
}
