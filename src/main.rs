#![feature(io)]
#![feature(net)]
#![feature(core)]
extern crate hyper;
extern crate url;


use std::io::Write;
use std::io::Read;
use std::net::IpAddr;

use hyper::Client;
use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::net::Fresh;
use hyper::uri::RequestUri;
use hyper::Url;
use hyper::header::Host;

use url::ParseError;

static HOST: &'static str = "open.tan.fr";

macro_rules! ret_err(
	($e:expr) => {{
		match $e {
			Ok(v) => v,
			Err(e) => { println!("Line {}: {}", line!(), e); return; }
		}
	}}
);

fn create_proxy_url(uri: RequestUri) -> Result<Url, ParseError> {
	match uri {
		RequestUri::AbsolutePath(val) => {
			let str = format!("https://{}{}", HOST, val);
			Url::parse(&str)
		},
		RequestUri::AbsoluteUri(val) => Ok(val),
		_ => Err(ParseError::InvalidScheme)
	}
}

fn handler(mut client_request: Request, mut server_response: Response<Fresh>) -> () {
	let mut proxy = Client::new();

	let mut body: Vec<u8> = Vec::new();
	ret_err!(client_request.read_to_end(&mut body));
	let mut headers = client_request.headers;
	headers.set(Host {
		hostname: HOST.to_string(),
		port: None
	});
	let method = client_request.method;
	let url = ret_err!(create_proxy_url(client_request.uri));
	let proxy_request = proxy.request(method, url)
		.headers(headers)
		.body(body.as_slice());

	let mut proxy_response = ret_err!(proxy_request.send());

	let mut body: Vec<u8> = Vec::with_capacity(1024 * 1024 * 16);
	ret_err!(proxy_response.read_to_end(&mut body));
	*server_response.status_mut() = proxy_response.status;
	*server_response.headers_mut() = proxy_response.headers;
	let mut server_response = ret_err!(server_response.start());
	ret_err!(server_response.write_all(body.as_slice()));
	ret_err!(server_response.end());
}

fn main() {
	let server = Server::http(handler);
	ret_err!(server.listen(IpAddr::new_v4(127, 0, 0, 1), 3000));
}
