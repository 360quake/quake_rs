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
        let  mut path = dirs::home_dir().unwrap().as_path().to_str().unwrap().to_string();
        path.push_str("/.config/quake/");
        path
    }

    // 检测API KEY 是否可用
    fn check_api(apikey:String) -> bool{
        let (local, one_years_ago) = Quake::getdate();
        let s = Service{
            query: String::from("ip:192.185.144.54"),
            start: 1,
            size: 1,
            ignore_cache: false,
            start_time: one_years_ago,
            end_time: local
        };
        let res = match Quake::new(apikey).search(&s) {
            Ok(_) => {
                true
            },
            Err(_) => false,
        };
        res
    }

    // 设置API KEY
    pub fn set_api(apikey: String) -> bool{
        let path  = Self::get_path();
        let path = Path::new(path.as_str());
        if  !Path::exists(path){
            create_dir(path).unwrap();
        }
        let mut  path = Self::get_path();
        path.push_str("api_key");
        if Self::check_api(apikey.clone()){
            let path = Path::new(path.as_str());
            let mut file: File = match File::create(path){
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
        path.push_str("api_key");
        res = fs::read_to_string(path)?;
        Ok(res)
    }
}