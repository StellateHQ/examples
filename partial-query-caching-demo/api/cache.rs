use anyhow::anyhow;
use http::header;
use serde_json::json;
use std::env;

fn get_edge_config_id() -> anyhow::Result<String> {
    env::var("EDGE_CONFIG_ID").map_err(|e| anyhow!("Missing EDGE_CONFIG_ID: {}", e))
}

fn get_team_id() -> anyhow::Result<String> {
    env::var("TEAM_ID").map_err(|e| anyhow!("Missing TEAM_ID: {}", e))
}

pub fn get_item(key: &str) -> anyhow::Result<String> {
    let token = env::var("EDGE_CONFIG_TOKEN")?;
    let config_id = get_edge_config_id()?;

    let client = reqwest::blocking::Client::new();
    let res = client
        .get(format!(
            "https://edge-config.vercel.com/{config_id}/item/{key}"
        ))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .send()?;
    Ok(serde_json::from_str::<String>(&res.text()?)?)
}

pub fn set_item(key: &str, value: &str) -> anyhow::Result<()> {
    let token = env::var("VERCEL_API_TOKEN")?;
    let config_id = get_edge_config_id()?;
    let team_id = get_team_id()?;

    let client = reqwest::blocking::Client::new();
    let res = client
        .patch(format!(
            "https://api.vercel.com/v1/edge-config/{config_id}/items?teamId={team_id}"
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
        Err(anyhow!("Request failed: {:?}", res.text()?))
    }
}
