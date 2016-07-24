use std::iter;
use std::thread;
use std::sync::Arc;
use std::net::{TcpListener, TcpStream, Shutdown};
use request::HTTPRequest;
use response::HTTPResponse;
use middleware::{MiddlewareStack, MResult};
use std::cmp;
use num_cpus;

use std::sync::mpsc::{channel, sync_channel, Sender};

pub struct Worker {
    jobs: usize,
    tx: Sender<TcpStream>,
    idx: usize
}


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
        let shared_app = Arc::new(self);

        let num_workers = cmp::max(num_cpus::get() - 1, 1);
        println!("[main] starting {} workers", num_workers);

        let mut workers = vec![];

        // this channel is for tracking when threads finish a job
        let (worker_done_tx, worker_done_rx) = channel();

        for worker_idx in 0 .. num_workers {
            // this channel, for each worker thread, is for sending the
            // TcpStream from the main thread to the worker
            let (worker_tx, worker_rx) = channel();

            let app = shared_app.clone();
            let done_tx = worker_done_tx.clone();

            thread::spawn(move|| {
                loop {
                    let stream = worker_rx.recv().unwrap();
                    println!("[worker {}] got stream, handling", worker_idx);
                    handle_incoming(&app, stream);
                    println!("[worker {}] done with a job", worker_idx);
                    done_tx.send(worker_idx);
                }
            });
            workers.push(Worker { jobs: 0, tx: worker_tx, idx: worker_idx })
        }

        let mut cycler = (0..num_workers).cycle();

        let listener = TcpListener::bind(address).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("[main] got conn");

                    println!("[main] sweep threads of completed jobs");
                    loop {
                        match worker_done_rx.try_recv() {
                            Ok(worker_idx) => {
                                println!("[main] worker {} finsihed a job", worker_idx);
                                //workers[worker_idx].jobs -= 1;
                            },
                            _ => break
                        };
                    }

                    let idx = cycler.next().unwrap();
                    println!("[main] sending to {}", idx);
                    //workers[idx].jobs += 1;
                    workers[idx].tx.send(stream).unwrap();
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
            }.as_bytes();
        }
    }
}

fn handle_incoming(app: &Arc<RservApp>, mut stream: TcpStream) {
    let _ = stream.set_nodelay(true);

    let mut res = HTTPResponse::new();

    let (req, is_valid) = match HTTPRequest::new(&mut stream) {
        Some(r) => (r, true),
        None => {
            let req = HTTPRequest::new_empty();
            res.status = 400;
            (app.error_handler)(&req, &mut res);
            (req, false)
        }
    };

    if is_valid {
        let mut index: usize = 0;
        loop {
            let route = match app.mstack.query(req.path.as_ref(), index) {
                Some(r) => r,
                None => {
                    res.status = 404;
                    (app.error_handler)(&req, &mut res);
                    break
                }
            };

            let handle = route.handle;
            index = match handle(&req, &mut res) {
                MResult::End => break,
                MResult::Next => route.index,
                MResult::Err => {
                    (app.error_handler)(&req, &mut res);
                    break
                }
            };
        }
    }

    let _ = res.end(stream);
}
