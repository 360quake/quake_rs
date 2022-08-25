# Quake Command-Line Application

## 安装

1. 直接下载即可使用

    https://github.com/360quake/quake_rs/releases/

2. 或者本地编译：
    
    ```
    // 安装rust后使用cargo编译
    cargo  build --release
    ```
   
## 更新日志
* 2022-08-25 v2.1:
    * 优化新增排除cdn、排除蜜罐、显示最新数据参数
    * 新增workflows工作流打包成release
* 2021-08-12 v2.0.3:
    * 新增排除蜜罐、排除CDN、使用最新数据功能
    * 新增过滤无效请求和数据去重功能
* 2021-04-06 v2.0.2:
    * 修复域名搜不到的问题。 :)
* 2021-04-06 v2.0.1:
    * 优化搜索结果，去除重复数据。  
    * 添加文件上传搜索功能。
    * 添加指定时间搜索功能。
    * 优化代码。
* 2021-01-22 v1.0.5:
    * 修复TLS解构解析不一致的问题。
    * 修复命令行工具被杀软报毒问题。
* 2021-01-15 v1.0.4:
    * 优化title显示，删除不可见字符。
    * host命令新增地理位置、设备信息和更新时间字段。
    * 修复域名查询，出现无关域名的问题。
* 2021-01-08 v1.0.3:
    * 修复init命令bug
    * 新增证书域名提取。
* 2020-12-25 v1.0.2 : 
    * 添加info和honeypot子命令，可以查看个人信息和进行蜜罐识别。

## 问题反馈
请添加微信：quake_360 邀您加入技术交流群 :)

## 使用方法
```bash
Quake Command-Line Application 1.0.4
Author: soap  <imelloit@gmail.com>
Dose awesome things.

USAGE:
    quake [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    domain      View all available information for a domain.
    help        Prints this message or the help of the given subcommand(s)
    honeypot    Check whether the IP is a honeypot or not.
    host        View all available information for an IP address
    info        Shows general information about your account
    init        Initialize the Quake command-line
    search      Search the Quake database
```
#### 1. 初始化

*ApiKey请到Quake个人中心查看*

```bash
quake init apikey
```
#### 2. 域名查询
```bash
┬─[kali@kali:~/q/t/release]─[09:29:44 PM]─[G:master=]
╰─>$ ./quake domain
quake-domain
View all available information for a domain.

USAGE:
    quake domain [FLAGS] [OPTIONS] [DOMAIN_NAME]

FLAGS:
    -c, --count      Count of results
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <FILENAME>    Output result to file.
        --size <NUMBER>        The size of the number of responses, up to a maximum of 100 (Default 10).
        --start <NUMBER>       Starting position of the query (Default 0).
    -t, --type <TYPE>          Fields displayed:domain,ip,port,title. (Default domain, ip)

ARGS:
    <DOMAIN_NAME>    The domain name to be queried.163.251.254 extsign-b.browser.360.cn
```
域名查询可以查询某个域名的子域名和相似域名，或者域名周含有某些关键子的域名，如：oa、vpn等，可以设置start和size参数进行翻页，可以使用-o/--output，将查询结果保存至文件。
-t参数可以控制显示的字段，有domain、ip、端口和站点title等。

