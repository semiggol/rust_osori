use std::net::SocketAddr;
use std::env;
use clap::Parser;

#[derive(Parser)]
#[clap(name = "Osori")]
#[clap(author = "webaresoft <webaresoft@webaresoft.com>")]
#[clap(version = "1.0")]
#[clap(about = "Osori is awesome API Gateway", long_about = None)]
struct Args {
    #[clap(short='a', long, name="address (ip:port)", help="set address to connect to admin (ENV: OSORI_ADMIN)", parse(try_from_str=validate_ip_address))]
    admin_address: Option<String>,

    #[clap(short='n', long, name="engine name", help="set engine name for UI (ENV: OSORI_ENGINE)")]
    engine_name: Option<String>,

    #[clap(short='g', long, name="group name", help="set groupname (ENV: OSORI_GROUP)")]
    group_name: Option<String>,

    #[clap(short='t', long, name="seconds", help="set health check timeout of handler")]
    health_check_timeout: Option<usize>,

    #[clap(short='s', long,name="signal", help="send signal to osori: stop", parse(try_from_str=signal_in_rage))]
    signal: Option<String>,

    #[clap(short='v', long, help="show version information")]
    version: bool,

    #[clap(short='V', long, help="set booting mode to verbose")]
    verbose_mode: bool,

    #[clap(short='h', long, help="print help information")]
    help: bool,
}

// check validation of arguments
fn validate_ip_address(s: &str) -> Result<String, String>{
    match s.parse::<SocketAddr>() {
        Ok(_socket) => {
            // can check ip address format and port number range
            return Ok(s.to_string())
        },
        Err(e) =>return Err(e.to_string())
    };
}

fn signal_in_rage(s: &str) -> Result<String, String> {
   let signal_list = ["stop", "reload"];
    if signal_list.iter().any(|&v| v == s) {
        return Ok(s.to_string());
    }

    Err(format!("Invalid signal option"))
}

pub struct SystemConfig {
    pub admin_address: String,
    pub engine_name: Option<String>,
    pub group_name: Option<String>
}

pub fn parse() -> Result<SystemConfig, String>{
    let args = Args::parse();

    if args.verbose_mode {
        // TODO: verbose mode
        println!("Verbose mode is not yet implemented.");
    }

    // get admin server address
    let admin_address = match args.admin_address {
        Some(address) => address,
        None => {
            if let Ok(address) = env::var("OSORI_ADMIN") {
                if let Err(_e) = validate_ip_address(address.as_str()) {
                    return Err(String::from("Address of admin server is not valid (ex. 127.0.0.1:5581). check ENV OSORI_ADMIN"));
                }
                address
            }
            else {
                return Err(String::from("Address of admin server is required (ex. 127.0.0.1:5581). use ENV OSORI_ADMIN or -a option"));
            }
        }
    };

    // get engine name
    let engine_name = match args.engine_name{
        Some(name) => Some(name),
        None => {
            if let Ok(name) = env::var("OSORI_ENGINE") {
               Some(name)
            }
            else {
                None
            }
        }
    };

    // get group name
    let group_name = match args.group_name{
        Some(name) => Some(name),
        None => {
            if let Ok(name) = env::var("OSORI_GROUP") {
                Some(name)
            }
            else {
                None
            }
        }
    };

    Ok(SystemConfig{admin_address, engine_name, group_name})
}