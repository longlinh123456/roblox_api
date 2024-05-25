#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

use apis::{RequestResult, RobloxError};
use async_trait::async_trait;
use reqwest::{IntoUrl, Method};
use serde::{de::DeserializeOwned, Serialize};

pub mod apis;
pub mod clients;
pub(crate) mod private;
pub(crate) mod utils;

#[async_trait]
pub trait BaseClient: Sync {
    async fn request<'a, T: DeserializeOwned, U: Serialize, V: Serialize, E: RobloxError>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: impl Into<Option<U>> + Send,
        payload: impl Into<Option<V>> + Send,
    ) -> RequestResult<T, E>;
    #[inline]
    async fn get<'a, T: DeserializeOwned, U: Serialize, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        query: impl Into<Option<U>> + Send,
    ) -> RequestResult<T, E> {
        self.request::<T, U, (), E>(Method::GET, url, query, None)
            .await
    }
    #[inline]
    async fn post<T: DeserializeOwned, U: Serialize, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T, E> {
        self.request::<T, (), U, E>(Method::POST, url, None, payload)
            .await
    }
    #[inline]
    async fn put<T: DeserializeOwned, U: Serialize, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T, E> {
        self.request::<T, (), U, E>(Method::PUT, url, None, payload)
            .await
    }
    #[inline]
    async fn patch<T: DeserializeOwned, U: Serialize, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T, E> {
        self.request::<T, (), U, E>(Method::PATCH, url, None, payload)
            .await
    }
    #[inline]
    async fn delete<T: DeserializeOwned, U: Serialize, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T, E> {
        self.request::<T, (), U, E>(Method::DELETE, url, None, payload)
            .await
    }
}
#[async_trait]
impl<C: AuthenticatedClient> BaseClient for C {
    #[inline]
    async fn request<'a, T: DeserializeOwned, U: Serialize, V: Serialize, E: RobloxError>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: impl Into<Option<U>> + Send,
        payload: impl Into<Option<V>> + Send,
    ) -> RequestResult<T, E> {
        self.authenticated_request(method, url, query, payload)
            .await
    }
}

#[async_trait]
pub trait AuthenticatedClient: Sync {
    async fn authenticated_request<
        'a,
        T: DeserializeOwned,
        U: Serialize,
        V: Serialize,
        E: RobloxError,
    >(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: impl Into<Option<U>> + Send,
        payload: impl Into<Option<V>> + Send,
    ) -> RequestResult<T, E>;
    #[inline]
    async fn authenticated_get<'a, T: DeserializeOwned, U: Serialize, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        query: impl Into<Option<U>> + Send,
    ) -> RequestResult<T, E> {
        self.authenticated_request::<T, U, (), E>(Method::GET, url, query, None)
            .await
    }
    #[inline]
    async fn authenticated_post<T: DeserializeOwned, U: Serialize, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T, E> {
        self.authenticated_request::<T, (), U, E>(Method::POST, url, None, payload)
            .await
    }
    #[inline]
    async fn authenticated_put<T: DeserializeOwned, U: Serialize, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T, E> {
        self.authenticated_request::<T, (), U, E>(Method::PUT, url, None, payload)
            .await
    }
    #[inline]
    async fn authenticated_patch<T: DeserializeOwned, U: Serialize, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T, E> {
        self.authenticated_request::<T, (), U, E>(Method::PATCH, url, None, payload)
            .await
    }
    #[inline]
    async fn authenticated_delete<T: DeserializeOwned, U: Serialize, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: impl Into<Option<U>> + Send,
    ) -> RequestResult<T, E> {
        self.authenticated_request::<T, (), U, E>(Method::DELETE, url, None, payload)
            .await
    }
}
