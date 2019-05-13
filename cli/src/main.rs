use futures::future::Future;
use rusoto_core::{
    credential::CredentialsError, request::HttpDispatchError, signature::SignedRequest, Client,
    Region,
};
use structopt::StructOpt;

#[derive(Debug)]
enum Error {
    Dispatch(HttpDispatchError),
    Credentials(CredentialsError),
}

impl From<HttpDispatchError> for Error {
    fn from(err: HttpDispatchError) -> Self {
        Error::Dispatch(err)
    }
}

impl From<CredentialsError> for Error {
    fn from(err: CredentialsError) -> Self {
        Error::Credentials(err)
    }
}

#[derive(StructOpt)]
#[structopt(name = "sigv4", about = "sign aws sigv4 requests like a prod")]
struct Options {
    #[structopt(short = "r", long = "region", default_value = "us-east-1")]
    region: String,
    #[structopt(short = "s", long = "service", default_value = "execute-api")]
    service: String,
    #[structopt(short = "X", long = "request", default_value = "GET")]
    method: String,
    #[structopt(short = "H", long = "header")]
    headers: Vec<String>,
    #[structopt(short = "d", long = "data")]
    data: Option<String>,
    uri: String,
}

fn main() {
    let Options {
        region,
        service,
        method,
        headers,
        data,
        uri,
    } = Options::from_args();
    let region = Region::Custom {
        name: region,
        endpoint: uri,
    };
    let mut request = SignedRequest::new(&method, &service, &region, Default::default());
    for header in headers {
        if let [key, value] = &header.splitn(2, ':').collect::<Vec<_>>()[..] {
            request.add_header(key.trim(), value.trim())
        }
    }
    request.set_payload(data.map(String::into_bytes));
    let response = Client::shared()
        .sign_and_dispatch::<_, Error>(request, |response| Box::new(response.buffer().from_err()))
        .sync();

    println!("{:?}", response);
}
