// for test
#[derive(Debug, Clone)]
pub struct Apis {
    pub name: String,
    pub version: u32,
    pub methods: Vec<hyper::Method>,
    pub base_path: String,
    pub target_path: String,
    pub target_servers: Vec<String,>,
}

/// ToDo: Api Builder?
impl Apis {
    pub fn new (
        name: String,
        version: u32,
        methods: Vec<hyper::Method>,
        base_path: String,
        target_path: String,
        target_servers: Vec<String>) -> Self {
        Apis { 
            name,
            version,
            methods,
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

        println!("sample api key ={}", key);
        key
    }
}

pub fn make_sample_api1() -> Apis {
    let name = String::from("test_api1");
    let version: u32 = 1;
    let methods = vec![hyper::Method::GET, hyper::Method::POST];
    let base_path = String::from("test");
    let target_path = String::from("/");
    let target_servers = vec![String::from("https://httpbin.com")];

    Apis::new(name, version, methods, base_path, target_path, target_servers)
}

pub fn make_sample_api2() -> Apis {
    let name = String::from("test_api2");
    let version: u32 = 2;
    let methods = vec![hyper::Method::GET, hyper::Method::HEAD];
    let base_path = String::from("/google");
    let target_path = String::from("/");
    let target_servers = vec![String::from("https://google.com")];

    Apis::new(name, version, methods, base_path, target_path, target_servers)
}