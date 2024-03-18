use axum::{extract::Request, http::Uri, middleware::Next, response::Response};

#[derive(Debug, Clone)]
pub struct RequestUri(Uri);

pub async fn uri_middleware(request: Request, next: Next) -> Response {
    let uri = request.uri().clone();

    let mut response = next.run(request).await;

    response.extensions_mut().insert(RequestUri(uri));

    response
}

pub fn logging_middleware(
    response: &Response,
    latency: std::time::Duration,
    _span: &tracing::Span,
) {
    let url = match response.extensions().get::<RequestUri>().map(|r| &r.0) {
        Some(uri) => uri.to_string(),
        None => "unknown".to_string(),
    };
    let status = response.status();
    // TODO add this function impl
    // let latency = utils::duration_to_ms_string(latency);
    let latency = format!("{:?}", latency);

    if url == "/healthcheck" {
        tracing::trace!("{} {} {}", url, status, latency);
        return;
    }

    tracing::debug!("{} {} {}", url, status, latency);
}
