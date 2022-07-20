
/// 配置
use std::net::SocketAddr;
use std::path::PathBuf;
use std::fs;
use std::env;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "config/config.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub port: u16,
    pub server: Vec<String>,
    pub preload_domain_count: u32,
    pub cache_max_size: u32,
    pub cache_max_ttl: u32,
    pub preload_domain_file: String,
}

/// read configuration from json.
macro_rules! read_config { 
    ($struct: ident) => ({ 
        let current_dir = match env::current_dir(){
            Ok(path) => path,
            Err(_err) => PathBuf::from("."),
        };
        let config_path = current_dir.join(CONFIG_FILE).into_os_string();
        let config_str = match fs::read_to_string(&config_path) {
            Ok(str) => str,
            Err(err) => panic!("Fail to read config file(:{:?}), error:{}", config_path, err),
        };
        match serde_json::from_str::<$struct>(&config_str.as_str()){
            Ok(result) => result,
            Err(err) => panic!("Fail to parse config, error:{}", err),
        }
    })
}

macro_rules! parse_upstream_dns_servers {
    ($config:ident) => ({
        let mut server_addrs:Vec<SocketAddr> =Vec::new();
        let item_list = $config.server.to_vec();
        for item in item_list {
            let addr_str = match item.rfind(":"){
                Some(_) => item.to_string(),
                None => format!("{}:53", item),
            };
            if let Ok(addr) = addr_str.parse::<SocketAddr>(){
                if addr.is_ipv4(){
                    server_addrs.push(addr);
                }
            }

        }
        server_addrs
    });
}

lazy_static! { 
    pub static ref GLOBE_CONFIG:Config = read_config!(Config);
    pub static ref UPSTREAM_DNS_SERVER_ADDR:Vec<SocketAddr> = parse_upstream_dns_servers!(GLOBE_CONFIG);
}


/// 获取服务端绑定地址
pub fn get_bind_addr() -> SocketAddr {
    format!("0.0.0.0:{:?}", GLOBE_CONFIG.port).parse::<SocketAddr>().expect("Server port configuration error.")
}

/// 获取最大缓存时长
pub fn get_cache_max_ttl() -> u32 {
    GLOBE_CONFIG.cache_max_ttl
}

/// 获取缓存最大值
pub fn get_cache_max_size() -> u32 {
    GLOBE_CONFIG.cache_max_size
}

/// 获取需要预加载的域名文件
pub fn get_preloader_filename() -> String {
    GLOBE_CONFIG.preload_domain_file.to_string()
}

/// 获取上游服务器DNS地址列表
pub fn get_upstream_dns_addrs() -> Vec<SocketAddr> {
    UPSTREAM_DNS_SERVER_ADDR.clone()
}

/// 预加载的域名数量
pub fn get_preload_domain_count() -> u32 {
    GLOBE_CONFIG.preload_domain_count
}