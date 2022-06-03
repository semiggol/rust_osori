use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::RwLock;
use std::time::Duration;
use std::sync::atomic::{ AtomicUsize, Ordering };
use serde::{ Serialize, Deserialize };

// for global api map
lazy_static! {
    static ref GLOBAL_API_VIEW: AtomicUsize = AtomicUsize::new(0);
    static ref GLOBAL_API_MAP_LEFT: RwLock<Map> = RwLock::new(Map::new());
    static ref GLOBAL_API_MAP_RIGHT: RwLock<Map> = RwLock::new(Map::new());
}

///! atomic operation for gloval view/map
fn get_gloval_view() -> usize {
    GLOBAL_API_VIEW.load(Ordering::SeqCst) & 1
}

fn change_global_view() {
    GLOBAL_API_VIEW.fetch_add(1, Ordering::SeqCst);
}

#[derive(Debug, Clone)]
pub enum MatchType {
    Exact,
    Prefix,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    #[serde(skip)]
    pub match_prefix: bool,
}

impl Api {
    pub fn get_key(&self) -> String {
        self.base_path.clone()
    }

    // ToDo: match_prefix 변수에 대한 고민 필요
    pub fn fix_matchtype_and_remove_asterisk(&mut self) {
        // fix match type
        let mut len = self.base_path.len();
        let mut path = self.base_path.as_bytes();
        if path[len-1] == b'*' {
            // set 
            self.match_prefix = true;
            // remove '*' in base_path
            self.base_path = self.base_path[0..len-1].to_string();
            
            // ToDo: how to process '*'?
            len = self.target_path.len();
            path = self.target_path.as_bytes();
            if path[len-1] == b'*' {
                // remove '*' in target_path
                self.target_path = self.target_path[0..len-1].to_string();
            }
        } else {
            self.match_prefix = false;
        }
    }
}


#[derive(Debug, Clone)]
pub struct Map {
    exact_match: HashMap<String, Api>,
    prefix_match: Vec<Api>,
}

impl Map {
    pub fn new() -> Self {
        let exact_match: HashMap<String, Api> = HashMap::new();
        let prefix_match: Vec<Api> = Vec::new();
        Map {
            exact_match,
            prefix_match,
        }
    }
    
    // clear hashmap/vector
    pub fn clear(&mut self) {
        self.exact_match.clear();
        self.prefix_match.clear();
    }

    // insert new api: ToDo: protocol에 사용되는 구조체와 분리 방안
    pub fn insert(&mut self, mut api: Api) {
        api.fix_matchtype_and_remove_asterisk();
        
        println!("\nAPI.MAP.Insert => {:?}", api);
        if api.match_prefix {
            self.prefix_match.push(api);
        } else {
            let key = api.get_key();
            self.exact_match.insert(key, api);
        }
    }

    // find api
    pub fn find(&self, method: &str, uri: &str) -> Option<Api> {
        // 1. get from exact map
        match self.exact_match.get(uri) {
            Some(api) => {
                // found
                for api_method in api.methods.iter() {
                    if method.eq(api_method) {
                        // ToDo: need to deep copy?
                        return Some(api.clone());
                    }
                }
            },
            None => {},
        };

        // 2. get from prefix map .. ToDo: remove '*' in base_path?
        for api in &self.prefix_match {
            // remove '*'
            //ToDo: use eq() insted contains() // jang 
            let len = api.base_path.len();
            let prefix_path = &api.base_path[0..len-1];
            //let prefix_uri = uri..();
            if uri.contains(prefix_path) {
                // found
                for api_method in api.methods.iter() {
                    if method.eq(api_method) {
                        // ToDo: need to deep copy?
                        return Some(api.clone());
                    }
                }
            }
        }

        // Not Found
        None
    }
}

///! find_api_by_reqline
pub fn find_api_by_reqline(method: &str, uri: &str) -> Option<Api> {
    let view = get_gloval_view();
    println!("find_api_by_uri: uri={}, view={} (0.left, 1.right)", uri, view);
    if view == 0 { // from LEFT map
        GLOBAL_API_MAP_LEFT.read().unwrap().find(method, uri)
    } else { // from RIGHT map
        GLOBAL_API_MAP_RIGHT.read().unwrap().find(method, uri)
    }
}

fn clear_old_map() {
    let view = get_gloval_view();
    if view == 0 { // left
        GLOBAL_API_MAP_RIGHT.write().unwrap().clear();
    } else {       // right
        GLOBAL_API_MAP_LEFT.write().unwrap().clear();
    }
}

fn insert_into_new_map(api: Api) {
    let view = get_gloval_view();
    if view == 0 { // left
        GLOBAL_API_MAP_RIGHT.write().unwrap().insert(api);
    } else {       // right
        GLOBAL_API_MAP_LEFT.write().unwrap().insert(api);
    }
}

pub fn bulk_insert_into_new_map(apis: Vec<Api>) {
    // 1. clear
    clear_old_map();

    // 2. update
    for api in apis {
        insert_into_new_map(api);
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
    bulk_insert_into_new_map(apis);

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
        },
        None => {
            println!("Not found!> /v1/test ");
        }
    };

    let found_api = find_api_by_reqline("POST", "/v2/naver/favicon.ico");
    match found_api {
        Some(api) => {
            println!("Found! > {:?}", api);
        },
        None => {
            println!("Not found!> /v2/naver/favicon.ico ");
        }
    };
}
