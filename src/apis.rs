use std::mem;

use async_stream::try_stream;
use chrono::NaiveDate;
use deranged::{OptionRangedU64, RangedU64};
use derive_is_enum_variant::is_enum_variant;
use futures::Stream;
use serde::{Deserialize, Deserializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;

use crate::private::RobloxErrorSealed;

pub mod economy;
pub mod games;
pub mod general;
pub mod groups;
pub mod thumbnails;
pub mod users;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Serialize_repr, Default, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
enum SortOrderDefaultAscending {
    #[default]
    Ascending = 1,
    Descending = 2,
}
impl From<SortOrder> for SortOrderDefaultAscending {
    fn from(value: SortOrder) -> Self {
        match value {
            SortOrder::Ascending => Self::Ascending,
            SortOrder::Descending => Self::Descending,
        }
    }
}

#[derive(Debug, Serialize_repr, Default, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
enum SortOrderDefaultDescending {
    Ascending = 1,
    #[default]
    Descending = 2,
}
impl From<SortOrder> for SortOrderDefaultDescending {
    fn from(value: SortOrder) -> Self {
        match value {
            SortOrder::Ascending => Self::Ascending,
            SortOrder::Descending => Self::Descending,
        }
    }
}

pub type Id = RangedU64<1, { i64::MAX as u64 }>;
pub type OptionId = OptionRangedU64<1, { i64::MAX as u64 }>;

#[derive(Deserialize, Default, Clone, Copy)]
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
    Ok(ZeroableId::deserialize(deserializer)?.into())
}

fn deserialize_date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDate, D::Error> {
    use serde::de::Error;
    let time = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&time, "%m/%d/%Y").map_err(D::Error::custom)
}

pub type RequestResult<T, E> = Result<T, Error<E>>;

#[derive(Deserialize, Default, Debug, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct Empty {}

pub trait RobloxError: RobloxErrorSealed + std::error::Error + Send {
    fn parse(res: String) -> Self;
}

#[derive(Debug, Default, Error)]
#[error("string api error: {message}")]
pub struct StringError {
    message: String,
}
impl RobloxErrorSealed for StringError {}
impl RobloxError for StringError {
    fn parse(res: String) -> Self {
        Self { message: res }
    }
}

#[derive(Debug, Default, Error, is_enum_variant)]
#[non_exhaustive]
pub enum Error<T: RobloxError> {
    #[error(transparent)]
    Api(#[from] T),

    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("rate limited")]
    #[default]
    RateLimit,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct JsonErrors {
    errors: [InnerJsonError; 1],
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct InnerJsonError {
    pub code: i8,
    pub message: String,
    pub user_facing_message: Option<String>,
    pub field: Option<String>,
}

#[derive(Debug, Deserialize, Error, Clone)]
#[serde(rename_all = "camelCase", from = "JsonErrors")]
#[error("{}", self.display_error_message())]
pub enum JsonError {
    Valid {
        code: i8,
        message: String,
        user_facing_message: Option<String>,
        field: Option<String>,
    },
    Malformed(String),
}
impl RobloxErrorSealed for JsonError {}
impl RobloxError for JsonError {
    fn parse(res: String) -> Self {
        sonic_rs::from_str::<Self>(&res).map_or(Self::Malformed(res), |value| value)
    }
}
impl JsonError {
    #[must_use]
    pub fn display_error_message(&self) -> String {
        match self {
            Self::Valid {
                user_facing_message,
                message,
                ..
            } => {
                format!(
                    "json error: {}",
                    user_facing_message.as_ref().unwrap_or(message)
                )
            }
            Self::Malformed(value) => {
                format!("malformed response: {value}")
            }
        }
    }
}
impl From<JsonErrors> for JsonError {
    fn from(mut value: JsonErrors) -> Self {
        Self::Valid {
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

#[derive(Debug, Deserialize_repr, Default, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum MembershipType {
    #[default]
    None = 0,
    BuildersClub = 1,
    TurboBuildersClub = 2,
    OutrageousBuildersClub = 3,
    Premium = 4,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub previous_page_cursor: Option<String>,
    pub next_page_cursor: Option<String>,
    pub data: Vec<T>,
}

pub fn paginate<T, R, E>(
    mut request: R,
    cursor: Option<impl Into<String>>,
) -> impl Stream<Item = RequestResult<Page<T>, E>>
where
    R: AsyncFnMut(Option<&'_ str>) -> RequestResult<Page<T>, E>,
    E: RobloxError,
{
    try_stream! {
        let mut cursor: Option<String> = cursor.map(Into::into);
        loop {
            let response = request(cursor.as_deref()).await?;
            if response.next_page_cursor.is_none() {
                yield response;
                break;
            }
            cursor.clone_from(&response.next_page_cursor);
            yield response;
        }
    }
}
