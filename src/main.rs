
extern crate regex;
extern crate num_cpus;

mod request;
mod response;
mod middleware;
mod server;

use request::HTTPRequest;
use response::HTTPResponse;
use middleware::{Middleware, MiddlewareStack, MResult};
use server::RservApp;


fn route_home(_: &HTTPRequest, res: &mut HTTPResponse) -> MResult {
    res.set_header("Content-Type", "text/html");
    res.body = "Welcome! Try /nothere <img src='1'><img src='2'><img src='3'><img src='4'><img src='5'><img src='6'><img src='7'><img src='8'>".as_bytes();
    MResult::End
}

fn error_handler(_: &HTTPRequest, res: &mut HTTPResponse) {
    res.set_header("Content-Type", "text/plain");
    res.body = "it done broke".as_bytes()
}


fn main() {
    let mut mstack = MiddlewareStack::new();
    mstack.push(Middleware::new("^/?$", route_home));

    let mut app = RservApp::new(mstack);
    app.error_handler = error_handler;
    app.listen("127.0.0.1:8081");
}
