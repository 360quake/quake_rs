use crate::api::ApiKey;
use crate::common::Output;
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::json;
use serde_json::Value;
use std::error::Error;
pub struct Gpt;

fn remove_control_characters(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).collect::<String>()
}
impl Gpt {
    pub fn query_gpt(query_string: &str) -> Result<String, Box<dyn Error>> {
        let mut header = HeaderMap::new();
        const GPT_URL: &str = "https://api.openai.com/v1/chat/completions";

        let mut url: String = String::new();
        let api = ApiKey::get_gptapi().expect("Failed to read GPTapikey:\t");
        //print!("{}",api);
        url.push_str(GPT_URL);
        header.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", api)).unwrap(),
        );
        header.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::new();
        let text = r#"
        作为精通Quake测绘引擎语法的AI助手，我将接收一段文字，并将其转化为简洁的、符合Quake测绘引擎语法的查询语句。然后，我将以文本形式返回结果，确保没有额外文字或多余的回车换行。
    
        你熟练掌握了以下语法参考：

        app:"Apache"Apache服务器产品
        country:"CN"：搜索国家地区资产，可以使用国家缩写"
        country_cn:"中国":用于搜索中文国家名称
        province:"beijing"：用于搜索英文省份名称"
        province_cn:"北京"：用于搜索中文省份名称"
        city:"changsha"：用于搜索英文省份名称"
        city_cn:"长沙"：用于搜索中文城市名称"
        ssl:"google"：搜索ssl证书存在"google"字符串的资产，常常用来提过公司名及产品来搜索对应的资产
        ip:"8.8.8.8"：搜索指定IPv4地址相关资产
        ip:"2600:3c00::f03c:91ff:fefc:574a"：搜索指定IPv6地址相关资产
        ip:52.2.254.36/24:支持检索单个IP、CIDR地址段、支持IPv6地址
        org:"No.31,Jin-rong Street":自治域归属组织名称
        asn:"12345":自治域号码
        isp:"China Mobile"：搜索相关网络服务提供商的资产，可结合org数据相互补充
        asn:42893：搜索对应ASN（Autonomous system number）自治系统编号相关IP资产
        port:80：搜索相关端口资产
        ports:80,8080,9999:搜索多端口资产
        port:>或<port:>=或<=：搜索满足某个端口范围的主机
        port:<80：查询开放端口小于80的主机
        port:[80 TO 1024]：查询开放的端口介入80和1024之间的主机
        port:>=80：查询开放端口包含且大于80端口的主机
        hostname:google.com：搜索相关IP"主机名"的资产
        service:"ssh"：搜索对应服务协议的资产，常见服务协议包括：http、ftp、ssh、telnet等等
        os:"RouterOS"：搜索相关操作系统，常见系统包括Linux、Windows、RouterOS、IOS、JUNOS等等
        title:"Cisco"：搜索html内容里标题中存在"Cisco"的数据
        favicon_hash:"f3418a443e7d841097c714d69ec4bcb8"：通过 md5 方式对目标数据进行解析，根据图标搜索相关内容的资产，搜索包含“google”图标的相关资产
        html_hash:"69d7683445fed9e517e33750615f46c0"：网页html的md5值
        headers:"ThinkPHP"：http headers字符串
        body:"奇虎"：网页body内容
        powered_by:PHP:网站开发语言（http headers里面的X-Powered-By）
        response:"220 ProFTPD 1.3.5a Server"：端口原生返回数据中包含"220 ProFTPD 1.3.5a Server"字符串的主机

        如果出现的关键词可能出现在上面的语法里，请带上语法词，比如输入一个IP，直接使用ip:x.x.x.xx ，如果提交的关键词可能出现在标题里可以使用title:xxx ，如果关键词可能出现在证书里，请使用ssl:xxxx 等
        
        注意不要用上面语法参考里没有包含的语法词！
        
        在quake搜索中，用or表示or运算，用and表示and运算，用not表示not运算，括号表示优先处理！
        
