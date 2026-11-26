// use std::process::Command;
use crate::api::ApiKey;
use crate::gpt::Gpt;
use crate::quake::quake::Quake;
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use clap::{Arg, Command};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
/*
  TODO: Comment
*/
// 派生序列化、反序列化和调试输出的 trait
#[derive(Serialize, Deserialize, Debug)]
/// 表示服务查询的结构体，包含查询所需的各种参数
pub struct Service {
    /// 查询语句，用于指定查询的条件
    pub query: String,
    /// 查询的起始位置，用于分页查询
    pub start: i32,
    /// 查询结果的数量，即每页返回的记录数
    pub size: i32,
    /// 是否忽略缓存，true 表示忽略缓存，直接从数据源查询
    pub ignore_cache: bool,
    /// 是否只查询最新数据，true 表示只返回最新的数据
    pub latest: bool,
    /// 查询的开始时间，格式通常为特定的时间字符串
    pub start_time: String,
    /// 查询的结束时间，格式通常为特定的时间字符串
    pub end_time: String,
    /// IP 地址列表，用于筛选特定 IP 相关的数据
    pub ip_list: Vec<Value>,
    /// 快捷方式列表，可能包含一些预设的查询条件或配置
    pub shortcuts: Vec<Value>,
}

/*
  TODO: Comment
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Scroll {
    pub query: String,
    pub size: i32,
    pub ignore_cache: bool,
    pub latest: bool,
    pub pagination_id: String,
    pub start_time: String,
    pub end_time: String,
    pub ip_list: Vec<Value>,
    pub shortcuts: Vec<Value>,
}

/*
  TODO: Comment
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Host {
    pub query: String,
    pub start: i32,
    pub size: i32,
    pub ignore_cache: bool,
}

/*
  TODO: Comment
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct ScrollHost {
    pub query: String,
    pub size: i32,
    pub pagination_id: String,
    pub ignore_cache: bool,
}

/*
  TODO: Comment
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct AggService {
    pub query: String,
    pub start: i32,
    pub size: i32,
    pub ignore_cache: bool,
    pub aggregation_list: Vec<String>,
}

pub struct ArgParse;

impl ArgParse {
    pub fn parse() {
        let matches = Command::new("Quake Command-Line Application")
            .version("3.1.7")
            .author("Author: 360 Quake Team  <quake@360.cn>")
            .about("Dose awesome things.")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(
                Command::new("init")
                    .about("Initialize the Quake command-line")
                    .arg(
                        Arg::new("Api_Key")
                            .index(1)
                            .help("Initialize the Quake command-line")
                    )
            ).subcommand(
                Command::new("gptinit")
                    .about("Initialize the gtpapi")
                    .arg(
                        Arg::new("gpt_Key")
                            .help("Initialize the gptapi")
                    )
            )
            .subcommand(
                Command::new("info")
                    .about("Shows general information about your account")
            )
            .subcommand(
                Command::new("host")
                    .about("View all available information for an IP address")
                    .arg(
                        Arg::new("ip")
                            .index(1)
                            .help(" View all available information for an IP address")
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("Save the host information in the given file (append if file exists).")
                            .value_name("FILENAME")
                    )
                    .arg(
                        Arg::new("query_host_file")
                            .short('q')
                            .long("query_host_file")
                            .help("Quake Host file(Only support --size); Example: quake search -q hosts.txt")
                            .value_name("FILENAME")
                    )
                    .arg(
                        Arg::new("size")
                            .long("size")
                            .value_name("NUMBER")
                            .help("The size of the number of responses, up to a maximum of 100 (Default 10).")
                    )
                    .arg(
                        Arg::new("start")
                            .long("start")
                            .value_name("NUMBER")
                            .help("Starting position of the query (Default 0).")
                    )
                    .arg(
                        Arg::new("type")
                            .short('t')
                            .long("type")
                            .value_name("TYPE")
                            .help("Fields displayed:ip,port,title,country,province,city,owner,time,ssldomain. (Default ip,port)")
                    )
            )
            .subcommand(
                Command::new("search")
                    .about("Search the Quake database")
                    .arg(
                        Arg::new("query_string")
                            .index(1)
                            .help("Quake Querystring; Example: quake search 'port:80'")
                    )
                    .arg(
                        Arg::new("query_file")
                            .short('q')
                            .long("query_file")
                            .help("Quake Querystring file; Example: quake search -q test.txt")
                            .value_name("FILENAME")
                    )
                    .arg(
                        Arg::new("time_start")
                            .short('s')
                            .long("start_time")
                            .help("Search start time\r\n\
                            Example: quake search 'port:80' -s 2020-01-01")
                            .value_name("TIME START")
                    )
                    .arg(
                        Arg::new("time_end")
                            .short('e')
                            .long("end_time")
                            .help("Search end time\r\n\
                            Example: quake search 'port:80' -e 2020-01-01")
                            .value_name("TIME END")
                    )
                    .arg(
                        Arg::new("upload")
                            .short('u')
                            .long("upload")
                            .help("Uploading *.txt files containing only IP addresses, with no more than 1000 IPs.\r\n\
                            Example: quake search -u ips.txt")
                            .value_name("IP File")
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("Save the host information in the given file (append if file exists).")
                            .value_name("FILENAME")
                    )
                    .arg(
                        Arg::new("size")
                            .long("size")
                            .value_name("NUMBER")
                            .help("The size of the number of responses, up to a maximum of 100 (Default 10).")
                    )
                    .arg(
                        Arg::new("start")
                            .long("start")
                            .value_name("NUMBER")
                            .help("Starting position of the query (Default 0).")
                    )
                    .arg(
                        Arg::new("type")
                            .short('t')
                            .long("type")
                            .value_name("TYPE")
                            .help("Fields displayed:ip,port,title,product_name_cn,version,protocol,country,province,city,owner,time,ssldomain,domain. (Default ip,port)")
                    )
                    .arg(
                        Arg::new("filter")
                            .short('f')
                            .long("filter")
                            .value_name("TYPE")
                            .help("Filter search results with more regular expressions.\r\n\
                            Example: quake search 'app:\"exchange 2010\"' -t ip,port,title -f \"X-OWA-Version: (.*)\"")
                    )
                    .arg(Arg::new("cdn")
                        .short('c')
                        .long("cdn")
                        .value_name("NUMBER")
                        .help("Exclude cdn data when parameter is 1,Not excluded by default"))
                    .arg(Arg::new("honey_jar")
                        .short('m')
                        .long("honey_jar")
                        .value_name("NUMBER")
                        .help("Exclude honey_jar data when parameter is 1,Not excluded by default"))
                    .arg(Arg::new("latest_data")
                        .short('l')
                        .long("latest_data")
                        .value_name("NUMBER")
                        .help("Display latest data when parameter is 1,Not up to date by default"))
                    .arg(Arg::new("filter_request")
                        .short('r')
                        .long("filter_request")
                        .value_name("NUMBER")
                        .help("When the parameter is 1, invalid requests are filtered, such as 400, 401, 403 and other request data, the default is not filtered"))
                    .arg(Arg::new("deduplication")
                        .short('d')
                        .long("deduplication")
                        .value_name("NUMBER")
                        .help("When the parameter is 1, data deduplication is performed, and no deduplication is performed by default."))
            )
            .subcommand(
                Command::new("domain")
                    .about("View all available information for a domain.")
                    .arg(Arg::new("cdn")
                        .short('n')
                        .long("cdn")
                        .value_name("NUMBER")
                        .help("Exclude cdn data when parameter is 1,Not excluded by default"))
                    .arg(Arg::new("honey_jar")
                        .short('m')
                        .long("honey_jar")
                        .value_name("NUMBER")
                        .help("Exclude honey_jar data when parameter is 1,Not excluded by default"))
                    .arg(Arg::new("latest_data")
                        .short('l')
                        .long("latest_data")
                        .value_name("NUMBER")
                        .help("Display latest data when parameter is 1,Not up to date by default"))

                    .arg(
                        Arg::new("domain_name")
                            .index(1)
                            .value_name("DOMAIN_NAME")
                            .help("The domain name to be queried.")
                    )
                    .arg(
                        Arg::new("count")
                            .short('c')
                            .long("count")
                            .value_name("NUMBER")
                            .help("Count of results")
                    )
                    .arg(
                        Arg::new("size")
                            .long("size")
                            .value_name("NUMBER")
                            .help("The size of the number of responses, up to a maximum of 100 (Default 10).")
                    )
                    .arg(
                        Arg::new("start")
                            .long("start")
                            .value_name("NUMBER")
                            .help("Starting position of the query (Default 0).")
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .value_name("FILENAME")
                            .help("Output result to file.")
                    )
                    .arg(
                        Arg::new("type")
                            .short('t')
                            .long("type")
                            .value_name("TYPE")
                            .help("Fields displayed:domain,ip,port,title. (Default domain, ip, port)")
                    )
                    .arg(Arg::new("filter_request")
                        .short('r')
                        .long("filter_request")
                        .value_name("NUMBER")
                        .help("When the parameter is 1, invalid requests are filtered, such as 400, 401, 403 and other request data, the default is not filtered"))
                    .arg(Arg::new("deduplication")
                        .short('d')
                        .long("deduplication")
                        .value_name("NUMBER")
                        .help("When the parameter is 1, data deduplication is performed, and no deduplication is performed by default."))
            )
            .subcommand(
                Command::new("honeypot")
                    .about("Check whether the IP is a honeypot or not.")
                    .arg(
                        Arg::new("ip")
                            .index(1)
                            .value_name("ip")
                            .help("The ip address to be queried.")
                    )
            ) .subcommand(
                Command::new("gpt")
                    .about("Artificial intelligence engine, directly say what you want to check without grammar")
                    .arg(
                        Arg::new("gpt_match")
                            .index(1)
                            .value_name("GPT_MATVH")
                            .help("what to say")
                    ).arg(
                        Arg::new("size")
                            .long("size")
                            .value_name("NUMBER")
                            .help("The size of the number of responses, up to a maximum of 100 (Default 10).")
                    )
            )
            .get_matches();

        match matches.subcommand() {
            Some(("init", init_match)) => {
                if let Some(api_key) = init_match.get_many::<String>("Api_Key") {
                    ApiKey::init(api_key.map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
                }
            }
            Some(("gptinit", init_match)) => {
                if let Some(api_key) = init_match.get_many::<String>("gpt_Key") {
                    ApiKey::gptinit(api_key.map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
                }
            }
            Some(("domain", domain_match)) => {
                let domain = match domain_match.get_many::<String>("domain_name") {
                    Some(domain) => domain.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    None => {
                        Output::error(
                            "Error: You must choose a domain name.\r\nPlease execute -h for help.",
                        );
                        std::process::exit(1);
                    }
                };
                let start = match domain_match.get_many::<String>("start") {
                    Some(start) => start
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let size = match domain_match.get_many::<String>("size") {
                    Some(size) => size
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 10,
                };
                if size > 100 {
                    Output::warning("Warning: Size is set to a maximum of 100, if set too high it may cause abnormal slowdowns or timeouts.");
                }
                let query = &format!("domain:*.{}", domain);
                let data_type_str = match domain_match.get_many::<String>("type") {
                    Some(data_type_str) => data_type_str
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => "ip,port,domain".to_string(),
                };
                let data_type: Vec<&str> = data_type_str.split(',').collect();

                let cdn = match domain_match.get_many::<String>("cdn") {
                    Some(cdn) => cdn
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let mg = match domain_match.get_many::<String>("honey_jar") {
                    Some(mg) => mg
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let zxsj = match domain_match.get_many::<String>("latest_data") {
                    Some(zxsj) => zxsj
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let wxqq = match domain_match.get_many::<String>("filter_request") {
                    Some(wxqq) => wxqq
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let sjqc = match domain_match.get_many::<String>("deduplication") {
                    Some(sjqc) => sjqc
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let response =
                    Quake::query(query, "", start, size, "", "", cdn, mg, zxsj, wxqq, sjqc);

                let count = match domain_match.get_many::<String>("count") {
                    Some(count) => count
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let mut onlycount = false;
                if count > 0 {
                    onlycount = false;
                }
                let output = match domain_match.get_many::<String>("output") {
                    Some(output) => output.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    None => {
                        Quake::show_domain(response, onlycount, true, data_type);
                        Quake::show_info_jf();
                        std::process::exit(0);
                    }
                };
                let filename = &output;
                // save to file.
                match Quake::save_domain_data(filename, response, data_type) {
                    Ok(count) => {
                        Output::success(&format!(
                            "Successfully saved {} pieces of data to {}",
                            count, output
                        ));
                    }
                    Err(e) => {
                        Output::error(&format!("Data saving failure:{}", e.to_string()));
                    }
                };
            }
            Some(("host", host_match)) => {
                let query_host_file = match host_match.get_many::<String>("query_host_file") {
                    Some(query_host_file) => query_host_file
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    None => "".to_string(),
                };
                let query_host_file = &query_host_file;
                let start = match host_match.get_many::<String>("start") {
                    Some(start) => start
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let size = match host_match.get_many::<String>("size") {
                    Some(size) => size
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 10,
                };
                if size > 100 {
                    Output::warning("Warning: Size is set to a maximum of 100, if set too high it may cause abnormal slowdowns or timeouts.");
                }
                if query_host_file == "" {
                    let ip = match host_match.get_many::<String>("ip") {
                        Some(ip) => ip.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                        None => {
                            Output::error(
                                "Error: You must choose a ip or cidr.\r\nPlease execute -h for help.",
                            );
                            std::process::exit(1);
                        }
                    };
                    let query = &format!("ip:{}", ip);
                    let response = Quake::query_host(query, start, size);
                    let output = match host_match.get_many::<String>("output") {
                        Some(name) => name.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                        None => {
                            Quake::show_host(response, true);
                            Quake::show_info_jf();
                            std::process::exit(0);
                        }
                    };
                    let filename = &output;
                    // save to file.
                    match Quake::save_host_data(filename, response) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e.to_string()));
                        }
                    };
                } else {
                    let host_string = Quake::read_file_host(query_host_file);
                    let query = host_string.as_str();
                    if query == "" {
                        Output::info(&format!("The host file is None!"));
                        std::process::exit(1);
                    }
                    let response = Quake::query_host_by_scroll(query, size);
                    let output = match host_match.get_many::<String>("output") {
                        Some(name) => name.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                        None => {
                            Quake::show_host_by_scroll(response, true);
                            std::process::exit(0);
                        }
                    };
                    let filename = &output;
                    // save to file
                    match Quake::save_host_by_scroll_data(filename, response) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e.to_string()));
                        }
                    };
                }
            }
            Some(("search", search_match)) => {
                let upload = match search_match.get_many::<String>("upload") {
                    Some(file_name) => file_name.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    None => "".to_string(),
                };
                let upload = &upload;
                let query_file = match search_match.get_many::<String>("query_file") {
                    Some(query_file) => query_file
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    None => "".to_string(),
                };
                let query_file = &query_file;
                let query_string;
                let query = match search_match.get_many::<String>("query_string") {
                    Some(query) => query.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    None => {
                        if upload == "" && query_file == "" {
                            Output::error("Error: You must enter a search syntax.\r\nPlease execute -h for help.");
                            std::process::exit(1);
                        } else if query_file != "" {
                            query_string = Quake::read_file_search(query_file);
                            query_string.to_string()
                        } else {
                            "".to_string()
                        }
                    }
                };
                let query = &query;
                let start = match search_match.get_many::<String>("start") {
                    Some(start) => start
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let size = match search_match.get_many::<String>("size") {
                    Some(size) => size
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 10,
                };
                let cdn = match search_match.get_many::<String>("cdn") {
                    Some(cdn) => cdn
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let mg = match search_match.get_many::<String>("honey_jar") {
                    Some(mg) => mg
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let zxsj = match search_match.get_many::<String>("latest_data") {
                    Some(zxsj) => zxsj
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let wxqq = match search_match.get_many::<String>("filter_request") {
                    Some(wxqq) => wxqq
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let sjqc = match search_match.get_many::<String>("deduplication") {
                    Some(sjqc) => sjqc
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let time_start = match search_match.get_many::<String>("time_start") {
                    Some(time_start) => time_start
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    None => "".to_string(),
                };
                let time_start = &time_start;
                let time_end = match search_match.get_many::<String>("time_end") {
                    Some(time_end) => time_end.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    None => "".to_string(),
                };
                let time_end = &time_end;
                if size > 100 {
                    Output::warning("Warning: Size is set to a maximum of 100, if set too high it may cause abnormal slowdowns or timeouts.");
                }
                let data_type_str = match search_match.get_many::<String>("type") {
                    Some(data_type_str) => data_type_str
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => "ip,port".to_string(),
                };
                let data_type: Vec<&str> = data_type_str.split(',').collect();
                let filter = match search_match.get_many::<String>("filter") {
                    Some(filter) => filter.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    None => "".to_string(),
                };
                let filter = &filter;
                if query_file == "" {
                    let response = Quake::query(
                        query, upload, start, size, time_start, time_end, cdn, mg, zxsj, wxqq, sjqc,
                    );
                    let output = match search_match.get_many::<String>("output") {
                        Some(name) => name.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                        None => {
                            Quake::show(response, true, filter, data_type);
                            Quake::show_info_jf();
                            std::process::exit(0);
                        }
                    };
                    let output = &output;
                    // save to file.
                    match Quake::save_search_data(output, response, filter, data_type) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e.to_string()));
                        }
                    };
                } else {
                    if query != "" {
                        Output::info(&format!("Search with {}", query));
                    }
                    let response = Quake::query_for_scroll(
                        query, size, time_start, time_end, cdn, mg, zxsj, wxqq, sjqc,
                    );
                    // Quake::show_scroll(response,true,filter, data_type);
                    let output = match search_match.get_many::<String>("output") {
                        Some(name) => name.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                        None => {
                            Quake::show_scroll(response, true, filter, data_type);
                            std::process::exit(0);
                        }
                    };
                    let output = &output;
                    // save to file.
                    match Quake::save_scroll_data(output, response, filter, data_type) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e.to_string()));
                        }
                    };
                }
            }
            Some(("info", _)) => {
                Quake::show_info();
            }
            //gpt引擎
            Some(("gpt", gpt_match)) => {
                let gptcs = match gpt_match.get_many::<String>("gpt_match") {
                    Some(gptcs) => gptcs.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    None => {
                        Output::error("You have to say something. \r\nPlease execute -h for help.");
                        std::process::exit(1);
                    }
                };

                let gpt_sj: Result<String, Box<dyn Error>> = match Gpt::query_gpt(&gptcs) {
                    Ok(res) => {
                        Output::info(&format!(
                            "Successfully converted the quake language method:{}",
                            res
                        ));
                        Ok(res)
                    }
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        Err(err.into())
                    }
                };
                let upload = match gpt_match.get_many::<String>("upload") {
                    Some(file_name) => file_name.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    None => "".to_string(),
                };
                let upload = &upload;
                let query_file = match gpt_match.get_many::<String>("query_file") {
                    Some(query_file) => query_file
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    None => "".to_string(),
                };
                let query_file = &query_file;
                //let query_string;
                // let query = match gpt_match.get_many::<String>("query_string") {
                //     Some(query) => query.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                //     None => {
                //         if upload == "" && query_file == "" {
                //             Output::error("Error: You must enter a search syntax.\r\nPlease execute -h for help.");
                //             std::process::exit(1);
                //         } else if query_file != "" {
                //             query_string = Quake::read_file_search(query_file);
                //             query_string.to_string()
                //         } else {
                //             "".to_string()
                //         }
                //     }
                // };
                let query = gpt_sj.unwrap().trim_matches('"').replace("\\", "");

                let sizere = Regex::new(r"--size\s+(\d+)").unwrap();
                let time_startre = Regex::new(r"--time_start\s+(\d+-\d+)(?:-\d+)?").unwrap();
                let time_endre = Regex::new(r"--time_end\s+(\d+-\d+)(?:-\d+)?").unwrap();
                let outputre = Regex::new(r"--output\s+([^\s]+)").unwrap();
                let andre = Regex::new(r"and\s*$").unwrap();
                let size = match sizere.captures(query.as_str()) {
                    Some(captures) => {
                        let size = captures.get(1).unwrap().as_str();
                        size.parse::<i32>().unwrap()
                    }
                    None => 10,
                };
                let time_start = match time_startre.captures(query.as_str()) {
                    Some(captures) => {
                        let size = captures.get(1).unwrap().as_str();
                        size.parse::<String>().unwrap()
                    }
                    None => "".to_string(),
                };
                let time_start = &time_start;
                let time_end = match time_endre.captures(query.as_str()) {
                    Some(captures) => {
                        let size = captures.get(1).unwrap().as_str();
                        size.parse::<String>().unwrap()
                    }
                    None => "".to_string(),
                };
                let time_end = &time_end;
                let start = match gpt_match.get_many::<String>("start") {
                    Some(start) => start
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };

                let cdn = match gpt_match.get_many::<String>("cdn") {
                    Some(cdn) => cdn
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let mg = match gpt_match.get_many::<String>("honey_jar") {
                    Some(mg) => mg
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let zxsj = match gpt_match.get_many::<String>("latest_data") {
                    Some(zxsj) => zxsj
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let wxqq = match gpt_match.get_many::<String>("filter_request") {
                    Some(wxqq) => wxqq
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };
                let sjqc = match gpt_match.get_many::<String>("deduplication") {
                    Some(sjqc) => sjqc
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                        .parse::<i32>()
                        .unwrap(),
                    _ => 0,
                };

                let query = sizere.replace_all(&query, "").to_string();
                let query = time_startre.replace_all(&query, "").to_string();
                let query = time_endre.replace_all(&query, "").to_string();
                let outputquery = query.as_str();
                let query = outputre.replace_all(&query, "").to_string();
                let query = andre.replace_all(&query, "").to_string();
                let query = query.replace("\"", "").to_string();

                //query.push('"');
                let query = query.as_str();

                if size > 100 {
                    Output::warning("Warning: Size is set to a maximum of 100, if set too high it may cause abnormal slowdowns or timeouts.");
                }

                let data_type_str = match gpt_match.get_many::<String>("type") {
                    Some(data_type_str) => data_type_str
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => "ip,port".to_string(),
                };
                let data_type: Vec<&str> = data_type_str.split(',').collect();
                let filter = match gpt_match.get_many::<String>("filter") {
                    Some(filter) => filter.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    None => "".to_string(),
                };
                let filter = &filter;
                if query_file == "" {
                    let response = Quake::query(
                        query, upload, start, size, time_start, time_end, cdn, mg, zxsj, wxqq, sjqc,
                    );
                    let output = match outputre.captures(outputquery) {
                        Some(captures) => {
                            let size = captures.get(1).unwrap().as_str();
                            size.parse::<String>().unwrap()
                        }
                        None => {
                            Quake::show(response, true, filter, data_type);
                            std::process::exit(0);
                        }
                    };
                    // let output = match gpt_match.get_many::<String>("output") {
                    //     Some(name) => name.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    //     None => {
                    //         Quake::show(response, true, filter, data_type);
                    //         std::process::exit(0);
                    //     }
                    // };
                    let output = &output;
                    // save to file.
                    match Quake::save_search_data(output, response, filter, data_type) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e.to_string()));
                        }
                    };
                } else {
                    if query != "" {
                        Output::info(&format!("Search with {}", query));
                    }
                    let response = Quake::query_for_scroll(
                        query, size, time_start, time_end, cdn, mg, zxsj, wxqq, sjqc,
                    );
                    // Quake::show_scroll(response,true,filter, data_type);
                    let output = match gpt_match.get_many::<String>("output") {
                        Some(name) => name.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                        None => {
                            Quake::show_scroll(response, true, filter, data_type);
                            std::process::exit(0);
                        }
                    };
                    let output = &output;
                    // save to file.
                    match Quake::save_scroll_data(output, response, filter, data_type) {
                        Ok(count) => {
                            Output::success(&format!(
                                "Successfully saved {} pieces of data to {}",
                                count, output
                            ));
                        }
                        Err(e) => {
                            Output::error(&format!("Data saving failure:{}", e.to_string()));
                        }
                    };
                }
            }
            Some(("honeypot", honeypot_match)) => {
                let ip = match honeypot_match.get_many::<String>("ip") {
                    Some(query) => query.map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                    None => {
                        Output::error(
                            "Error: You must choose a ip.\r\nPlease execute -h for help.",
                        );
                        std::process::exit(1);
                    }
                };
                Quake::honeypot(ip.to_string());
            }
            _ => {}
        }
    }
}

pub struct Output;

impl Output {
    pub fn error(msg: &str) {
        println!("{} {}", Red.bold().paint("[!]"), msg);
    }
    pub fn info(msg: &str) {
        println!("{} {}", Blue.bold().paint("[+]"), msg);
    }
    pub fn success(msg: &str) {
        println!("{} {}", Green.bold().paint("[+]"), msg);
    }
    pub fn warning(msg: &str) {
        println!("{} {}", Yellow.bold().paint("[-]"), msg);
    }
}
