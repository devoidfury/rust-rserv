
use std::net::TcpStream;
use std::collections::HashMap;
use std::io::Write;


pub struct HTTPResponse<'a> {
    pub body: &'a [u8],
    pub headers: HashMap<String, String>,
    pub status: u32,
    pub sent_headers: bool
}

impl<'a> HTTPResponse<'a> {
    pub fn new() -> HTTPResponse<'a> {
        let mut res = HTTPResponse {body: &[], headers: HashMap::new(), status: 200, sent_headers: false};
        res.set_header("Server", "rserv");
        res
    }

    pub fn set_header(&mut self, key: &str, val: &str) -> &mut HTTPResponse<'a> {
        self.headers.entry(key.to_string()).or_insert(val.to_string());
        self
    }

    pub fn end(&self, mut stream: TcpStream) {
        let status_text = self.status_text();
        stream.write_all(format!("HTTP/1.0 {} {}\r\n", self.status, status_text).as_bytes());
        let headtxt: String = self.headers.iter().map(&|(k, v)| format!("{}: {}\r\n", k, v)).collect();

        stream.write_all(headtxt.as_bytes());
        stream.write_all("\r\n".as_bytes());
        stream.write_all(self.body);
        stream.flush();
    }

    pub fn status_text(&self) -> String {
        match self.status {
            200 => "OKAY",
            400 => "WHAT",
            404 => "NOPE",
            500 => "OOPS",
            _ => "???"
        }.to_string()
    }
}