        注意：
        遇到搜索“可能”属于xxx的资产时候，要使用or运算（也就是空格），应该考虑title、ssl、hostname、org、isp等语法，最终输出的quake查询语法：(title:"xxx" ssl:"xxx" hostname:"xxx" org:"xxx" isp:"xxx")
        
        下面这是一些学习的例子：
        
        提示词："搜索1.1.1.1c段地址"
        输出结果："ip:1.1.1.1/24"

        提示词："搜索1.1.1.1b段地址"
        输出结果："ip:1.1.1.1/16"

        提示词："不要来自某个国下的某个省的数据"
        输出结果：country_cn:"某个国" and not province_cn:"某个省"

        提示词："不要来自香港的数据"
        输出结果："not city_cn:"香港" AND country: "China""

        提示词："不要来自台湾的数据apache数据"
        输出结果："not province_cn:"台湾省" AND country: "China"  and  app:Apache"

        提示词："不要来自澳门的数据"
        输出结果："not city_cn:"澳门" AND country: "China""
        
        提示词："美国开放80或者443端口的主机"
        输出结果：country:"US" and (port:"80" port:"443")
         提示词："我需要湖南省长沙市所有使用ChatGPT的网站"
        提示词："Russian hosts running RDP or FTP"
        输出结果：country:"RU" and (service:"rdp" service:"ftp")
        
        提示词："我需要湖南省长沙市所有使用ChatGPT的网站"
        输出结果：country:"中国" and province_cn:"湖南" and city_cn:"长沙" and title:"ChatGPT"
        
        提示词："搜索存在目录遍历的资产"
        输出结果：title:"Index of /"

        提示词："不要来自美国的数据"
        输出结果：country:"US""

        提示词："xxxx年xx月到xxxx年xxx月"
        输出结果：--time_start xxxx-xx --time_end xxxx-xxx"(前面不要加and符号)
        
        提示词："搜索使用GoAhead的Web服务器"
        输出结果："Server: GoAhead"

        提示词："返回包存在xxx"
        输出结果："response:xxx"

        提示词："xx条或者xx个"
        输出结果："--size xx"(前面不要加and符号)

        提示词："xx一打"
        输出结果："--size 12"(前面不要加and符号)

        提示词："导出到当前目录下a.txt"
        输出结果："--output ./a.txt"(前面不要加and符号)
        
        提示词："导出到xxxx"
        输出结果："--output xxxx"(前面不要加and符号)
        

        提示词："状态码xxx"
        输出结果："status_code:xxx"

        请认真学习并记住这些案例，请特别注意如果输入的是Web服务器如Apache、IIS、nginx、GoAhead等，明确是用于Web服务器的，请使用http头里的关键词，如"server: Server: Microsoft-IIS/7.5"，而不能使用server:"Microsoft-IIS/7.5"
        
        注意：台湾是中国的一个省，禁止当作国家处理，要用province_cn:"台湾省"这个代表，禁止使用country_cn或country参数。
        注意：香港和澳门是中国的一个省或者城市，禁止当作国家处理，要用province_cn:"xx"这个代表或者city_cn:"xx"，禁止使用country_cn或country参数。
        
        注意：在--size和--start_time和--end_time标签前禁止加and。
        注意：quake使用or表示“或者”,禁止使用|或者||。
        注意：禁止返回额外文字与多余的回车换行和\符号，只需要返回quake搜索语法。"#;
        let text1 = remove_control_characters(text);
        let sj = query_string;
        let json_data = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "content": format!("{} \n\n {}",text1,sj),
                    "role": "user"
                }
            ]
        });

        let response: Result<String, reqwest::Error> = client
            .post(&url)
            .headers(header)
            .json(&json_data)
            .send()
            .unwrap()
            .text();

        match response {
            Ok(res) => {
                //println!("{}", res);
                let res: Value = serde_json::from_str(&res)?;
                let code = res["choices"][0]["message"]["content"].to_string();
                Ok(code)
                //println!("{:}",code);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                Err(err.into())
            }
        }
    }
}
