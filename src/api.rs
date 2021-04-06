use std::{io, fs};
use std::path::Path;
use std::fs::{create_dir, File};

use crate::common::{Service, Output};
use std::io::Write;
use crate::quake::quake::Quake;

pub struct ApiKey;


impl ApiKey{

    // 初始化API
    pub fn init(api_key: String){
        // write api key
        if Self::set_api(api_key){
            Output::success("Successfully initialized");
        }else {
            Output::error("Error: Invalid API key");
        }
    }

    // 获取API路径
    fn get_path() -> String{
        let  path = dirs::home_dir().unwrap().as_path().to_str().unwrap().to_string();
        path
    }

    // 检测API KEY 是否可用
    fn check_api(apikey:String) -> bool{
        let (local, one_years_ago) = Quake::getdate();
        let s = Service{
            ip_list: Vec::new(),
            query: String::from("port:80"),
            start: 1,
            size: 1,
            ignore_cache: false,
            start_time: one_years_ago,
            end_time: local
        };
        let res = match Quake::new(apikey).search(s) {
            Ok(_) => {
                true
            },
            Err(_) => false,
        };
        res
    }

    // 设置API KEY
    pub fn set_api(apikey: String) -> bool{
        let path = Self::get_path();
        let config_path = path.to_string().clone() + "/.config/";
        let quake_path = path.to_string().clone() + "/.config/quake/";
        let api_path = path.to_string().clone() + "/.config/quake/api_key";
        if !Path::exists(Path::new(config_path.as_str())){
            match create_dir(Path::new(config_path.as_str())){
                Err(e) => {
                    Output::error(&format!("Failed to create path: .config/. {}", e.to_string()));
                    std::process::exit(1);
                }
                _ => {},
            }
        }
        if !Path::exists(Path::new(quake_path.as_str())){
            match create_dir(Path::new(quake_path.as_str())){
                Err(e) => {
                    Output::error(&format!("Failed to create path:.config/quake/. {}", e.to_string()));
                    std::process::exit(1);
                }
                _ => {}
            }
        }
        if Self::check_api(apikey.clone()){
            let mut file: File = match File::create(Path::new(api_path.as_str())){
                Ok(f) => f,
                Err(e) => {
                    Output::error(&format!("File creation failure: {}", e.to_string()));
                    std::process::exit(1);
                }
            };
            return match file.write_all(apikey.as_bytes()){
                Ok(_) => true,
                Err(e) => {
                    Output::error(&format!("File write failure: {}", e.to_string()));
                    std::process::exit(1);
                }
            }
        }
        false
    }

    // 获取API KEY
    pub fn get_api() -> Result<String, io::Error>{
        let res;
        let mut path = Self::get_path();
        path.push_str("/.config/quake/api_key");
        res = fs::read_to_string(path)?;
        Ok(res)
    }
}