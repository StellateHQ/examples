use anyhow::anyhow;
use http::header;
use serde_json::json;

const EDGE_CONFIG_ID: &str = "ecfg_z8s2mz4i9mlljcsjrlipefel0stx";
const TEAM_ID: &str = "team_IrmIsd17rfwBUaDzlhndCGAe";

pub fn get_item(key: &str) -> anyhow::Result<String> {
    let token = std::env::var("EDGE_CONFIG_TOKEN")?;

    let client = reqwest::blocking::Client::new();
    let res = client
        .get(format!(
            "https://edge-config.vercel.com/{EDGE_CONFIG_ID}/item/{key}"
        ))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .send()?;
    Ok(serde_json::from_str::<String>(&res.text()?)?)
}

pub fn set_item(key: &str, value: &str) -> anyhow::Result<()> {
    let token = std::env::var("VERCEL_API_TOKEN")?;

    let client = reqwest::blocking::Client::new();
    let res = client
        .patch(format!(
            "https://api.vercel.com/v1/edge-config/{EDGE_CONFIG_ID}/items?teamId={TEAM_ID}"
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(
            json!({ "items": [{ "operation": "upsert", "key": key, "value": value }] }).to_string(),
        )
        .send()?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(anyhow!("Request failed: {:?}", res.text()))
    }
}
