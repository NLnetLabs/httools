use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use hyper::service::{make_service_fn, service_fn};
use crate::request::Request;
use crate::response::Response;


pub async fn serve<H: Handler>(handler: H, addr: SocketAddr) {
    let make_svc = make_service_fn(move |_conn| {
        let this = handler.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |r| {
                let this = this.clone();
                async move {
                    Ok::<_, Infallible>(
                        match this.handle(r.into()).await {
                            Ok(resp) => resp,
                            Err(resp) => resp
                        }.into_hyper()
                    )
                }
            }))
        }
    });

    let server = hyper::Server::bind(&addr).serve(make_svc);

    // Run this server for ... ever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}


pub trait Handler: Clone + Send + Sync + 'static {
    type Future: Future<Output = Result<Response, Response>> + Send;

    fn handle(self, request: Request) -> Self::Future;
}


