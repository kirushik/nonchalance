use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_rustls;

lazy_static! {
    static ref HTTP_CLIENT: Client = Client::with_connector(HttpsConnector::new(hyper_rustls::TlsClient::new()));
}

pub fn get_status_code(url: &str, response_code: &mut String) {
    let res = HTTP_CLIENT.get(url).send().expect("failed to get a resource");
    *response_code = format!("{}", res.status);
}