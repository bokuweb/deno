// Copyright 2018-2019 the Deno authors. All rights reserved. MIT license.

use futures::{future, Future};

use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use std::io;
use std::path::Path;

static NOTFOUND: &[u8] = b"Not Found";
static PORT: u16 = 4545;

fn main() {
    let addr = ([127, 0, 0, 1], PORT).into();
    let server = Server::bind(&addr)
        .serve(|| service_fn(response_examples))
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}

type ResponseFuture = Box<Future<Item = Response<Body>, Error = io::Error> + Send>;

fn response_examples(req: Request<Body>) -> ResponseFuture {
    match (req.method(), req.uri().path()) {
        (&Method::GET, _) => {
            let path = Path::new(req.uri().path());
            response_file(
                path.strip_prefix("/")
                    .expect("strip path prefix")
                    .to_str()
                    .expect("to str"),
            )
        }
        _ => Box::new(future::ok(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap(),
        )),
    }
}

fn response_file(f: &str) -> ResponseFuture {
    let filename = f.to_string(); // we need to copy for lifetime issues
    Box::new(
        tokio_fs::file::File::open(filename)
            .and_then(|file| {
                let buf: Vec<u8> = Vec::new();
                tokio_io::io::read_to_end(file, buf)
                    .and_then(|item| Ok(Response::new(item.1.into())))
                    .or_else(|_| {
                        Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::empty())
                            .unwrap())
                    })
            }).or_else(|_| {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(NOTFOUND.into())
                    .unwrap())
            }),
    )
}