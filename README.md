# Quake Command-Line Application

## 下载 OR 安装
下载：https://github.com/360quake/quake_rs/releases/tag/1.0.1

安装：

```
// 安装rust后使用cargo编译
cargo  build --release
```
## 使用方法
```bash
Quake Command-Line Application 0.1.0
Author: soap  <imelloit@gmail.com>
Dose awesome things.

USAGE:
    quake [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    domain    View all available information for a domain.
    help      Prints this message or the help of the given subcommand(s)
    host      View all available information for an IP address
    init      Initialize the Quake command-line
    search    Search the Quake database
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
[+] Search with domain:"*.360.cn"
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
┬─[kali@kali:~/q/t/release]─[09:35:51 PM]─[G:master=]
╰─>$ ./quake host 5.188.34.101
[+] Search with ip:5.188.34.101
[+] Successful.
[+] count: 1    total: 1
IP: 5.188.34.101
|  3306 mysql   MySQL
|  8080 http
|  80   http    Apache httpd    2.2.15
|  25   smtp
|  22   ssh     OpenSSH 5.3

┬─[kali@kali:~/q/t/release]─[09:38:56 PM]─[G:master=]
╰─>$ ./quake host 5.188.34.101/26
[+] Search with ip:5.188.34.101/26
[+] Successful.
[+] count: 10   total: 55
IP: 5.188.34.98
|  443  http/ssl        Apache httpd    2.2.15
|  22   ssh     OpenSSH 5.3
|  80   http    Apache httpd    2.2.15
|  8080 http
|  25   smtp


IP: 5.188.34.111
|  22   ssh     OpenSSH 7.4
|  8083 http/ssl        nginx
|  993  imap
|  995  pop3
|  5555 http


IP: 5.188.34.69
|  443  http/ssl
|  80   http
|  21   ftp     Pure-FTPd


IP: 5.188.34.109
|  3306 mysql   MySQL   5.1.73
|  80   http    nginx
|  587  smtp    Exim smtpd      4.92.3
|  53   dns
|  25   smtp    Exim smtpd      4.92.3

...
```
#### 3. 搜索查询
```bash
┬─[kali@kali:~/q/t/release]─[09:44:34 PM]─[G:master=]
╰─>$ ./quake search
quake-search
Search the Quake database

USAGE:
    quake search [OPTIONS] [query_string]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --filter <TYPE>        Filter
    -o, --output <FILENAME>    Save the host information in the given file (append if file exists).
        --size <NUMBER>        The size of the number of responses, up to a maximum of 100 (Default 10).
        --start <NUMBER>       Starting position of the query (Default 0).
    -t, --type <TYPE>          Fields displayed:ip,port,title,country,province,city,owner. (Default ip,port)

ARGS:
    <query_string>    Quake Querystring
```
搜索功能相当于在Quake的搜索框中进行搜索，支持Quake的搜索语法。start/size支持翻页，-t 显示返回的字段类型(ip,port,title,country,province,city,owner,time)，-o/--output 支持将搜索结果导出至文件，-f 可以自定义正则表达式去匹配返回数据中的内容并高亮显示。

##### Example：
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


