/*TODO: Comment*/

pub mod quake {
    // use log::{debug, error, info};
    use reqwest::header::{HeaderMap, HeaderValue};
    use serde_json::Value;
    use crate::common::{Service, Output, Host};
    use crate::api::ApiKey;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::io;
    use chrono::{Local, Duration};
    use regex::{Regex};
    use ansi_term::Colour::Red;

    //BaseUrl is the basis for all of our api requests.
    const BASE_URL: &'static str = "https://quake.360.cn";
    // TODO: Comment
    pub struct Quake{
        api_key: String,
    }

    impl Quake{

        // TODO: Comment
        pub fn new(api_key: String) -> Quake{
            Quake{
                api_key
            }
        }

        // TODO: Comment
        pub fn query_host(query_string:&str, start: i32, size:i32)->Value{
            Output::info(&format!("Search with {}", query_string));
            let res= match ApiKey::get_api(){
                Ok(res) => res,
                Err(e) =>{
                    Output::error(&format!("Failed to read apikey:\t{}", e.to_string()));
                    std::process::exit(1);
                }
            };
            // TODO: Comment
            let h = Host{
                query: String::from(query_string),
                start,
                size,
                ignore_cache: false,
            };
            let response:Value = match Quake::new(res).search_host(&h) {
                Ok(response) => response,
                Err(e) =>{
                    Output::error(&format!("Query failed: {}", e.to_string()));
                    std::process::exit(1);
                }
            };
            response
        }

        // TODO: Comment
        pub fn search_host(&self, host:&Host)->Result<Value, serde_json::Error>{
            let mut url = String::new();
            url.push_str(BASE_URL);
            url.push_str("/api/v3/search/quake_host");
            let client = reqwest::blocking::Client::new();
            let resp = match client.post(&url).headers(self.header()).json(&host).send(){
                Ok(resp) => resp,
                Err(e) =>{
                    if e.is_timeout(){
                        Output::error("Connect Timeout!!");
                    }else {
                        Output::error(&format!("Connect error!!!\r\n{}", e.to_string()));
                    }
                    std::process::exit(1);
                }
            };
            let res = resp.text().unwrap();
            let response:Value = serde_json::from_str(&res)?;
            // TODO: Comment
            let code = response["code"].as_i64().unwrap() as i32;
            let message = response["message"].as_str().unwrap();
            if code != 0{
                Output::error(&format!("Query failed: {}", message));
                std::process::exit(1);
            }
            Ok(response)
        }

        // TODO: Comment
        pub fn query(query_string:&str, start: i32, size:i32) -> Value{
            Output::info(&format!("Search with {}", query_string));
            let res= match ApiKey::get_api(){
                Ok(res) => res,
                Err(e) =>{
                    Output::error(&format!("Failed to read apikey:\t{}", e.to_string()));
                    std::process::exit(1);
                }
            };
            let (local, one_years_ago) = Self::getdate();
            // TODO: Comment
            let s = Service{
                query: String::from(query_string),
                start,
                size,
                ignore_cache: false,
                start_time: one_years_ago,
                end_time: local
            };
            let response:Value = match Quake::new(res).search(&s) {
                Ok(response) => response,
                Err(e) =>{
                    Output::error(&format!("Query failed: {}", e.to_string()));
                    std::process::exit(1);
                }
            };
            response
        }

        // TODO: Comment
        pub fn search(&self, service:&Service) -> Result<Value, serde_json::Error>{
            let mut url = String::new();
            url.push_str(BASE_URL);
            url.push_str("/api/v3/search/quake_service");
            let client = reqwest::blocking::Client::new();
            let resp = match client.post(&url).headers(self.header()).json(&service).send(){
                Ok(resp) => resp,
                Err(e) =>{
                    if e.is_timeout(){
                        Output::error("Connect Timeout!!");
                    }else {
                        Output::error(&format!("Connect error!!!\r\n{}", e.to_string()));
                    }
                    std::process::exit(1);
                }
            };
            let res = resp.text().unwrap();
            let response:Value = serde_json::from_str(&res)?;
            // TODO: Comment
            let code = response["code"].as_i64().unwrap() as i32;
            let message = response["message"].as_str().unwrap();
            if code != 0{
                Output::error(&format!("Query failed: {}", message));
                std::process::exit(1);
            }
            Ok(response)
        }

