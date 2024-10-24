use std::{fs, io::{self, BufRead}, net::IpAddr, str::FromStr};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
// TODO not in rfc8427?
struct DnsRes {
    // DNS rcode (?), 4bits
    status: u8,
    answer: Vec<DnsAnswer>,
}

// TODO: serde-rs/serde#745
// #[allow(dead_code, clippy::upper_case_acronyms)]
// #[derive(Deserialize)]
// #[serde(tag = "type", content = "data")]
// enum DnsAnswer {
//     #[serde(rename = "1")]
//     A(Ipv4Addr),
//     #[serde(rename = "28", skip)]
//     AAAA(Ipv6Addr),
//     #[serde(rename = "5", skip)]
//     CNAME(String),
// }

#[derive(Deserialize)]
struct DnsAnswer {
    #[serde(rename = "type")]
    ty: u16,
    // data: IpAddr,
    data: String,
}

macro_rules! enum_with_parse {
    ($vis:vis enum $name:ident { $($variant_str:literal -> $variant_name:ident)* } raises $error_ty:ty) => {
        $vis enum $name { $($variant_name,)* }
        impl FromStr for $name {
            type Err = $error_ty;
        
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    // TODO $(stringify!($variant_name) => Self::$variant_name,)* // needs `stringify!` case convert
                    $($variant_str => Self::$variant_name,)*
                    _ => return Err(()),
                })
            }
        }
    };
    ($vis:vis enum $name:ident { $($variant_str:literal -> $variant_name:ident)* } fallback $fallback_name:ident) => {
        $vis enum $name { $($variant_name,)* $fallback_name(String) }
        impl FromStr for $name {
            type Err = core::convert::Infallible;
        
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    // TODO $(stringify!($variant_name) => Self::$variant_name,)* // needs `stringify!` case convert
                    $($variant_str => Self::$variant_name,)*
                    s => Self::$fallback_name(s.to_owned()),
                })
            }
        }
    };
}

enum_with_parse! {
    enum Provider {
        "cf1" -> Cf1
        "cf2" -> Cf2
        "dnspod" -> Dnspod
        "ali1" -> Ali1
        "ali2" -> Ali2
    } fallback Custom
}

impl Provider {
    fn ipv4(&self) -> &str {
        match self {
            Provider::Cf1    => "1.1.1.1",
            Provider::Cf2    => "1.0.0.1",
            Provider::Dnspod => "1.12.12.12",
            Provider::Ali1   => "223.5.5.5",
            Provider::Ali2   => "223.6.6.6",
            Provider::Custom(ipv4) => ipv4,
        }
    }

    fn word(&self) -> &'static str {
        match self {
            Provider::Cf1       |
            Provider::Cf2       |
            Provider::Dnspod    |
            Provider::Custom(_) => "dns-query",
            Provider::Ali1      |
            Provider::Ali2      => "resolve",
        }
    }

    // TODO -> &'static str
    // TODO url::Builder
    // TODO ureq::get<T: AsUrl>(url: T)
    fn build_url_prefix(&self) -> String {
        format!("https://{}/{}?name=", self.ipv4(), self.word())
    }
}

enum_with_parse! {
    enum InputType {
        "list" -> List
        "args" -> Args
    } raises ()
}

struct LineFileIter {
    handle: io::BufReader<fs::File>,
    url_prefix: String,
    url_buf: String,
}

impl LineFileIter {
    fn new(path: std::ffi::OsString, provider: Provider) -> LineFileIter {
        let handle = io::BufReader::new(fs::File::open(path).unwrap());
        let url_prefix = provider.build_url_prefix();
        let url_buf = String::with_capacity(url_prefix.len());
        LineFileIter { handle, url_prefix, url_buf }
    }

    fn next(&mut self) -> Option<(&str, &str)> {
        self.url_buf.clear();
        self.url_buf.push_str(&self.url_prefix);
        let read = self.handle.read_line(&mut self.url_buf).unwrap();
        if read == 0 {
            None
        } else {
            // contains \n or \r\n depends on list file
            // won't affect http request (handled by url crate used by ureq) and end up the same in dst
            // TODO utf8 check overhead?
            let domain = &self.url_buf[self.url_prefix.len()..];
            Some((&self.url_buf, domain))
        }
    }
}

fn main() {
    // cargo run --release --bin hostsgen -- cf2 list hosts-list hosts
    // cargo run --release --bin hostsgen -- cf2 args example.com
    let mut args = std::env::args_os();
    let _ = args.next();
    let provider: Provider = args.next().unwrap().into_string().unwrap().parse().unwrap();
    let input_type: InputType = args.next().unwrap().into_string().unwrap().parse().unwrap();
    assert!(matches!(input_type, InputType::List));
    let mut list = LineFileIter::new(args.next().unwrap(), provider);
    let dst = args.next();
    let mut dst_h: Box<dyn io::Write> = if let Some(dst_path) = dst {
        Box::new(fs::OpenOptions::new().create_new(true).write(true).open(&dst_path).unwrap())
    } else {
        Box::new(io::stdout().lock())
    };
    loop {
        if let Some((url, domain)) = list.next() {
            let dns_res = ureq::get(url)
                .set("accept", "application/dns-json")
                .set("User-Agent", "curl/8.9.1")
                .call().unwrap();
            assert_eq!(dns_res.status(), 200);
            let dns_res: DnsRes = serde_json::from_reader(dns_res.into_reader()).unwrap();
            assert_eq!(dns_res.status, 0);
            // TODO: serde-rs/serde#745
            // for item in dns_res.answer {
            //     if let DnsAnswer::A(ipv4) = item {
            //         dst_h.write_fmt(format_args!("{ipv4} {domain}")).unwrap();
            //     }
            // }
            for DnsAnswer { ty, data } in dns_res.answer {
                if ty == 1 {
                    let ip: IpAddr = data.parse().unwrap();
                    dst_h.write_fmt(format_args!("{ip} {domain}")).unwrap();
                }
            }
        } else {
            break;
        }
    }
}
