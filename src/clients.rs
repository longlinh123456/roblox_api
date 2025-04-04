use ahash::RandomState;
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use itertools::Itertools;
use reqwest::{
    Client as ReqwestClient, ClientBuilder as ReqwestClientBuilder, IntoUrl, Method,
    RequestBuilder, Url, cookie::CookieStore, header::HeaderValue,
};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

use crate::apis::{Error, RequestResult, RobloxError};
use crate::{AuthenticatedClient, BaseClient};

pub use reqwest::ClientBuilder;
pub use reqwest::Proxy;

const CSRF_TOKEN_HEADER: &str = "x-csrf-token";
const AUTHENTICATION_COOKIE_NAME: &str = ".roblosecurity";

#[derive(Default, Debug)]
struct StaticSharedJar(DashMap<String, String, RandomState>);
impl StaticSharedJar {
    fn new() -> Self {
        Self::default()
    }
    fn insert(&self, name: &str, value: &str) {
        self.0.insert(String::from(name), String::from(value));
    }
    fn remove(&self, name: &str) {
        self.0.remove(name);
    }
    fn get(&self, name: &str) -> Option<String> {
        self.0.get(name).map(|x| x.clone())
    }
    fn clear(&self) {
        self.0.clear();
    }
}
impl CookieStore for StaticSharedJar {
    fn cookies(&self, _url: &Url) -> Option<HeaderValue> {
        let cookie_string = self
            .0
            .iter()
            .map(|x| format!("{}={}", x.key(), x.value()))
            .join("; ");

        if cookie_string.is_empty() {
            None
        } else {
            HeaderValue::from_maybe_shared(Bytes::from(cookie_string)).ok()
        }
    }
    fn set_cookies(&self, _cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, _url: &Url) {}
}

#[derive(Debug, Clone, Default)]
pub struct Client {
    client: ReqwestClient,
    csrf_token: Arc<ArcSwapOption<HeaderValue>>,
}
impl Client {
    fn build_request(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize>,
        payload: Option<impl Serialize>,
        csrf_token: Option<&HeaderValue>,
    ) -> RequestBuilder {
        let is_get = matches!(method, Method::GET);
        let mut builder = self.client.request(method, url);
        if let Some(query) = query {
            builder = builder.query(&query);
        }
        builder = match payload {
            Some(payload) => builder.json(&payload),
            None => builder
                .body("")
                .header("Content-Length", 0)
                .header("Content-Type", "application/json"),
        };
        if let Some(csrf_token) = csrf_token {
            if !is_get {
                builder = builder.header(CSRF_TOKEN_HEADER, csrf_token);
            }
        }
        builder
    }
    pub async fn request<T: DeserializeOwned, E: RobloxError>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize + Send>,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        let old_csrf_token = self.csrf_token.load();
        let builder = self.build_request(method, url, query, payload, old_csrf_token.as_deref());
        let mut response = builder.try_clone().unwrap().send().await?;
        if let Some(csrf_token) = response.headers().get(CSRF_TOKEN_HEADER) {
            self.csrf_token.store(Some(Arc::new(csrf_token.to_owned())));
            response = builder.header(CSRF_TOKEN_HEADER, csrf_token).send().await?;
        }
        if response.status() == 429 {
            return Err(Error::RateLimit);
        }
        let res = response.text().await?;
        sonic_rs::from_str::<T>(&res).map_or_else(|_| Err(E::parse(res).into()), |value| Ok(value))
    }
    #[must_use]
    pub fn new(builder: ReqwestClientBuilder) -> Self {
        Self {
            client: builder.build().unwrap(),
            csrf_token: Arc::new(ArcSwapOption::const_empty()),
        }
    }
}

#[async_trait]
impl BaseClient for Client {
    async fn request<T: DeserializeOwned, E: RobloxError>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize + Send>,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.request(method, url, query, payload).await
    }
}

#[derive(Debug, Clone, Default)]
pub struct CookieClient {
    client: ReqwestClient,
    csrf_token: Arc<ArcSwapOption<HeaderValue>>,
    jar: Arc<StaticSharedJar>,
}
impl CookieClient {
    fn build_request(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize>,
        payload: Option<impl Serialize>,
        csrf_token: Option<&HeaderValue>,
    ) -> RequestBuilder {
        let is_get = matches!(method, Method::GET);
        let mut builder = self.client.request(method, url);
        if let Some(query) = query {
            builder = builder.query(&query);
        }
        builder = match payload {
            Some(payload) => builder.json(&payload),
            None => builder
                .body("")
                .header("Content-Length", 0)
                .header("Content-Type", "application/json"),
        };
        if let Some(csrf_token) = csrf_token {
            if !is_get {
                builder = builder.header(CSRF_TOKEN_HEADER, csrf_token);
            }
        }
        builder
    }
    pub async fn request<T: DeserializeOwned, E: RobloxError>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize + Send>,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        let old_csrf_token = self.csrf_token.load();
        let builder = self.build_request(method, url, query, payload, old_csrf_token.as_deref());
        let mut response = builder.try_clone().unwrap().send().await?;
        if let Some(csrf_token) = response.headers().get(CSRF_TOKEN_HEADER) {
            self.csrf_token.swap(Some(Arc::new(csrf_token.to_owned())));
            response = builder.header(CSRF_TOKEN_HEADER, csrf_token).send().await?;
        }
        if response.status() == 429 {
            return Err(Error::RateLimit);
        }
        let res = response.text().await?;
        sonic_rs::from_str::<T>(&res).map_or_else(|_| Err(E::parse(res).into()), |value| Ok(value))
    }
    #[must_use]
    pub fn new(builder: ReqwestClientBuilder, auth_cookie: &str) -> Self {
        let jar = Arc::new(StaticSharedJar::new());
        jar.insert(AUTHENTICATION_COOKIE_NAME, auth_cookie);
        Self {
            client: builder.cookie_provider(jar.clone()).build().unwrap(),
            csrf_token: Arc::new(ArcSwapOption::const_empty()),
            jar,
        }
    }
    #[inline]
    pub fn insert_cookie(&self, name: &str, value: &str) {
        self.jar.insert(name, value);
    }
    #[inline]
    pub fn remove_cookie(&self, name: &str) {
        self.jar.remove(name);
    }
    #[inline]
    #[must_use]
    pub fn get_cookie(&self, name: &str) -> Option<String> {
        self.jar.get(name)
    }
    #[inline]
    pub fn clear_cookies(&self) {
        self.jar.clear();
    }
    #[inline]
    pub fn set_auth_cookie(&self, cookie: &str) {
        self.jar.insert(".ROBLOSECURITY", cookie);
    }
}

#[async_trait]
impl AuthenticatedClient for CookieClient {
    #[inline]
    async fn authenticated_request<T: DeserializeOwned, E: RobloxError>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize + Send>,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.request(method, url, query, payload).await
    }
}
