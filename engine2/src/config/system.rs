use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SystemConfig {
    pub _id: String,
    #[serde(rename = "accessLogFormat")]
    pub access_log_format: String,
    #[serde(rename = "listenHttpPort")]
    pub listen_http_port: String,
    #[serde(rename = "listenHttps")]
    pub listen_https: HttpsConfig,
    #[serde(rename = "systemLogLevel")]
    pub system_log_level: String,
    pub threads: String,
}

#[derive(Serialize, Deserialize)]
pub struct HttpsConfig {
    pub _id: String,
    #[serde(rename = "certificateFileData")]
    pub certificate_file_data: String,
    #[serde(rename = "certificateFileName")]
    pub certificate_file_name: String,
    pub password: String,
    pub port: String,
    #[serde(rename = "privateKeyFileData")]
    pub private_key_file_data: String,
    #[serde(rename = "privateKeyFileName")]
    pub private_key_file_name: String,
}
