#![deny(
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic,
    clippy::correctness,
    clippy::nursery
)]
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use apis::RequestResult;
use async_trait::async_trait;
use reqwest::{IntoUrl, Method};
use serde::{de::DeserializeOwned, Serialize};

pub mod apis;
pub mod clients;

#[async_trait]
pub trait BaseClient: Sync {
    async fn request<'a, T: DeserializeOwned, U: Serialize, V: Serialize>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: impl Into<Option<U>> + Send,
        payload: impl Into<Option<V>> + Send,
    ) -> RequestResult<T>;
    async fn get<'a, T: DeserializeOwned, U: Serialize>(
        &self,
        url: impl IntoUrl + Send,
        query: impl Into<Option<U>> + Send,
    ) -> RequestResult<T> {
        self.request::<T, U, ()>(Method::GET, url, query, None)
            .await
    }
    async fn post<T: DeserializeOwned, U: Serialize>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T> {
        self.request::<T, (), U>(Method::POST, url, None, payload)
            .await
    }
    async fn put<T: DeserializeOwned, U: Serialize>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T> {
        self.request::<T, (), U>(Method::PUT, url, None, payload)
            .await
    }
    async fn patch<T: DeserializeOwned, U: Serialize>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T> {
        self.request::<T, (), U>(Method::PATCH, url, None, payload)
            .await
    }
    async fn delete<T: DeserializeOwned, U: Serialize>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T> {
        self.request::<T, (), U>(Method::DELETE, url, None, payload)
            .await
    }
}
#[async_trait]
impl<C: AuthenticatedClient> BaseClient for C {
    async fn request<'a, T: DeserializeOwned, U: Serialize, V: Serialize>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: impl Into<Option<U>> + Send,
        payload: impl Into<Option<V>> + Send,
    ) -> RequestResult<T> {
        self.authenticated_request(method, url, query, payload)
            .await
    }
}

#[async_trait]
pub trait AuthenticatedClient: Sync {
    async fn authenticated_request<'a, T: DeserializeOwned, U: Serialize, V: Serialize>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: impl Into<Option<U>> + Send,
        payload: impl Into<Option<V>> + Send,
    ) -> RequestResult<T>;
    async fn authenticated_get<'a, T: DeserializeOwned, U: Serialize>(
        &self,
        url: impl IntoUrl + Send,
        query: impl Into<Option<U>> + Send,
    ) -> RequestResult<T> {
        self.authenticated_request::<T, U, ()>(Method::GET, url, query, None)
            .await
    }
    async fn authenticated_post<T: DeserializeOwned, U: Serialize>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T> {
        self.authenticated_request::<T, (), U>(Method::POST, url, None, payload)
            .await
    }
    async fn authenticated_put<T: DeserializeOwned, U: Serialize>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T> {
        self.authenticated_request::<T, (), U>(Method::PUT, url, None, payload)
            .await
    }
    async fn authenticated_patch<T: DeserializeOwned, U: Serialize>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T> {
        self.authenticated_request::<T, (), U>(Method::PATCH, url, None, payload)
            .await
    }
    async fn authenticated_delete<T: DeserializeOwned, U: Serialize>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T> {
        self.authenticated_request::<T, (), U>(Method::DELETE, url, None, payload)
            .await
    }
}
