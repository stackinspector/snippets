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
}

enum_with_parse! {
    enum Provider {
        "cf1" -> Cf1
        "cf2" -> Cf2
        "dnspod" -> Dnspod
        "ali1" -> Ali1
        "ali2" -> Ali2
    } raises ()
}

impl Provider {
    fn ipv4(&self) -> &'static str {
        match self {
            Provider::Cf1    => "1.1.1.1",
            Provider::Cf2    => "1.0.0.1",
            Provider::Dnspod => "1.12.12.12",
            Provider::Ali1   => "223.5.5.5",
            Provider::Ali2   => "223.6.6.6",
        }
    }

    fn word(&self) -> &'static str {
        match self {
            Provider::Cf1    |
            Provider::Cf2    |
            Provider::Dnspod => "dns-query",
            Provider::Ali1   |
            Provider::Ali2   => "resolve",
        }
    }

    // TODO -> &'static str
    fn build_url_prefix(&self) -> String {
        format!("https://{}/{}?name=", self.ipv4(), self.word())
    }
}

fn main() {
    // cargo run --release --bin hostsgen -- cf2 hosts-list hosts
    let mut args = std::env::args_os();
    let _ = args.next();
    let provider: Provider = args.next().unwrap().into_string().unwrap().parse().unwrap();
    let mut list = io::BufReader::new(fs::File::open(args.next().unwrap()).unwrap());
    let dst = args.next();
    let mut dst_h: Box<dyn io::Write> = if let Some(dst_path) = dst {
        Box::new(fs::OpenOptions::new().create_new(true).write(true).open(&dst_path).unwrap())
    } else {
        Box::new(io::stdout().lock())
    };
    let url_prefix = provider.build_url_prefix();
    let mut url_buf = String::with_capacity(url_prefix.len());
    loop {
        url_buf.clear();
        url_buf.push_str(&url_prefix);
        let read = list.read_line(&mut url_buf).unwrap();
        if read == 0 { break; }
        // contains \n or \r\n depends on list file
        // won't affect http request (handled by url crate used by ureq) and end up the same in dst
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
                let ip: IpAddr = data.parse().unwrap();
                dst_h.write_fmt(format_args!("{ip} {domain}")).unwrap();
            }
        }
    }
}
