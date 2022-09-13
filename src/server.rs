use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use hyper::service::{make_service_fn, service_fn};
use crate::request::Request;
use crate::response::Response;

pub async fn serve<T, F, Fut>(addr: SocketAddr, state: Arc<T>, op: F)
where
    T: Send + Sync + 'static,
    F: (Fn(Arc<T>, Request) -> Fut) + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<Response, Response>> + Send,
{
    let make_svc = make_service_fn(move |_conn| {
        let state = state.clone();
        let op = op.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |r| {
                let state = state.clone();
                let op = op.clone();
                async move {
                    Ok::<_, Infallible>(
                        match op(state, r.into()).await {
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

