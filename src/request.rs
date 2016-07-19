
use std::collections::HashMap;
use std::net::TcpStream;
use std::io::Read;
//use std::time::Duration;

const BUF_SIZE: usize = 1024;

pub struct HTTPRequest {
    pub body: String,
    pub headers: HashMap<String, String>,
    pub method: String,
    pub path: String,
    pub protocol: String
}

impl HTTPRequest {
    pub fn new(mut stream: &mut TcpStream) -> Option<HTTPRequest> {
        let message = HTTPRequest::read(&mut stream)
            .expect("Could not read message!");

        let (header, body) = match message.find("\r\n\r\n") {
            Some(i) => message.split_at(i),
            None => return None
        };

        let mut contents: Vec<&str> = header.split("\r\n").collect();
        // First line of the incoming message should be a status line
        // ex: GET /path HTTP/1.1
        let status_line: Vec<&str> = contents.remove(0).split(' ').collect();
        let mut headers = HashMap::new();

        for line in contents {
            match line.find(':') {
                Some(_) => {
                    let parts: Vec<&str> = line.split(':').collect();
                    headers.insert(parts[0].to_string(), parts[1..].join(":").trim().to_string());
                },
                None => continue
            }
        }

        Some(HTTPRequest {
            body: body.to_string(),
            headers: headers,
            method: status_line[0].to_string(),
            path: status_line[1].to_string(),
            protocol: status_line[2].to_string(),
        })
    }

    pub fn new_empty() -> HTTPRequest {
        HTTPRequest {
            body: "".to_string(),
            headers: HashMap::new(),
            method: "".to_string(),
            path: "".to_string(),
            protocol: "".to_string(),
        }
    }


    fn read(stream: &mut TcpStream) -> Option<String> {
        let mut raw = Vec::new();

        //stream.set_read_timeout(Some(Duration::from_millis(2000)));

        loop {
            let mut bytes = [0; BUF_SIZE];
            let bytes_read = stream.read(&mut bytes).unwrap();
            if bytes_read == 0 {
                break;
            }

            let (data, _) = bytes.split_at(bytes_read);
            raw.extend_from_slice(data);

            if bytes_read < BUF_SIZE {
                break;
            }
        };

        match String::from_utf8(raw) {
            Ok(s) => Some(s),
            Err(_) => None
        }
    }
}
