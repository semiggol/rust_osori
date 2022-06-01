use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::RwLock;
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};

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

#[derive(Debug, Clone)]
pub struct Api {
    pub name: String,
    pub version: u32,
    pub methods: Vec<hyper::Method>,
    pub match_type: MatchType,
    pub base_path: String,
    pub target_path: String,
    pub target_servers: Vec<String>,
}
/// ToDo: Api Builder? change this to something with the json data
impl Api {
    pub fn new (
        name: String,
        version: u32,
        methods: Vec<hyper::Method>,
        base_path: String,
        target_path: String,
        target_servers: Vec<String>) -> Self {

        let len = base_path.len();
        let path = base_path.as_bytes();
        let mut match_type = MatchType::Exact;
        if path[len-1] == b'*' {
            match_type = MatchType::Prefix;
        }

        Api {
            name,
            version,
            methods,
            match_type,
            base_path,
            target_path,
            target_servers,
        }
    }

    // key = "/v1/base_path"
    pub fn get_key(&self) -> String {
        let mut key = String::from("/v");
        key.push_str(&self.version.to_string()[..]);
        let slash = self.base_path.as_bytes()[0];
        if slash != b'/' {
            key.push('/');
        }
        key.push_str(&self.base_path[..]);
        key
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

    // insert new api
    pub fn insert(&mut self, api: Api) {
        match api.match_type {
            MatchType::Exact => {
                let key = api.get_key();
                self.exact_match.insert(key, api);
            },
            MatchType::Prefix => self.prefix_match.push(api)
        }
    }

    // find api
    pub fn find(&self, method: hyper::Method, uri: &str) -> Option<Api> {
        // 1. get from exact map
        match self.exact_match.get(uri) {
            Some(api) => {
                // found
                for api_method in api.methods.iter() {
                    if api_method == method {
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
            let len = api.base_path.len();
            let prefix_path = &api.base_path[0..len-1];
            if uri.contains(prefix_path) {
                // found
                for api_method in api.methods.iter() {
                    if api_method == method {
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
pub fn find_api_by_reqline(method: hyper::Method, uri: &str) -> Option<Api> {
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
}

/* -------------------------------[for test]--------------------------------- */
///! Be careful: must be called orderly clear -> insert -> complete
///  or use bulk insert function: buik_insert_into_new_map()
pub async fn test_update_apis() {
    let mut index = 1;
    loop {
        println!("\n{}=============== test update api map1-1 ===============", index);
//1. how to use update api
/* 
        // 1. clear
        clear_old_map();

        // 2. update
        let api1 = make_sample_api1();
        let api2 = make_sample_api2();
        insert_into_new_map(api1);
        insert_into_new_map(api2);

        // 3. complete: change view
        change_global_view();
*/     
//2. how to use update api
        let apis = vec![make_sample_api1(), make_sample_api2()];
        bulk_insert_into_new_map(apis);

        // sleep 3 seconds.
        std::thread::sleep(Duration::from_millis(3000));

        let apis = vec![];
        bulk_insert_into_new_map(apis);

        // sleep 3 seconds.
        std::thread::sleep(Duration::from_millis(3000));
        index +=1;
    }
}

// Todo: remove this after test
pub async fn test_find_apis() {
    loop {
        println!("===============test find api map ===============");
        let found_api = find_api_by_reqline(hyper::Method::GET, "/v1/test");
        match found_api {
            Some(api) => {
                println!("Found! > {:?}", api);
            },
            None => {
                println!("Not found!> /v1/test ");
            }
        };

        let found_api = find_api_by_reqline(hyper::Method::POST, "/v2/naver/favicon.ico");
        match found_api {
            Some(api) => {
                println!("Found! > {:?}", api);
            },
            None => {
                println!("Not found!> /v2/naver/favicon.ico ");
            }
        };

        // sleep 2 seconds.
        std::thread::sleep(Duration::from_millis(2000));
    }
}

fn make_sample_api1() -> Api {
    let name = String::from("test_api1");
    let version: u32 = 1;
    let methods = vec![hyper::Method::GET];
    let base_path = String::from("/test");
    let target_path = String::from("/target");
    let target_servers = vec![String::from("https://httpbin.com")];
    Api::new(name, version, methods, base_path, target_path, target_servers)
}

// remove asterisk when making?
fn make_sample_api2() -> Api {
    let name = String::from("test_api2");
    let version: u32 = 2;
    let methods = vec![hyper::Method::GET, hyper::Method::POST];
    let base_path = String::from("/naver/*");
    let target_path = String::from("/*");
    let target_servers = vec![String::from("https://www.naver.com")];
    Api::new(name, version, methods, base_path, target_path, target_servers)
}