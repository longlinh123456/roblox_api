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
    async fn request<T: DeserializeOwned, E: RobloxError>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize + Send>,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E>;
    #[inline]
    async fn get<T: DeserializeOwned, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.request(Method::GET, url, query, None::<()>).await
    }
    #[inline]
    async fn post<T: DeserializeOwned, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.request(Method::POST, url, None::<()>, payload).await
    }
    #[inline]
    async fn put<T: DeserializeOwned, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.request(Method::PUT, url, None::<()>, payload).await
    }
    #[inline]
    async fn patch<T: DeserializeOwned, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.request(Method::PATCH, url, None::<()>, payload).await
    }
    #[inline]
    async fn delete<T: DeserializeOwned, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.request(Method::DELETE, url, None::<()>, payload).await
    }
}
#[async_trait]
impl<C: AuthenticatedClient> BaseClient for C {
    #[inline]
    async fn request<T: DeserializeOwned, E: RobloxError>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize + Send>,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.authenticated_request(method, url, query, payload)
            .await
    }
}

#[async_trait]
pub trait AuthenticatedClient: Sync {
    async fn authenticated_request<T: DeserializeOwned, E: RobloxError>(
        &self,
        method: Method,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize + Send>,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E>;
    #[inline]
    async fn authenticated_get<T: DeserializeOwned, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        query: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.authenticated_request(Method::GET, url, query, None::<()>)
            .await
    }
    #[inline]
    async fn authenticated_post<T: DeserializeOwned, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.authenticated_request(Method::POST, url, None::<()>, payload)
            .await
    }
    #[inline]
    async fn authenticated_put<T: DeserializeOwned, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.authenticated_request(Method::PUT, url, None::<()>, payload)
            .await
    }
    #[inline]
    async fn authenticated_patch<T: DeserializeOwned, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.authenticated_request(Method::PATCH, url, None::<()>, payload)
            .await
    }
    #[inline]
    async fn authenticated_delete<T: DeserializeOwned, E: RobloxError>(
        &self,
        url: impl IntoUrl + Send,
        payload: Option<impl Serialize + Send>,
    ) -> RequestResult<T, E> {
        self.authenticated_request(Method::DELETE, url, None::<()>, payload)
            .await
    }
}
