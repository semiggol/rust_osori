use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize)]
pub struct Api {
    pub _id: String,
    #[serde(rename = "apiKeys")]
    pub api_keys: Vec<String>,
    #[serde(rename = "authType")]
    pub auth_type: String,
    pub author: String,
    #[serde(rename = "basePath")]
    pub base_path: String,
    pub cors: bool,
    #[serde(rename = "createTime")]
    pub create_time: String,
    pub description: String,
    #[serde(rename = "engineGroups")]
    pub engine_groups: Vec<String>,
    #[serde(rename = "latestVersion")]
    pub latest_version: bool,
    pub methods: Vec<String>,
    pub name: String,
    #[serde(rename = "targetPath")]
    pub target_path: String,
    #[serde(rename = "targetServers")]
    pub target_servers: Vec<String>,
    pub version: usize,
}

