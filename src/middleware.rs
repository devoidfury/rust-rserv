
use request::HTTPRequest;
use response::HTTPResponse;
use regex::Regex;

#[allow(dead_code)]
pub enum MResult {
    Next,
    End
}

pub struct Middleware {
    pub mount: Regex,
    pub handle: fn(&HTTPRequest, &mut HTTPResponse) -> MResult,
    pub index: usize
}


impl Middleware {
    pub fn new(mount: &str, handle: fn(&HTTPRequest, &mut HTTPResponse) -> MResult) -> Middleware {
        Middleware {handle: handle, mount: Regex::new(mount).unwrap(), index: 0}
    }
}

pub struct MiddlewareStack {
    pub stack: Vec<Middleware>
}

impl MiddlewareStack {
    pub fn new() -> MiddlewareStack {
        MiddlewareStack {stack: Vec::new()}
    }

    pub fn push(&mut self, mut mid: Middleware) {
        mid.index = self.stack.len();
        self.stack.push(mid);
    }

    pub fn query(&self, path: &str, mut index: usize) -> Option<&Middleware> {
        loop {
            match self.stack.get(index) {
                Some(r) => {
                    if r.mount.is_match(path) {
                        return Some(r)
                    }
                },
                None => return None
            };
            index += 1;
        }
    }
}
