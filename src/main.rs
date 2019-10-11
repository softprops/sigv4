use futures::future::Future;
use rusoto_core::{signature::SignedRequest, Client, Region};
use smallvec::SmallVec;
use std::error::Error as StdError;
use structopt::StructOpt;
mod error;
use error::Error;
use std::fmt;
use colored_json::ToColoredJson;
use colored::Colorize;

#[derive(StructOpt)]
#[structopt(name = "sigv4", about = "sign aws sigv4 requests like a prod")]
struct Options {
    #[structopt(short = "r", long = "region", default_value = "us-east-1")]
    region: String,
    #[structopt(short = "s", long = "service", default_value = "execute-api")]
    service: String,
    #[structopt(short = "X", long = "request", default_value = "GET")]
    method: String,
    #[structopt(short = "i", long = "include")]
    include_headers: bool,
    #[structopt(short = "H", long = "header")]
    headers: Vec<String>,
    #[structopt(short = "d", long = "data")]
    data: Option<String>,
    uri: String,
}

impl Into<SignedRequest> for Options {
    fn into(self: Options) -> SignedRequest {
        let Options {
            region,
            service,
            method,
            headers,
            data,
            uri,
            ..
        } = self;
        let region = Region::Custom {
            name: region,
            endpoint: uri,
        };
        let mut request = SignedRequest::new(&method, &service, &region, Default::default());
        for header in headers {
            if let [key, value] = &header.splitn(2, ':').collect::<SmallVec<[_; 2]>>()[..] {
                request.add_header(key.trim(), value.trim())
            }
        }
        request.set_payload(data.map(String::into_bytes));
        request
    }
}

struct Display((rusoto_core::request::BufferedHttpResponse, bool));

impl fmt::Display for Display {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> std::result::Result<(), fmt::Error> {
        let Display((res, include_headers)) = self;
        if *include_headers {
            writeln!(f, "HTTP/2 {}", res.status.to_string().bold())?;
            for (k, v) in &res.headers {
                writeln!(f, "{}: {}", k.to_string().dimmed(), v)?;
            }
            f.write_str("\n")?;
        }
        let body = std::str::from_utf8(&res.body).unwrap_or_default();
        if res.headers.get("content-type").iter().any(|value| "application/json" == *value) {
            write!(f, "{}", body.to_colored_json_auto().unwrap())?;
            //colored_json::write_colored_json(&value, &mut f)?
            //writeln!(f, "{}", std::str::from_utf8(&res.body).unwrap_or_default())?;
        } else {
            f.write_str(body)?;
        }

        Ok(())
    }
}

fn run(options: Options) -> Result<(), Box<dyn StdError>> {
    let Options {
        include_headers, ..
    } = options;
    let response = Client::shared()
        .sign_and_dispatch::<_, Error>(options.into(), |response| {
            Box::new(response.buffer().from_err())
        })
        .sync()?;
    println!("{}", Display((response, include_headers)));
    Ok(())
}

fn main() {
    env_logger::init();
    if let Err(e) = run(Options::from_args()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_require_uri() {
        assert!(Options::from_iter_safe(&["sigv4"]).is_err());
        assert!(Options::from_iter_safe(&["sigv4", "http://foo.com"]).is_ok());
    }

    #[test]
    fn options_default_to_get_method() {
        assert_eq!(
            Options::from_iter(&["sigv4", "http://foo.com"]).method,
            "GET"
        );
    }

    #[test]
    fn options_default_to_execute_api_service() {
        assert_eq!(
            Options::from_iter(&["sigv4", "http://foo.com"]).service,
            "execute-api"
        );
    }
}