##### Example：
```bash
┬─[kali@kali:~/q/t/release]─[09:33:10 PM]─[G:master=]
╰─>$ ./quake domain 360.cn --start 10 --size 10
[+] Search with domain:"*.360.cn"
[+] Successful.
[+] count: 10   total: 11586
221.130.199.210 audit-s.scan.shouji.360.cn
112.65.68.50    mall.safe.360.cn
112.65.68.51    mall.safe.360.cn
27.115.124.94   salesvideotest.crm.360.cn
101.198.192.187 salesvideotest.crm.360.cn
111.206.65.160  0479ae.link.yunpan.360.cn
111.206.65.160  0479ae.link.yunpan.360.cn
112.64.200.118  hb065779_893.fx.sj.360.cn
112.64.200.118  hb065779_893.fx.sj.360.cn
180.163.235.141 up-q.f.360.cn

┬─[kali@kali:~/q/t/release]─[09:33:19 PM]─[G:master=]
╰─>$ ./quake domain 360.cn --start 10 --size 10 -t ip,port,domain,title
[+] Search with domain:*.360.cn
[+] Successful.
[+] count: 10   total: 11586
221.130.199.210 443     audit-s.scan.shouji.360.cn
112.65.68.50    80      mall.safe.360.cn        安全卫士-360商城
112.65.68.51    443     mall.safe.360.cn        安全卫士-360商城
27.115.124.94   80      salesvideotest.crm.360.cn       403 Forbidden
101.198.192.187 443     salesvideotest.crm.360.cn       400 The plain HTTP request was sent to HTTPS port
111.206.65.160  80      0479ae.link.yunpan.360.cn       安全存储的云盘_360安全云盘
111.206.65.160  443     0479ae.link.yunpan.360.cn       安全存储的云盘_360安全云盘
112.64.200.118  80      hb065779_893.fx.sj.360.cn
112.64.200.118  443     hb065779_893.fx.sj.360.cn
180.163.235.141 80      up-q.f.360.cn   403 Forbidden
```

#### 3. IP查询
```bash
┬─[kali@kali:~/q/t/release]─[09:35:45 PM]─[G:master=]
╰─>$ ./quake host
quake-host
View all available information for an IP address

USAGE:
    quake host [OPTIONS] [ip]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <FILENAME>    Save the host information in the given file (append if file exists).
        --size <NUMBER>        The size of the number of responses, up to a maximum of 100 (Default 10).
        --start <NUMBER>       Starting position of the query (Default 0).

ARGS:
    <ip>     View all available information for an IP address
```
可以快速查询某个IP或IP段下开放的端口和服务。start/size参数支持翻页，-o/--output支持将搜索结果保存至文件。

