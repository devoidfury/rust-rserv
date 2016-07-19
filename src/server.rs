
use std::thread;
use std::sync::Arc;
use std::net::{TcpListener, TcpStream};
use request::HTTPRequest;
use response::HTTPResponse;
use middleware::{Middleware, MiddlewareStack, MResult};


pub struct RservApp {
    mstack: MiddlewareStack,
    pub error_handler: fn(&HTTPRequest, &mut HTTPResponse),
}

impl RservApp {
    pub fn new(mstack: MiddlewareStack) -> RservApp {
        RservApp {
            mstack: mstack,
            error_handler: RservApp::default_error_handler
        }
    }

    pub fn listen(self, address: &str) {
        // accept connections and process them, spawning a new thread for each one
        let listener = TcpListener::bind(address).unwrap();
        let shared_app = Arc::new(self);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let app = shared_app.clone();
                    thread::spawn(move|| { // connection succeeded
                        handle_incoming(app, stream)
                    });
                }
                Err(e) => { println!("Error: {}", e); }
            }
        }
    }

    fn default_error_handler(_: &HTTPRequest, res: &mut HTTPResponse) {
        if res.body.len() == 0 {
            res.set_header("Content-Type", "text/plain");
            res.body = match res.status {
                400 => "ERROR IN MESSAGE FORMAT",
                404 => "NOTHING HERE, GO AWAY",
                _ => "AN UNKNOWN ERROR OCCURRED. HOW ABOUT THAT?"
            }.as_bytes()
        }
    }
}

fn handle_incoming(app: Arc<RservApp>, mut stream: TcpStream) {
    let mstack = &app.mstack;
    let _ = stream.set_nodelay(true);

    let mut res = HTTPResponse::new();
    let req = match HTTPRequest::new(&mut stream) {
        Some(r) => r,
        None => {
            let req = HTTPRequest::new_empty();
            res.status = 400;
            let error_handler = app.error_handler;
            error_handler(&req, &mut res);
            res.end(stream);
            return
        }
    };

    let mut handled = false;
    let mut index: usize = 0;
    loop {
        let route = match mstack.query(req.path.as_ref(), index) {
            Some(r) => r,
            None => {
                res.status = 404;
                let error_handler = app.error_handler;
                error_handler(&req, &mut res);
                res.end(stream);
                return
            }
        };

        let handle = route.handle;
        match handle(&req, &mut res) {
            MResult::Next => {
                index = route.index;
            },
            MResult::End => {
                handled = true;
                break
            }
        };
    }
    if handled != true {
        res.status = 404;
    }
    res.end(stream);
}
