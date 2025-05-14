use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    #[serde(default)]
    pub cache_config: HashMap<String, CacheConfigType>,

    #[serde(default)]
    pub scopes: HashMap<String, ConfigScope>,

    #[serde(default = "default_key_fields")]
    pub default_key_fields: CacheKeyFields,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CacheConfigType {
    cache_control: Option<CacheControl>,
    key_fields: Option<CacheKeyFields>,
    fields: HashMap<String, CacheConfigField>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CacheConfigField {
    cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CacheControl {
    max_age: Option<u64>,
    swr: Option<u64>,
    scope: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CacheKeyFields(HashMap<String, bool>);

impl Default for CacheKeyFields {
    fn default() -> Self {
        Self(HashMap::from([
            (String::from("id"), true),
            (String::from("_id"), true),
            (String::from("key"), true),
        ]))
    }
}

impl CacheKeyFields {
    pub fn keys(&self) -> Vec<&str> {
        self.0.keys().into_iter().map(|key| key.as_ref()).collect()
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigScope {
    header: Vec<String>,
    cookie: Vec<String>,
    jwt: Option<JwtDefinition>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JwtDefinition {
    claim: String,
    algorithm: JwtAlgorithm,
    secret: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum JwtAlgorithm {
    HS256,
    HS384,
    HS512,
    RS256,
    RS384,
    RS512,
    ES256,
    ES384,
    ES256k,
    EdDSA,
    PS256,
    PS384,
    PS512,
}

fn default_key_fields() -> CacheKeyFields {
    CacheKeyFields(HashMap::from([
        (String::from("id"), true),
        (String::from("_id"), true),
        (String::from("key"), true),
    ]))
}

impl Manifest {
    pub fn type_max_age(&self, type_name: &str) -> Option<u64> {
        self.cache_config
            .get(type_name)
            .and_then(|type_config| type_config.cache_control.as_ref())
            .and_then(|cache_control| cache_control.max_age)
    }

    pub fn field_max_age(&self, type_name: &str, field_name: &str) -> Option<u64> {
        self.cache_config
            .get(type_name)
            .and_then(|type_config| type_config.fields.get(field_name))
            .and_then(|field_config| field_config.cache_control.as_ref())
            .and_then(|cache_control| cache_control.max_age)
    }

    pub fn key_field_names(&self, type_name: &str) -> Vec<&str> {
        self.cache_config
            .get(type_name)
            .and_then(|type_config| type_config.key_fields.as_ref())
            .map(|key_fields| key_fields.keys())
            .unwrap_or(self.default_key_fields.keys())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let manifest: Manifest = serde_json::from_str("{}").unwrap();
        assert_eq!(
            manifest,
            Manifest {
                cache_config: HashMap::default(),
                scopes: HashMap::default(),
                default_key_fields: default_key_fields()
            }
        )
    }

    #[test]
    fn deserialize_rules_minimal() {
        let manifest: Manifest =
            serde_json::from_str(r#"{"cacheConfig":{"Query":{"cacheControl":{"maxAge":42,"swr":null,"scope":null},"keyFields":null,"fields":{}}}}"#)
                .unwrap();
        assert_eq!(
            manifest,
            Manifest {
                cache_config: HashMap::from([(
                    String::from("Query"),
                    CacheConfigType {
                        cache_control: Some(CacheControl {
                            max_age: Some(42),
                            swr: None,
                            scope: None
                        }),
                        key_fields: None,
                        fields: HashMap::default()
                    },
                )]),
                scopes: HashMap::new(),
                default_key_fields: default_key_fields()
            }
        );
    }

    #[test]
    fn deserialize_rules_types_full() {
        let manifest: Manifest = serde_json::from_str(
            r#"{"cacheConfig":{
              "Query":{"cacheControl":{"maxAge":42,"swr":43,"scope":"AUTH"},"keyFields":null,"fields":{}},
              "Post":{"cacheControl":null,"keyFields":null,"fields":{"content":{"cacheControl":{"maxAge":42,"swr":43,"scope":"AUTH"}}}}
            }}"#,
        )
        .unwrap();
        assert_eq!(
            manifest,
            Manifest {
                cache_config: HashMap::from([
                    (
                        String::from("Query"),
                        CacheConfigType {
                            cache_control: Some(CacheControl {
                                max_age: Some(42),
                                swr: Some(43),
                                scope: Some(String::from("AUTH"))
                            }),
                            key_fields: None,
                            fields: HashMap::default()
                        },
                    ),
                    (
                        String::from("Post"),
                        CacheConfigType {
                            cache_control: None,
                            key_fields: None,
                            fields: HashMap::from([(
                                String::from("content"),
                                CacheConfigField {
                                    cache_control: Some(CacheControl {
                                        max_age: Some(42),
                                        swr: Some(43),
                                        scope: Some(String::from("AUTH"))
                                    })
                                }
                            )])
                        },
                    )
                ]),
                scopes: HashMap::new(),
                default_key_fields: default_key_fields()
            }
        );
    }

    #[test]
    fn deserialize_scopes() {
        let manifest: Manifest =
            serde_json::from_str(r#"{"scopes":{"AUTH":{"header":["authorization"],"cookie":[]}}}"#)
                .unwrap();
        assert_eq!(
            manifest,
            Manifest {
                cache_config: HashMap::default(),
                scopes: HashMap::from([(
                    String::from("AUTH"),
                    ConfigScope {
                        header: vec![String::from("authorization")],
                        cookie: vec![],
                        jwt: None
                    }
                )]),
                default_key_fields: default_key_fields()
            }
        );
    }
}