##### Example：
```bash
┬─[kali@kali:~/q/t/release]─[10:14:57 PM]─[G:master=]
╰─>$ ./quake host 5.188.34.101
[+] Search with ip:5.188.34.101
[+] Successful.
[+] count: 1    total: 1
IP: 5.188.34.101        Country: Singapore      Province: Singapore     City: Singapore
| Port              Protocol                    time
| 3306                 mysql    2020-10-03T03:23:14.385Z
| 8080                  http    2020-10-29T01:11:01.721Z
| 80                    http    2020-10-30T16:19:33.698Z
| 25                    smtp    2020-11-05T15:10:57.932Z
| 22                     ssh    2020-12-11T03:02:01.624Z

┬─[kali@kali:~/q/t/release]─[10:15:06 PM]─[G:master=]
╰─>$ ./quake host 5.188.34.101/24
[+] Search with ip:5.188.34.101/24
[+] Successful.
[+] count: 10   total: 222
IP: 5.188.34.203        Country: Singapore      Province: Singapore     City: Singapore
| Port              Protocol                    time
| 80                    http    2020-12-21T14:44:17.322Z


IP: 5.188.34.17 Country: Singapore      Province: Singapore     City: Singapore
| Port              Protocol                    time
| 22                     ssh    2020-06-30T07:14:12.077Z
| 111                rpcbind    2020-12-22T15:56:07.436Z
| 123                    ntp    2020-12-24T12:53:05.514Z


IP: 5.188.34.41 Country: Singapore      Province: Singapore     City: Singapore
| Port              Protocol                    time
| 22                     ssh    2021-01-06T16:30:24.237Z


IP: 5.188.34.252        Country: Singapore      Province: Singapore     City: Singapore
| Port              Protocol                    time
| 995                   pop3    2020-12-14T21:23:37.832Z
| 80                    auto    2020-12-21T22:42:56.926Z


IP: 5.188.34.218        Country: Singapore      Province: Singapore     City: Singapore
| Port              Protocol                    time
| 143                   imap    2020-12-17T02:45:47.230Z
| 25                    smtp    2020-12-21T12:54:12.368Z
| 3306                 mysql    2020-12-28T02:10:44.445Z
...
```
#### 3. 搜索查询
```bash
┬─[kali@kali:~/q/t/release]─[09:44:34 PM]─[G:master=]
╰─>$ ./quake search
╰─>$ ./quake search 
quake-search 
Search the Quake database

USAGE:
    quake search [OPTIONS] [query_string]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --filter <TYPE>              Filter search results with more regular expressions.
                                     Example: quake search 'app:"exchange 2010"' -t ip,port,title -f "X-OWA-Version:
                                     (.*)"
    -o, --output <FILENAME>          Save the host information in the given file (append if file exists).
        --size <NUMBER>              The size of the number of responses, up to a maximum of 100 (Default 10).
        --start <NUMBER>             Starting position of the query (Default 0).
    -e, --end_time <TIME END>        Search end time
                                     Example: quake search 'port:80' -e 2020-01-01
    -s, --start_time <TIME START>    Search start time
                                     Example: quake search 'port:80' -s 2020-01-01
    -t, --type <TYPE>                Fields displayed:ip,port,title,country,province,city,owner,time,ssldomain,domain.
                                     (Default ip,port)
    -u, --upload <IP File>           Uploading *.txt files containing only IP addresses, with no more than 1000 IPs.
                                     Example: quake search -u ips.txt

ARGS:
    <query_string>    Quake Querystring
```
搜索功能相当于在Quake的搜索框中进行搜索，支持Quake的搜索语法。start/size支持翻页，-t 显示返回的字段类型(ip,port,title,country,province,city,owner,time,ssldomain)，-o/--output 支持将搜索结果导出至文件，-f 可以自定义正则表达式去匹配返回数据中的内容并高亮显示。
-e --end_time 搜索结束时间  -s --start-time 搜索开始时间。指定某段时间内返回的数据。
-u --upload 上传一个IP列表(不超过1000条)，进行批量查询。

##### Example：

```
╰─>$ ./quake search -u ips.txt
[+] Search for 44 IPs
[+] Successful.
[+] count: 10 	total: 81
165.229.11.173	443	
123.194.137.183	80	
123.194.137.183	443	
112.90.184.180	8443	
112.90.184.180	8081	
106.75.10.72	443	
112.90.184.180	27017	
165.229.11.173	80	
123.194.137.183	1723	
123.194.137.183	23
```

```
╰─>$ ./quake search 'port:80' -e 2022-01-01 -s 2011-01-01
[+] Search with port:80
[+] Successful.
[+] count: 10 	total: 118182073
185.8.172.250	80	
142.111.218.92	80	
142.111.241.223	80	
185.8.100.216	80	
217.62.137.0	80	
217.58.176.17	80	
179.43.148.203	80	
185.69.223.29	80	
185.69.240.170	80	
217.31.44.144	80
```

```bash
┬─[kali@kali:~/q/t/release]─[09:47:10 PM]─[G:master=]
╰─>$ ./quake search 'app:"exchange 2010"' -t ip,port,title
[+] Search with app:"exchange 2010"
[+] Successful.
[+] count: 10   total: 75905
216.31.255.176  443     Outlook Web App
216.196.183.114 443     Outlook Web App
200.70.57.149   443     Outlook Web App
95.0.14.166     443     Outlook Web App
31.13.16.76     443     Outlook Web App
178.249.68.117  443     Outlook Web App
212.192.35.55   443     Outlook Web App
188.165.67.142  443     Outlook Web App
23.28.247.211   443     Outlook Web App
182.93.16.236   443     Outlook Web App
```
查看ssl证书中的Common Name（包含域名等信息）。

