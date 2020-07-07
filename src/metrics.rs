lazy_static! {

    pub static ref HANDLER_SECS: prometheus::HistogramVec = register_histogram_vec!(
        "quietbackend_handler_secs",
        "Seconds taken for each response, partitioned by endpoint name",
        &["endpoint_name"],
        vec![1.0, 2.0, 4.0, 16.0] // Prometheus buckets
    )
    .expect("couldn't make HANDLER_SECS");

    pub static ref RESPONSES: prometheus::IntCounterVec = register_int_counter_vec!(
        "quietbackend_responses",
        "How many responses of Ok/Err per endpoint",
        &["endpoint_name", "result"]
    )
    .expect("couldn't make RESPONSES");

    pub static ref HTTP_RESPONSES: prometheus::IntCounterVec = register_int_counter_vec!(
        "quietbackend_http_responses",
        "Count of each HTTP status code served by quietbackend responses",
        &["status"]
    )
    .expect("couldn't make HTTP_RESPONSES");
}

pub mod endpoint {
    use actix_web::{http, HttpRequest, HttpResponse};
    use prometheus::Encoder;

    pub fn gather(_req: HttpRequest) -> HttpResponse {
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = vec![];
        let metric_families = prometheus::gather();
        match encoder.encode(&metric_families, &mut buffer) {
            Ok(()) => HttpResponse::build(http::StatusCode::OK).body(buffer),
            Err(e) => {
                let message = format!("{:?}", e);
                HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).body(message)
            }
        }
    }
}
