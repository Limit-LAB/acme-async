use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::Result;

pub(crate) fn base64url<T: ?Sized + AsRef<[u8]>>(input: &T) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

pub(crate) async fn read_json<T: DeserializeOwned>(res: Response) -> Result<T> {
    Ok(res.json().await.map_err(|e| e.to_string())?)
}
