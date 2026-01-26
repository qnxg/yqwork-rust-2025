#[derive(serde::Deserialize, Debug)]
pub struct Configs {
    pub server: Server,
    pub database: Database,
    pub jwt: Jwt,
    pub log: Log,
    pub weihuda: Weihuda,
}

#[derive(serde::Deserialize, Debug)]
pub struct Server {
    pub address: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Database {
    pub max_connections: u32,
    pub database_url: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Jwt {
    pub secret: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Log {
    pub filter_level: String,
    pub with_ansi: bool,
    pub to_stdout: bool,
    pub directory: String,
    pub file_name: String,
    pub rolling: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Weihuda {
    pub api_url: String,
}

pub static CFG: once_cell::sync::Lazy<Configs> = once_cell::sync::Lazy::new(|| {
    let s = std::fs::read_to_string("config/config.toml").expect("读取配置文件失败");
    toml::from_str(&s).expect("解析配置文件失败")
});
