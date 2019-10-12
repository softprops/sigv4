use futures::future::Future;
use rusoto_core::{request::BufferedHttpResponse, signature::SignedRequest, Client, Region};
use smallvec::SmallVec;
use std::{error::Error as StdError, process::exit};
use structopt::StructOpt;
mod error;
use colored::Colorize;
use colored_json::{ColorMode, ToColoredJson};
use error::Error;
use std::{
    convert::TryInto,
    fmt,
    fs::read_to_string,
    io::{self, Read},
    str,
};

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

impl TryInto<SignedRequest> for Options {
    type Error = io::Error;
    fn try_into(self: Options) -> io::Result<SignedRequest> {
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
        if let Some(value) = data {
            request.set_payload(Some(body(&value, &mut io::stdin().lock())?.into_bytes()));
        }
        Ok(request)
    }
}

struct Display((BufferedHttpResponse, bool, ColorMode));

impl fmt::Display for Display {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> std::result::Result<(), fmt::Error> {
        let Display((res, include_headers, colors)) = self;
        if *include_headers {
            writeln!(f, "HTTP/2 {}", res.status.to_string().bold())?;
            for (k, v) in &res.headers {
                writeln!(f, "{}: {}", k.to_string().dimmed(), v)?;
            }
            f.write_str("\n")?;
        }
        match str::from_utf8(&res.body) {
            Ok(body) if !body.is_empty() => {
                if res
                    .headers
                    .get("content-type")
                    .iter()
                    .any(|value| "application/json" == *value)
                {
                    match body.to_colored_json(*colors) {
                        Ok(colored) => write!(f, "{}", colored)?,
                        _ => f.write_str(body)?,
                    }
                } else {
                    f.write_str(body)?;
                }
            }
            _ => (),
        }

        Ok(())
    }
}

fn body<R>(
    value: &str,
    stdin: &mut R,
) -> io::Result<String>
where
    R: Read,
{
    match value {
        path if path.starts_with('@') && path.len() > 1 => match &path[1..] {
            "-" => {
                let mut buf = String::new();
                stdin.read_to_string(&mut buf)?;
                Ok(buf)
            }
            path => read_to_string(path),
        },
        raw => Ok(raw.to_string()),
    }
}

fn run(options: Options) -> Result<(), Box<dyn StdError>> {
    let Options {
        include_headers, ..
    } = options;
    let response = Client::shared()
        .sign_and_dispatch::<_, Error>(options.try_into()?, |response| {
            Box::new(response.buffer().from_err())
        })
        .sync()?;
    println!(
        "{}",
        Display((response, include_headers, ColorMode::default()))
    );
    Ok(())
}

fn main() {
    env_logger::init();
    if let Err(e) = run(Options::from_args()) {
        eprintln!("{}", e);
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{HeaderMap, StatusCode};
    use pretty_assertions::{assert_eq};

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

    #[test]
    fn body_can_be_raw_str() -> Result<(), Box<dyn StdError>> {
        assert_eq!(body("text", &mut &b""[..])?, "text");
        Ok(())
    }

    #[test]
    fn body_can_stdin() -> Result<(), Box<dyn StdError>> {
        assert_eq!(body("@-", &mut &b"stdin"[..])?, "stdin");
        Ok(())
    }

    #[test]
    fn body_can_be_read_from_file() -> Result<(), Box<dyn StdError>> {
        assert_eq!(
            body("@tests/data/file.json", &mut &b""[..])?,
            r#"{"hello":"aws"}"#
        );
        Ok(())
    }

    #[test]
    fn display_renders_basic_response() {
        colored::control::set_override(false);
        let response = BufferedHttpResponse {
            body: r#"{"hello":"aws"}"#.into(),
            headers: HeaderMap::default(),
            status: StatusCode::default(),
        };
        assert_eq!(
            Display((response, true, ColorMode::Off)).to_string(),
            indoc::indoc!(
                r#"HTTP/2 200 OK

                {"hello":"aws"}"#
            )
        )
    }

    #[test]
    fn display_renders_invalid_json_response() {
        colored::control::set_override(false);
        let mut headers: HeaderMap<String> = HeaderMap::default();
        headers.insert("Content-Type", "application/json".into());
        let response = BufferedHttpResponse {
            body: r#"helloaws"#.into(),
            headers,
            status: StatusCode::default(),
        };
        assert_eq!(
            Display((response, true, ColorMode::Off)).to_string(),
            indoc::indoc!(
                r#"HTTP/2 200 OK
                content-type: application/json

                helloaws"#
            )
        )
    }
}
