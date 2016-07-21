use std::net::TcpStream;
use std::collections::HashMap;
use std::io;
use std::io::Write;


pub struct HTTPResponse<'a> {
    pub body: &'a [u8],
    pub headers: HashMap<&'a str, &'a str>,
    pub status: u32,
    pub sent_headers: bool
}

impl<'a> HTTPResponse<'a> {
    pub fn new() -> HTTPResponse<'a> {
        let mut res = HTTPResponse {body: &[], headers: HashMap::new(), status: 200, sent_headers: false};
        res.set_header("Server", "rserv");
        res
    }

    pub fn set_header(&mut self, key: &'a str, val: &'a str) -> &mut HTTPResponse<'a> {
        self.headers.entry(key).or_insert(val);
        self
    }

    pub fn end(&self, mut stream: TcpStream) -> Result<(), io::Error>{
        let status_text = self.status_text();
        let headtxt: String = self.headers.iter().map(&|(k, v)| format!("{}: {}\r\n", k, v)).collect();

        try!(stream.write_all(format!("HTTP/1.0 {} {}\r\n", self.status, status_text).as_bytes()));
        try!(stream.write_all(headtxt.as_bytes()));
        try!(stream.write_all("\r\n".as_bytes()));
        try!(stream.write_all(self.body));
        stream.flush()
    }

    pub fn status_text(&self) -> &str {
        match self.status {
            200 => "OKAY",
            400 => "WHAT",
            404 => "NOPE",
            500 => "OOPS",
            _ => "???"
        }
    }
}
