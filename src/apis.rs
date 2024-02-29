use std::mem;

use bounded_integer::BoundedU64;
use serde::Deserialize;
use serde_repr::Serialize_repr;
use thiserror::Error;

pub mod economy;
pub mod games;
pub mod groups;
pub mod users;

type StrPairArray<'a, const N: usize> = [(&'a str, &'a str); N];

pub type Id = BoundedU64<1, { i64::MAX as u64 }>;

#[derive(Deserialize)]
#[serde(untagged)]
enum ResultDef<T, E> {
    Ok(T),
    Err(E),
}

#[derive(Deserialize)]
#[serde(from = "ResultDef<T, E>")]
pub(crate) struct UntaggedResult<T, E>(pub(crate) Result<T, E>);
impl<T, E> From<ResultDef<T, E>> for UntaggedResult<T, E> {
    fn from(result: ResultDef<T, E>) -> Self {
        match result {
            ResultDef::Ok(value) => Self(Ok(value)),
            ResultDef::Err(value) => Self(Err(value)),
        }
    }
}

pub type RequestResult<T> = Result<T, Error>;
pub(crate) type ApiResponse<T> = UntaggedResult<T, ApiError>;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Empty {}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("roblox api error: {0}")]
    Api(#[from] ApiError),

    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize, Clone, Default)]
struct ApiErrors {
    errors: [InnerApiError; 1],
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct InnerApiError {
    pub code: i8,
    pub message: String,
    pub user_facing_message: Option<String>,
    pub field: Option<String>,
}

#[derive(Debug, Deserialize, Error, Clone, Default)]
#[serde(rename_all = "camelCase", from = "ApiErrors")]
#[error("{code}: {}", Self::display_error_message(self))]
pub struct ApiError {
    pub code: i8,
    pub message: String,
    pub user_facing_message: Option<String>,
    pub field: Option<String>,
}
impl ApiError {
    #[must_use]
    pub fn display_error_message(&self) -> &String {
        self.user_facing_message.as_ref().unwrap_or(&self.message)
    }
}
impl From<ApiErrors> for ApiError {
    fn from(mut value: ApiErrors) -> Self {
        Self {
            code: value.errors[0].code,
            message: mem::take(&mut value.errors[0].message),
            user_facing_message: mem::take(&mut value.errors[0].user_facing_message),
            field: mem::take(&mut value.errors[0].field),
        }
    }
}
