use std::collections::HashMap;

use reqwest::header::HeaderMap;

///此处为请求头的组成
///
/// User-agent标明是正常请求操作，防止被反爬
///
/// Origin 是因为网站开启了strict-origin-when-cross-origin策略不得不采用
///
/// Referer 是网站开启了防止恶意请求操作的功能
fn header_make() -> HeaderMap {
    let mut h = reqwest::header::HeaderMap::new();
    h.insert("User-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.99 Safari/537.36 Edg/97.0.1072.76".parse().unwrap());
    h.insert("Origin", "http://ecard.csu.edu.cn:8070".parse().unwrap());
    h.insert("Referer", "http://ecard.csu.edu.cn:8070/Account/Login".parse().unwrap());
    h
}

///此处为登陆表单的组成
///
/// next_url决定登陆后重定向到哪个网址
///
/// 具体页面可以通过对next_url使用base64::decode()得知
///
/// 具体有那些页面可以登陆http://ecard.csu.edu.cn:8070/Account/Login查看
///
/// 例子
/// ```
/// let next_url = "aHR0cDovL2VjYXJkLmNzdS5lZHUuY246ODA3MC9BdXRvUGF5L1Bvd2VyRmVlL0NzdUluZGV4lo";
/// let url = base64::decode("next_url");
/// //url = "http://ecard.csu.edu.cn:8070/AutoPay/PowerFee/CsuIndex"
/// ```
/// SignType为登陆方式有以下三种,此处选用学工号
/// ```
/// //校园卡号
/// "SynCard"
/// //学工号
/// "SynSno"
/// //服务平台账号
/// "SynDream"
/// ```
/// openid为空，确实在表单上就是空，也不知道为什么
///
/// Schoolcode在表单上被写死是csu
fn map_maker<'a>(user_account: &'a str, password: &'a str) -> HashMap<&'a str, &'a str> {
    let mut map = HashMap::new();
    map.insert("UserAccount", user_account);
    map.insert("Password", password);
    map.insert("NextUrl", "");
    map.insert("SignType", "SynSno");
    map.insert("openid", "");
    map.insert("Schoolcode", "csu");
    map
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    ///
    /// 登陆更改配置
    /// user_account为用户名，这里选取学工号作为登录账号
    /// 例如
    /// ```
    /// 8000123456
    /// ```
    /// password为登陆密码通常为身份证后六位
    ///
    /// aspnet_sessionld请登陆浏览器查看获得
    ///
    /// 以及最后的sendkey和server ip:port记得更改
    let user_account = "1234561234";
    let password = "123456";
    let aspnet_sessionld = "ASP.NET_SessionId=q************v;";
    ///此处为固定设置
    ///
    /// password要经过base64编码后才能使用
    ///
    /// header_make()生成请求头所需文件
    ///
    /// map_maker()生成请求所需表单

    let password = base64::encode(password);
    let header = header_make();
    let map = map_maker(user_account, password.as_str());

    ///此处为获取iPlanetDirectoryPro
    ///
    /// 通过对该网页登陆获得的Response来获取
    let url = "http://ecard.csu.edu.cn:8070/Account/Login";
    let client = reqwest::Client::new();
    let res = client.post(url)
        .json(&map)
        .headers(header)
        .send().await?;
    let ipdp = res.headers().get("set-cookie").unwrap().to_str().unwrap().split_once(";").unwrap().0;
    // println!("{:?}",ipdp);

    ///此处为拼装Cookie
    ///
    /// Cookie的组成为
    /// ```
    /// //ASP.NET_Seesionld+iPlanetDirectoryPro
    /// //例如
    /// "Cookie":"ASP.NET_SessionId=qds51a...asddw2v; iPlanetDirectoryPro=U3lu...OTcx"
    /// ```
    let url = "http://ecard.csu.edu.cn:8070/AutoPay/PowerFee/CsuIndex";
    let mut header = header_make();
    let cookie = aspnet_sessionld.to_owned() + ipdp;
    // println!("{:?}",cookie);
    header.insert("Cookie", cookie.parse().unwrap());

    ///此处为登陆网站
    ///
    /// res为获得的页面信息
    ///
    /// -----稍后通过正则表达式筛选所需元素
    ///
    /// 因为不会用正则所以用split通过标签特征分割
    let client = reqwest::Client::new();
    let res = client.get(url)
        .headers(header)
        .send().await?;
    //使用正则筛选并将其转化为f64
    let html = res.text().await?;
    // println!("Text: {}", html);
    let money_str = html.split_once("<span id=\"getbanlse\" style=\"color:red\">").unwrap().1.split_once("</span>").unwrap().0;
    // println!("{}",money_str);
    let money = money_str.trim().to_string().parse::<f64>().unwrap();
    // println!("{}",test);

    //当余额低于阈值则发送消息通知
    //采用server酱的开源版本wecom酱
    //自行部署在服务器上
    //具体可看https://github.com/easychen/wecomchan
    if money < 10.0 {
        let sendkey = "your sendkey";
        let msg = "剩余的电费 ".to_string() + &money.to_string();
        let url = "http://your server:port/wecomchan?sendkey=".to_string() + sendkey + "&msg=" + msg.as_str() + "&msg_type=text";
        // println!("{}",url);
        reqwest::get(url).await;
    }
    Ok(())
}
