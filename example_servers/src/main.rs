use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use tokio::time::sleep;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;

async fn health_check(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Healthy")))
}

async fn work(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Simulate work by delaying for 10 milliseconds
    sleep(Duration::from_millis(10000)).await;
    Ok(Response::new(Body::from("Work done")))
}

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match req.uri().path() {
        "/health" => health_check(req).await,
        "/work" => work(req).await,
        _ => Ok(Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap()),
    }
}

async fn run_server(addr: SocketAddr) {
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(router)) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

#[tokio::main]
async fn main() {
    let addrs = vec![
        ([127, 0, 0, 1], 5000).into(),
        ([127, 0, 0, 1], 5001).into(),
        ([127, 0, 0, 1], 5002).into(),
        ([127, 0, 0, 1], 5003).into(),
    ];

    let mut handles = vec![];

    for addr in addrs {
        let handle = tokio::spawn(async move {
            run_server(addr).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}
