// 引入文件系统操作相关模块
use std::fs::{create_dir, File};
// 引入路径操作模块
use std::path::Path;
// 引入文件系统和输入输出相关模块
use std::{fs, io};

// 引入公共模块中的输出工具和服务结构体
use crate::common::{Output, Service};
// 引入 Quake 模块中的 Quake 结构体
use crate::quake::quake::Quake;
// 引入写入操作的 trait
use std::io::Write;

/// ApiKey 结构体，用于管理 API 密钥和 GPT API 密钥的初始化、设置和获取操作
pub struct ApiKey;

impl ApiKey {
    /// 初始化 Quake API 密钥
    ///
    /// # 参数
    /// - `api_key`: 要设置的 Quake API 密钥
    pub fn init(api_key: String) {
        // 写入 API 密钥
        if Self::set_api(api_key) {
            Output::success("成功初始化");
        } else {
            Output::error("错误: 无效的 API 密钥");
        }
    }

    /// 初始化 GPT API 密钥
    ///
    /// # 参数
    /// - `gpt_key`: 要设置的 GPT API 密钥
    pub fn gptinit(gpt_key: String) {
        // 写入 API 密钥
        if Self::set_gptapi(gpt_key) {
            Output::success("成功初始化");
        } else {
            Output::error("错误: 无效的 API 密钥");
        }
    }

    /// 获取用户主目录路径
    ///
    /// # 返回值
    /// 用户主目录的路径字符串
    fn get_path() -> String {
        let path = dirs::home_dir()
            .unwrap()
            .as_path()
            .to_str()
            .unwrap()
            .to_string();
        path
    }

    /// 检查 Quake API 密钥是否可用
    ///
    /// # 参数
    /// - `apikey`: 要检查的 Quake API 密钥
    ///
    /// # 返回值
    /// 如果密钥可用返回 `true`，否则返回 `false`
    fn check_api(apikey: String) -> bool {
        let (local, one_years_ago) = Quake::getdate();
        let s = Service {
            ip_list: Vec::new(),
            query: String::from("port:80"),
            start: 1,
            size: 1,
            ignore_cache: false,
            latest: false,
            start_time: one_years_ago,
            end_time: local,
            shortcuts: Vec::new(),
        };
        let res = match Quake::new(apikey).search(s) {
            Ok(_) => true,
            Err(_) => false,
        };
        res
    }

    /// 检查 GPT API 密钥是否可用
    ///
    /// # 参数
    /// - `apikey`: 要检查的 GPT API 密钥
    ///
    /// # 返回值
    /// 目前总是返回 `true`，可根据实际需求实现检查逻辑
    fn check_gptapi(apikey: String) -> bool {
        true
    }

    /// 创建配置目录
    ///
    /// # 参数
    /// - `path`: 要创建的目录路径
    fn create_config_dir(path: &str) {
        if !Path::exists(Path::new(path)) {
            match create_dir(Path::new(path)) {
                Err(e) => {
                    Output::error(&format!("创建路径失败: {}. {}", path, e.to_string()));
                    std::process::exit(1);
                }
                _ => {}
            }
        }
    }

    /// 设置 Quake API 密钥
    ///
    /// # 参数
    /// - `apikey`: 要设置的 Quake API 密钥
    ///
    /// # 返回值
    /// 如果密钥设置成功返回 `true`，否则返回 `false`
    pub fn set_api(apikey: String) -> bool {
        let path = Self::get_path();
        let config_path = format!("{}/.config/", path);
        let quake_path = format!("{}/.config/quake/", path);
        let api_path = format!("{}/.config/quake/api_key", path);

        Self::create_config_dir(&config_path);
        Self::create_config_dir(&quake_path);

        if Self::check_api(apikey.clone()) {
            let mut file: File = match File::create(Path::new(&api_path)) {
                Ok(f) => f,
                Err(e) => {
                    Output::error(&format!("文件创建失败: {}", e.to_string()));
                    std::process::exit(1);
                }
            };
            return match file.write_all(apikey.as_bytes()) {
                Ok(_) => true,
                Err(e) => {
                    Output::error(&format!("文件写入失败: {}", e.to_string()));
                    std::process::exit(1);
                }
            };
        }
        false
    }

    /// 获取 Quake API 密钥
    ///
    /// # 返回值
    /// 包含 Quake API 密钥的 `Result` 类型，可能包含错误信息
    pub fn get_api() -> Result<String, io::Error> {
        let path = format!("{}/.config/quake/api_key", Self::get_path());
        fs::read_to_string(path)
    }

    /// 设置 GPT API 密钥
    ///
    /// # 参数
    /// - `apikey`: 要设置的 GPT API 密钥
    ///
    /// # 返回值
    /// 如果密钥设置成功返回 `true`，否则返回 `false`
    pub fn set_gptapi(apikey: String) -> bool {
        let path = Self::get_path();
        let config_path = format!("{}/.config/", path);
        let quake_path = format!("{}/.config/quake/", path);
        let api_path = format!("{}/.config/quake/gptapi_key", path);

        Self::create_config_dir(&config_path);
        Self::create_config_dir(&quake_path);

        if Self::check_gptapi(apikey.clone()) {
            let mut file: File = match File::create(Path::new(&api_path)) {
                Ok(f) => f,
                Err(e) => {
                    Output::error(&format!("文件创建失败: {}", e.to_string()));
                    std::process::exit(1);
                }
            };
            return match file.write_all(apikey.as_bytes()) {
                Ok(_) => true,
                Err(e) => {
                    Output::error(&format!("文件写入失败: {}", e.to_string()));
                    std::process::exit(1);
                }
            };
        }
        false
    }

    /// 获取 GPT API 密钥
    ///
    /// # 返回值
    /// 包含 GPT API 密钥的 `Result` 类型，可能包含错误信息
    pub fn get_gptapi() -> Result<String, io::Error> {
        let path = format!("{}/.config/quake/gptapi_key", Self::get_path());
        fs::read_to_string(path)
    }
}
