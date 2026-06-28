#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use sqlx::FromRow;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, ToSchema, FromRow, Clone, Default)]
pub struct _SqlxMigrations {
    pub version: Option<i64>,
    pub description: String,
    pub installed_on: DateTime<Utc>,
    pub success: String,
    pub checksum: Vec<u8>,
    pub execution_time: i64,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, ToSchema, FromRow, Clone, Default)]
pub struct Metadata {
    pub id: i32,
    pub property: String,
    pub value: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, ToSchema, FromRow, Clone, Default)]
pub struct Nodes {
    pub id: i32,
    pub node_id: String,
    pub app_id: String,
    pub name: String,
    pub host: String,
    pub hostname: Option<String>,
    pub rel_dirname: Option<String>,
    pub remote_home_dir: Option<String>,
    pub domain_name: String,
    pub custom_domain: Option<String>,
    pub node_url: String,
    pub ssh: String,
    pub mysql: Option<String>,
    pub backup: String,
    pub timezone_offset: Option<i32>,
    pub mimics: Option<String>,
    pub active: Option<i32>,
    pub dev_mode: Option<i32>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, ToSchema, FromRow, Clone, Default)]
pub struct Servers {
    pub id: i32,
    pub ip: String,
    pub username: String,
    pub password: Option<String>,
    pub hostname: Option<String>,
    pub provider: Option<String>,
    pub key_path: Option<String>,
}

