use std::mem;

use async_stream::try_stream;
use deranged::{OptionRangedU64, RangedU64};
use futures::{future::BoxFuture, stream::BoxStream, Future};
use serde::{Deserialize, Deserializer};
use serde_repr::Serialize_repr;
use thiserror::Error;

pub mod economy;
pub mod games;
pub mod groups;
pub mod thumbnails;
pub mod users;

type StrPairArray<'a, const N: usize> = [(&'a str, &'a str); N];

#[derive(Debug, Serialize_repr, Default, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum SortOrder {
    Ascending = 1,
    #[default]
    Descending = 2,
}

pub type Id = RangedU64<1, { i64::MAX as u64 }>;
pub type OptionId = OptionRangedU64<1, { i64::MAX as u64 }>;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(transparent)]
struct ZeroableId(OptionRangedU64<0, { i64::MAX as u64 }>);
impl From<ZeroableId> for OptionId {
    fn from(value: ZeroableId) -> Self {
        let inner_value = value.0.get();
        inner_value.map_or(Self::None, |value| {
            value.narrow::<1, { i64::MAX as u64 }>().into()
        })
    }
}

fn deserialize_zeroable_id<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<OptionId, D::Error> {
    let res = ZeroableId::deserialize(deserializer);
    println!("{res:?}");
    Ok(res?.into())
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum ResultDef<T, E> {
    Ok(T),
    Err(E),
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Deserialize, Debug, Clone, Copy)]
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

#[derive(Debug, Serialize_repr, Default, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum RequestLimit {
    #[default]
    Ten = 10,
    TwentyFive = 25,
    Fifty = 50,
    OneHundred = 100,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub previous_page_cursor: Option<String>,
    pub next_page_cursor: Option<String>,
    pub data: Vec<T>,
}

pub type Paginator<'a, T> = BoxStream<'a, RequestResult<Page<T>>>;

pub type RequestFuture<'a, T> = BoxFuture<'a, RequestResult<Page<T>>>;

fn paginate<'a, T, S, Fut, R>(mut request: R, cursor: impl Into<Option<S>>) -> Paginator<'a, T>
where
    T: Unpin + Send + 'a,
    Fut: Future<Output = RequestResult<Page<T>>> + Send,
    R: 'a + FnMut(Option<String>) -> Fut + Send,
    S: Into<String>,
{
    let mut cursor: Option<String> = cursor.into().map(Into::into);
    Box::pin(try_stream! {
        loop {
            let response = request(cursor.clone()).await?;
            if response.next_page_cursor.is_none() {
                break;
            };
            cursor = response.next_page_cursor.clone();
            yield response;
        }
    })
}
