use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use std::time::Duration;

// for global api map
lazy_static! {
    static ref GLOBAL_API_VIEW: AtomicUsize = AtomicUsize::new(0);
    static ref GLOBAL_API_MAP_LEFT: RwLock<Map> = RwLock::new(Map::new());
    static ref GLOBAL_API_MAP_RIGHT: RwLock<Map> = RwLock::new(Map::new());
}

// atomic operation for gloval view/map
fn get_gloval_view() -> usize {
    GLOBAL_API_VIEW.load(Ordering::SeqCst) & 1
}

// change the global view for api map
fn change_global_view() {
    GLOBAL_API_VIEW.fetch_add(1, Ordering::SeqCst);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeserializedApi {
    pub _id: String,
    pub api_keys: Vec<String>,
    pub auth_type: String,
    pub author: String,
    pub base_path: String,
    pub cors: bool,
    pub create_time: String,
    pub description: String,
    pub engine_groups: Vec<String>,
    pub latest_version: bool,
    pub methods: Vec<String>,
    pub name: String,
    pub target_path: String,
    pub target_servers: Vec<String>,
    pub version: usize,
}

#[derive(Debug, Clone)]
pub struct ManagedApi {
    pub match_prefix: bool,
    pub de_api: DeserializedApi,
}

impl ManagedApi {
    pub fn new(de_api: DeserializedApi) -> Self {
        let mut m_api = ManagedApi {
            match_prefix: false,
            de_api,
        };
        m_api.fix_matchtype_and_remove_asterisk();
        m_api
    }

    pub fn get_key(&self) -> String {
        self.de_api.base_path.clone()
    }

    fn fix_matchtype_and_remove_asterisk(&mut self) {
        // fix match type
        let mut len = self.de_api.base_path.len();
        let mut path = self.de_api.base_path.as_bytes();
        if path[len - 1] == b'*' {
            // set
            self.match_prefix = true;
            // remove '*' in base_path
            self.de_api.base_path = self.de_api.base_path[0..len - 1].to_string();

            // ToDo: how to process '*'?
            len = self.de_api.target_path.len();
            path = self.de_api.target_path.as_bytes();
            if path[len - 1] == b'*' {
                // remove '*' in target_path
                self.de_api.target_path = self.de_api.target_path[0..len - 1].to_string();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    exact_match: HashMap<String, ManagedApi>,
    prefix_match: Vec<ManagedApi>,
}

impl Map {
    // constructor
    pub fn new() -> Self {
        let exact_match: HashMap<String, ManagedApi> = HashMap::new();
        let prefix_match: Vec<ManagedApi> = Vec::new();
        Map {
            exact_match,
            prefix_match,
        }
    }

    // find api
    pub fn find(&self, method: &str, uri: &str) -> Option<ManagedApi> {
        // 1. get from exact map
        match self.exact_match.get(uri) {
            Some(m_api) => {
                // found
                for api_method in m_api.de_api.methods.iter() {
                    if method.eq(api_method) {
                        // ToDo: need to deep copy?
                        return Some(m_api.clone());
                    }
                }
            }
            None => {}
        };

        // 2. get from prefix map ..
        for m_api in &self.prefix_match {
            let uri_len = uri.len();
            let prefix_len = m_api.de_api.base_path.len();
            let prefix_path = &m_api.de_api.base_path[..];

            if prefix_len <= uri_len {
                let sliced_uri = &uri[..prefix_len];
                if sliced_uri.eq(prefix_path) {
                    // found
                    for api_method in m_api.de_api.methods.iter() {
                        if method.eq(api_method) {
                            // ToDo: need to deep copy?
                            let mut found_api = m_api.clone();
                            let remaining_uri = &uri[prefix_len - 1..];
                            found_api.de_api.target_path.push_str(remaining_uri);

                            // target_path has been changed! -> use target_path for proxy request
                            return Some(found_api);
                        }
                    }
                }
            }
        }

        // Not Found
        None
    }

    // private: clear hashmap/vector
    fn clear(&mut self) {
        self.exact_match.clear();
        self.prefix_match.clear();
    }

    // private: insert new api: ToDo: protocol에 사용되는 구조체와 분리 방안
    fn insert(&mut self, mut de_api: DeserializedApi) {
        let m_api = ManagedApi::new(de_api);

        println!("\nAPI.MAP.Insert => {:?}", m_api);
        if m_api.match_prefix {
            self.prefix_match.push(m_api);
        } else {
            let key = m_api.get_key();
            self.exact_match.insert(key, m_api);
        }
    }
}

///! find_api_by_reqline
pub fn find_api_by_reqline(method: &str, uri: &str) -> Option<ManagedApi> {
    let view = get_gloval_view();
    println!(
        "find_api_by_uri: uri={}, view={} (0.left, 1.right)",
        uri, view
    );
    if view == 0 {
        // from LEFT map
        GLOBAL_API_MAP_LEFT.read().unwrap().find(method, uri)
    } else {
        // from RIGHT map
        GLOBAL_API_MAP_RIGHT.read().unwrap().find(method, uri)
    }
}

fn clear_old_map() {
    let view = get_gloval_view();
    if view == 0 {
        // left
        GLOBAL_API_MAP_RIGHT.write().unwrap().clear();
    } else {
        // right
        GLOBAL_API_MAP_LEFT.write().unwrap().clear();
    }
}

fn insert_api_into_new_map(api: DeserializedApi) {
    let view = get_gloval_view();
    if view == 0 {
        // left
        GLOBAL_API_MAP_RIGHT.write().unwrap().insert(api);
    } else {
        // right
        GLOBAL_API_MAP_LEFT.write().unwrap().insert(api);
    }
}

pub fn insert_apis_into_new_map(apis: Vec<DeserializedApi>) {
    // 1. clear
    clear_old_map();

    // 2. update
    for de_api in apis {
        insert_api_into_new_map(de_api);
    }

    // 3. complete: change view
    change_global_view();

    println!("--- global api map chaned --- view: {}", get_gloval_view());
}

/* -------------------------------[for test]--------------------------------- */
///! Be careful: must be called orderly clear -> insert -> complete
///  or use bulk insert function: buik_insert_into_new_map()
pub async fn test_update_apis() {
    let test_api1 = r#"
        {
            "methods": ["GET"],
            "author": "admin",
            "basePath": "/v1/test",
            "targetPath": "/woij123",
            "targetServers": ["https://httpbin.org:443"],
            "authType": "none",
            "cors": false,
            "engineGroups": ["OperatorGroup"],
            "createTime": "2022-02-18 18:15:00",
            "description": "test sample api",
            "_id": "620fe37e770d9e0a60f1a787",
            "name": "jang-test1",
            "version": 1,
            "latestVersion": true,
            "apiKeys": []
        }
        "#;

    let test_api2 = r#"
        {
            "methods": ["GET", "POST"],
            "author": "admin",
            "basePath": "/v2/naver/*",
            "targetPath": "/",
            "targetServers": ["http://www.naver.com:80"],
            "authType": "none",
            "cors": false,
            "engineGroups": ["OperatorGroup"],
            "createTime": "2022-02-18 18:15:00",
            "description": "test sample api",
            "_id": "620fe37e770d9e0a60f1a787",
            "name": "jang-test2",
            "version": 1,
            "latestVersion": true,
            "apiKeys": []
        }
        "#;

    // the way to update api
    let api1 = serde_json::from_str(test_api1).unwrap();
    let api2 = serde_json::from_str(test_api2).unwrap();
    let apis = vec![api1, api2];
    insert_apis_into_new_map(apis);

    // sleep 3 seconds.
    std::thread::sleep(Duration::from_millis(3000));
}

// Todo: remove this after test
pub async fn test_find_apis() {
    // sleep 2 seconds.
    std::thread::sleep(Duration::from_millis(2000));

    println!("===============test find api map ===============");
    let found_api = find_api_by_reqline("GET", "/v1/test");
    match found_api {
        Some(api) => {
            println!("Found! > {:?}", api);
        }
        None => {
            println!("Not found!> /v1/test ");
        }
    };

    let found_api = find_api_by_reqline("POST", "/v2/naver/favicon.ico");
    match found_api {
        Some(api) => {
            println!("Found! > {:?}", api);
        }
        None => {
            println!("Not found!> /v2/naver/favicon.ico ");
        }
    };
}
