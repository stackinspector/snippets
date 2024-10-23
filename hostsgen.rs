use std::{io::{self, BufRead}, fs};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DnsRes {
    status: u32,
    answer: Vec<DnsAnswer>,
}

#[derive(Deserialize)]
struct DnsAnswer {
    #[serde(rename = "type")]
    ty: u32,
    // data: std::net::IpAddr,
    data: String,
}

fn main() {
    let mut args = std::env::args_os();
    let _ = args.next();
    let mut list = io::BufReader::new(fs::File::open(args.next().unwrap()).unwrap());
    let dst = args.next();
    let mut dst_h: Box<dyn io::Write> = if let Some(dst_path) = dst {
        Box::new(fs::OpenOptions::new().create_new(true).write(true).open(&dst_path).unwrap())
    } else {
        Box::new(io::stdout().lock())
    };
    let url_prefix = "https://1.1.1.1/dns-query?name=";
    let mut url_buf = String::with_capacity(url_prefix.len());
    loop {
        url_buf.clear();
        url_buf.push_str(&url_prefix);
        let read = list.read_line(&mut url_buf).unwrap();
        if read == 0 { break; }
        // contains \n or \r\n depends on list file
        // won't affect http request and end up the same in dst
        let domain = &url_buf[url_prefix.len()..];
        let dns_res = ureq::get(&url_buf)
            .set("accept", "application/dns-json")
            .set("User-Agent", "curl/8.9.1")
            .call().unwrap();
        assert_eq!(dns_res.status(), 200);
        let dns_res: DnsRes = serde_json::from_reader(dns_res.into_reader()).unwrap();
        assert_eq!(dns_res.status, 0);
        for DnsAnswer { ty, data } in dns_res.answer {
            if ty == 1 {
                let ip: std::net::IpAddr = data.parse().unwrap();
                dst_h.write_fmt(format_args!("{ip} {domain}")).unwrap();
            }
        }
    }
}
