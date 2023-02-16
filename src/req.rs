use std::time::Duration;

use reqwest::{header::CONTENT_TYPE, Client, Response};

use crate::api::ApiProblem;

pub(crate) type ReqResult<T> = std::result::Result<T, ApiProblem>;

lazy_static::lazy_static! {
    static ref HTTP_CLIENT: Client = Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create client");
}

pub(crate) async fn req_get(url: &str) -> Result<Response, reqwest::Error> {
    let req = HTTP_CLIENT.get(url);
    trace!("{:?}", req);
    req.send().await
}

pub(crate) async fn req_head(url: &str) -> Result<Response, reqwest::Error> {
    let req = HTTP_CLIENT.head(url);
    trace!("{:?}", req);
    req.send().await
}

pub(crate) async fn req_post(url: &str, body: &str) -> Result<Response, reqwest::Error> {
    let req = HTTP_CLIENT
        .post(url)
        .header(CONTENT_TYPE, "application/jose+json")
        .body(body.to_string());
    trace!("{:?} {}", req, body);
    req.send().await
}

pub(crate) async fn req_handle_error(
    resp: Result<Response, reqwest::Error>,
) -> ReqResult<Response> {
    match resp {
        Ok(resp) if resp.status().is_success() => Ok(resp),
        Ok(resp)
            if resp
                .headers()
                .get(CONTENT_TYPE)
                .map(|h| h == "application/problem+json")
                .unwrap_or(false) =>
        {
            // if we were sent a problem+json, deserialize it
            let body = resp.text().await.unwrap_or_default();
            let problem = serde_json::from_str(&body).unwrap_or_else(|e| ApiProblem {
                _type: "problemJsonFail".into(),
                detail: Some(format!(
                    "Failed to deserialize application/problem+json ({}) body: {}",
                    e.to_string(),
                    body
                )),
                subproblems: None,
            });

            Err(problem)
        }
        Ok(resp) => {
            let status = format!("{} {}", resp.status(), resp.status().to_string());
            let detail = format!("{} body: {}", status, resp.text().await.unwrap_or_default());

            Err(ApiProblem {
                _type: "httpReqError".into(),
                detail: Some(detail),
                subproblems: None,
            })
        }
        Err(_) => {
            // some other problem
            Err(ApiProblem {
                _type: "httpReqIoError".into(),
                detail: None,
                subproblems: None,
            })
        }
    }
}

pub(crate) fn req_expect_header(res: &Response, name: &str) -> ReqResult<String> {
    res.headers()
        .get(name)
        .map(|v| v.to_str().unwrap_or_default().to_string())
        .ok_or_else(|| ApiProblem {
            _type: format!("Missing header: {}", name),
            detail: None,
            subproblems: None,
        })
}