        // TODO: Comment
        pub fn show(value:Value, showdata:bool, filter:&str, mut data_type: Vec<&str>) -> Vec<String>{
            let count = value["meta"]["pagination"]["count"].as_i64().unwrap() as usize;
            let total =  value["meta"]["pagination"]["total"].as_i64().unwrap() as i32;
            let mut res:Vec<String> = Vec::new();
            Output::success("Successful.");
            Output::success(&format!("count: {} \ttotal: {}", count, total));
            let re = Regex::new(filter).unwrap();
            for i in 0..count{
                let data_value = value["data"][i].as_object().unwrap();
                let title = data_value["service"]["http"]["title"].as_str().unwrap_or("").replace("\"", "");
                let ip = data_value["ip"].as_str().unwrap().replace("\"", "");
                let port = &data_value["port"];
                let country = data_value["location"]["country_cn"].as_str().unwrap_or("");
                let province = data_value["location"]["province_cn"].as_str().unwrap_or("");
                let city = data_value["location"]["city_cn"].as_str().unwrap_or("");
                let owner = data_value["location"]["owner"].as_str().unwrap_or("");
                let mut regex_data = String::new();
                if filter != ""{
                    let cert = data_value["service"]["cert"].as_str().unwrap_or("");
                    let response = data_value["service"]["response"].as_str().unwrap_or("");
                    let http_body = data_value["service"]["http"]["body"].as_str().unwrap_or("");
                    let http_header = data_value["service"]["http"]["response_headers"].as_str().unwrap_or("");
                    regex_data.push_str(cert);
                    regex_data.push_str(response);
                    regex_data.push_str(http_body);
                    regex_data.push_str(http_header);
                }
                let regex_res = match re.find(regex_data.as_str()){
                    Some(res) => res.as_str(),
                    None => ""
                };
                let mut f = String::new();
                for data in data_type.iter_mut(){
                    if data == &"title"{
                        f.push_str(&format!("{}\t", title));
                    }
                    if data == &"ip"{
                        f.push_str(&format!("{}\t", ip));
                    }
                    if data == &"port"{
                        f.push_str(&format!("{}\t", port));
                    }
                    if data == &"country"{
                        f.push_str(&format!("{}\t", country));
                    }
                    if data == &"province"{
                        f.push_str(&format!("{}\t", province));
                    }
                    if data == &"city"{
                        f.push_str(&format!("{}\t", city));
                    }
                    if data == &"owner"{
                        f.push_str(&format!("{}\t", owner));
                    }
                }
                // f.push_str(Red.bold().paint(regex_res).to_string().as_str());
                if showdata{
                    print!("{}", f);
                    println!("{}", Red.bold().paint(regex_res).to_string().as_str());
                }else {
                    f.push_str(regex_res);
                }
                res.push(f);
            }
            res
        }
        // TODO: Comment
        pub fn show_host(value: Value, showdata:bool) -> Vec<String>{
            let mut value = value;
            let mut res:Vec<String> = Vec::new();
            let count = value["meta"]["pagination"]["count"].as_i64().unwrap() as usize;
            let total =  value["meta"]["pagination"]["total"].as_i64().unwrap() as i32;
            Output::success("Successful.");
            Output::success(&format!("count: {} \ttotal: {}", count, total));
            for i in 0..count{
                // ip
                let data = value["data"][i].take();
                let ip = data["ip"].as_str().unwrap().replace("\"", "");
                let service = data["services"].as_array().unwrap();
                let mut info = String::new();
                info.push_str(&format!("IP: {}\n", ip));
                for s in service{
                    info.push_str(&format!("|  {}\t{}\t{}\t{}\n", s["port"], s["name"].as_str().unwrap().replace("\"", ""),
                                          s["product"].as_str().unwrap().replace("\"", ""),
                                          s["version"].as_str().unwrap().replace("\"", "")));
                }
                info.push_str("\n");
                if showdata{
                    println!("{}", info);
                }
                res.push(info);
            }
            res
        }
        // TODO: Comment
        pub fn show_domain(value: Value, onlycount: bool, showdata:bool, mut data_type: Vec<&str>) -> Vec<String>{
            let mut value = value;
            let mut res:Vec<String> = Vec::new();
            let count = value["meta"]["pagination"]["count"].as_i64().unwrap() as usize;
            let total =  value["meta"]["pagination"]["total"].as_i64().unwrap() as i32;
            Output::success("Successful.");
            Output::success(&format!("count: {} \ttotal: {}", count, total));
            if !onlycount{
                for i in 0..count{
                    let data_value = value["data"][i].take();
                    let domain = data_value["service"]["http"]["host"].as_str().unwrap_or("").replace("\"", "");
                    let title = data_value["service"]["http"]["title"].as_str().unwrap_or("").replace("\"", "");
                    let ip = data_value["ip"].as_str().unwrap().replace("\"", "");
                    let port = &data_value["port"];
                    let mut f = String::new();
                    for data in data_type.iter_mut(){
                        if data == &"domain"{
                            f.push_str(&format!("{}\t", domain));
                        }
                        if data == &"title"{
                            f.push_str(&format!("{}\t", title));
                        }
                        if data == &"ip"{
                            f.push_str(&format!("{}\t", ip));
                        }
                        if data == &"port"{
                            f.push_str(&format!("{}\t", port));
                        }
                    }
                    if showdata{
                        println!("{}", f);
                    }
                    res.push(f);
                }
            }else {
                if showdata{
                    println!("{}", total);
                }
            }
            res
        }
        // TODO: Comment
        pub fn save_domain_data(filename: &str, content: Value, data_type:Vec<&str>) ->io::Result<i32>{
            let mut f = OpenOptions::new().create(true).append(true).open(filename)?;
            let domains:Vec<String> = Self::show_domain(content, false, false, data_type);
            let mut count = 0;
            for domain in domains{
                f.write_all(format!("{}\n", domain).as_bytes())?;
                count += 1;
            }
            Ok(count)
        }
        // TODO: COmment
        pub fn save_host_data(filename: &str, content: Value)->io::Result<i32>{
            let mut f = OpenOptions::new().create(true).append(true).open(filename)?;
            let hosts = Self::show_host(content, false);
            let mut count = 0;
            for host in hosts{
                f.write_all(format!("{}\n", host).as_bytes())?;
                count += 1;
            }
            Ok(count)
        }

        // TODO: COmment
        pub fn save_search_data(filename: &str, content: Value, filter:&str, data_type: Vec<&str>)->io::Result<i32>{
            let mut f = OpenOptions::new().create(true).append(true).open(filename)?;
            let hosts = Self::show(content, false, filter, data_type);
            let mut count = 0;
            for host in hosts{
                f.write_all(format!("{}\n", host).as_bytes())?;
                count += 1;
            }
            Ok(count)
        }

        // TODO: Comment
        fn header(&self) -> HeaderMap {
            let mut header = HeaderMap::new();
            header.insert("X-QuakeToken", HeaderValue::from_str(self.api_key.as_str()).unwrap());
            header
        }

        // TODO: Comment
        pub(crate) fn getdate() ->(String, String){
            let local = Local::now();
            let one_years_ago = local - Duration::days(365);
            (local.format("%Y-%m-%d %H:%M:%S"). to_string(), one_years_ago.format("%Y-%m-%d %H:%M:%S").to_string())
        }
    }
}