```bash
┬─[kali@kali:~/quake_rs]─[02:29:15 AM]─[G:master=]
╰─>$ quake search 'app:"Exchange邮件服务器" AND port:443' -t ip,port,ssldomain
[+] Search with app:"Exchange邮件服务器" AND port:443
[+] Successful.
[+] count: 10   total: 1802535
192.67.33.101   443     mail.esolutionsgroup.ca
136.243.51.125  443     mail.misotextile.com
185.214.232.134 443     *.sued.cloud
128.204.218.20  443     www.cfg.com.ua
174.47.121.211  443     owa.dsb-cpa.com
67.134.230.115  443     mymail.swlaw.com
177.107.171.5   443     webmail.bottero.net
12.24.24.202    443     mail.haaskennedy.com
220.128.255.59  443     TWSRVEXCHANGE01.imlauto.com.tw
77.94.137.35    443     m.iscar.si
```

正则表达式可以灵活运用，匹配想匹配的任何数据。
正则匹配exchange内部版本。
```bash
┬─[kali@kali:~/q/t/release]─[09:47:32 PM]─[G:master=]
╰─>$ ./quake search 'app:"exchange 2010"' -t ip,port,title -f "X-OWA-Version: (.*)"
[+] Search with app:"exchange 2010"
[+] Successful.
[+] count: 10   total: 75905
216.31.255.176  443     Outlook Web App X-OWA-Version: 14.3.361.1
216.196.183.114 443     Outlook Web App X-OWA-Version: 14.3.123.3
200.70.57.149   443     Outlook Web App X-OWA-Version: 14.3.352.0
95.0.14.166     443     Outlook Web App X-OWA-Version: 14.3.487.0
31.13.16.76     443     Outlook Web App X-OWA-Version: 14.3.468.0
178.249.68.117  443     Outlook Web App X-OWA-Version: 14.3.439.0
212.192.35.55   443     Outlook Web App X-OWA-Version: 14.3.415.0
188.165.67.142  443     Outlook Web App X-OWA-Version: 14.3.487.0
23.28.247.211   443     Outlook Web App X-OWA-Version: 14.3.409.0
182.93.16.236   443     Outlook Web App X-OWA-Version: 14.3.487.0
```

```
┬─[kali@kali:~/q/t/release]─[02:06:45 AM]─[G:master=]
╰─>$ ./quake search 'response:"CobaltStrike Beacon configurations"' -t ip,port,title -f "C2 Server: (.*)"
[+] Search with response:"CobaltStrike Beacon configurations"
[+] Successful.
[+] count: 10   total: 4424
155.94.135.156  8080            C2 Server: 204.44.85.4,/pixel.gif
124.70.1.140    8081            C2 Server: 124.70.1.140,/dpixel
47.101.147.201  8080            C2 Server: 47.101.147.201,/updates.rss
47.104.108.112  8080            C2 Server: 47.104.108.112,/j.ad
39.100.147.159  8080            C2 Server: 39.100.147.159,/__utm.gif
18.166.76.239   8080            C2 Server: 18.166.76.239,/visit.js
49.233.155.141  7001            C2 Server: 49.233.155.141,/ca
47.56.224.63    8888            C2 Server: 47.56.224.63,/updates
47.75.55.181    8443            C2 Server: 47.75.55.181,/fwlink
34.97.55.204    8080            C2 Server: 34.97.55.204,/pixel
```

#### 4. 用户信息
可以查看用户信息
```
┬─[kali@kali:~]─[03:51:18 AM]
╰─>$ quake info
[+] Successful.
[+] 用户名:  XXXX
[+] 邮  箱:  user@example.com
[+] 手  机:  +xx xxxxxxxxxxx
[+] 月度积分: 3000
[+] 长效积分: 3000
[+] 角  色:  注册用户
```

#### 5. 蜜罐识别
可以识别部分蜜罐系统。
```
┬─[kali@kali:~]─[04:40:00 AM]
╰─>$ quake honeypot 93.89.146.23
[+] Search with 93.89.146.23
[!] Looks like a Kojoney SSH  honeypot system!
```
