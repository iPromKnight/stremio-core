use crate::{
    runtime::{ConditionalSend, Env, TryEnvFuture},
    types::api::{APIResult, FetchRequestParams},
};

use http::{HeaderValue, Request};
use serde::{Deserialize, Serialize};
use crate::constants::API_KEY_AUTH_HEADER;

pub fn fetch_api<
    E: Env,
    BODY: Serialize + ConditionalSend + 'static,
    REQ: FetchRequestParams<BODY> + Clone + Serialize,
    RESP: for<'de> Deserialize<'de> + ConditionalSend + 'static,
>(
    api_request: &REQ,
) -> TryEnvFuture<APIResult<RESP>> {
    let api_key = E::api_key();

    let mut url = E::api_endpoint()
        .join("api/")
        .expect("url builder failed")
        .join(&api_request.version_path())
        .expect("url builder failed");

    url.set_query(api_request.query().as_deref());
    let request_builder = Request::builder()
        .method(api_request.method())
        .uri(url.as_str());

    let mut request = request_builder
        .body(api_request.to_owned().body())
        .expect("request builder failed");

    if let Some(key) = api_key {
        request.headers_mut().insert(
            API_KEY_AUTH_HEADER,
            HeaderValue::from_str(&key).expect("invalid header value"),
        );
    }

    E::fetch::<_, _>(request)
}