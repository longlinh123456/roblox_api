use async_trait::async_trait;
use derive_is_enum_variant::is_enum_variant;
use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use thiserror::Error;

use crate::{BaseClient, private::BatchThumbnailResultExtSealed};

use super::{JsonError, OptionId, RequestResult};

macro_rules! add_base_url {
    ($api_route: literal) => {
        concat!("https://thumbnails.roblox.com/", $api_route)
    };
    ($api_format_string: expr, $($args:expr),+) => {
        format!(concat!("https://thumbnails.roblox.com/", $api_format_string), $($args),+)
    };
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequest<T1: Send, T2: Send, T3: Send> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<T1>,
    #[serde(skip_serializing_if = "crate::utils::option_id_is_none")]
    pub target_id: OptionId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<T2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<T3>,
    pub r#type: ThumbnailType,
    pub size: ThumbnailSize,
    #[serde(skip_serializing_if = "crate::utils::is_default")]
    pub format: ThumbnailFormat,
    #[serde(rename = "isCircular", skip_serializing_if = "crate::utils::is_false")]
    pub circular: bool,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq, is_enum_variant)]
pub enum ThumbnailFormat {
    #[default]
    Webp,
    Png,
    Jpeg,
}

#[derive(Debug, Serialize, Default, is_enum_variant, Clone, Copy)]
pub enum ThumbnailSize {
    #[default]
    #[serde(rename = "30x30")]
    _30x30,
    #[serde(rename = "42x42")]
    _42x42,
    #[serde(rename = "48x48")]
    _48x48,
    #[serde(rename = "50x50")]
    _50x50,
    #[serde(rename = "60x60")]
    _60x60,
    #[serde(rename = "60x62")]
    _60x62,
    #[serde(rename = "75x75")]
    _75x75,
    #[serde(rename = "100x100")]
    _100x100,
    #[serde(rename = "110x110")]
    _110x110,
    #[serde(rename = "140x140")]
    _140x140,
    #[serde(rename = "150x150")]
    _150x150,
    #[serde(rename = "180x180")]
    _180x180,
    #[serde(rename = "160x100")]
    _160x100,
    #[serde(rename = "160x600")]
    _160x600,
    #[serde(rename = "250x250")]
    _250x250,
    #[serde(rename = "256x144")]
    _256x144,
    #[serde(rename = "300x250")]
    _300x250,
    #[serde(rename = "352x352")]
    _352x352,
    #[serde(rename = "304x166")]
    _304x166,
    #[serde(rename = "384x216")]
    _384x216,
    #[serde(rename = "396x216")]
    _396x216,
    #[serde(rename = "420x420")]
    _420x420,
    #[serde(rename = "480x270")]
    _480x270,
    #[serde(rename = "512x512")]
    _512x512,
    #[serde(rename = "576x324")]
    _576x324,
    #[serde(rename = "700x700")]
    _700x700,
    #[serde(rename = "720x720")]
    _720x720,
    #[serde(rename = "728x90")]
    _728x90,
    #[serde(rename = "768x432")]
    _768x432,
    #[serde(rename = "1200x80")]
    _1200x80,
    #[serde(rename = "256x256")]
    _256x256,
    #[serde(rename = "128x128")]
    _128x128,
}

#[derive(Debug, Default, Serialize_repr, Clone, Copy)]
#[repr(u8)]
pub enum ThumbnailType {
    #[default]
    Avatar = 1,
    AvatarHeadShot = 2,
    GameIcon = 3,
    BadgeIcon = 4,
    GameThumbnail = 5,
    GamePass = 6,
    Asset = 7,
    BundleThumbnail = 8,
    Outfit = 9,
    GroupIcon = 10,
    DeveloperProduct = 11,
    AutoGeneratedAsset = 12,
    AvatarBust = 13,
    PlaceIcon = 14,
    AutoGeneratedGameIcon = 15,
    ForceAutoGeneratedGameIcon = 16,
    Look = 17,
}

#[derive(Debug, Default, Deserialize, Clone, Copy, is_enum_variant)]
enum ThumbnailState {
    #[default]
    Completed,
    Blocked,
    Error,
    InReview,
    Pending,
    TemporarilyUnavailable,
}
#[derive(Debug, Default, Clone, Copy, is_enum_variant)]
pub enum ThumbnailErrorState {
    #[default]
    Error,
    Blocked,
    InReview,
    Pending,
    TemporarilyUnavailable,
}

#[allow(clippy::fallible_impl_from)]
impl From<ThumbnailState> for ThumbnailErrorState {
    fn from(value: ThumbnailState) -> Self {
        match value {
            ThumbnailState::Completed => {
                panic!("successful thumbnail request should not be converted into an error")
            }
            ThumbnailState::Blocked => Self::Blocked,
            ThumbnailState::Error => Self::Error,
            ThumbnailState::InReview => Self::InReview,
            ThumbnailState::Pending => Self::Pending,
            ThumbnailState::TemporarilyUnavailable => Self::TemporarilyUnavailable,
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone, Copy)]
pub enum ThumbnailVersion {
    #[default]
    TN3,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct InnerBatchThumbnail {
    request_id: Option<String>,
    error_code: i8,
    error_message: String,
    #[serde(deserialize_with = "super::deserialize_zeroable_id")]
    target_id: OptionId,
    state: ThumbnailState,
    image_url: Option<String>,
    version: Option<ThumbnailVersion>,
}

pub type BatchThumbnailResult = Result<BatchThumbnail, BatchThumbnailError>;

#[derive(Debug, Default, Clone)]
pub struct BatchThumbnail {
    pub request_id: Option<String>,
    pub target_id: OptionId,
    pub version: ThumbnailVersion,
    pub image_url: String,
}

#[derive(Debug, Default, Clone, Error)]
#[error("error in requesting batch thumbnail: {self:?}")]
pub struct BatchThumbnailError {
    pub request_id: Option<String>,
    pub target_id: OptionId,
    pub error_code: i8,
    pub error_message: String,
    pub state: ThumbnailErrorState,
}

pub trait BatchThumbnailResultExt: BatchThumbnailResultExtSealed {
    fn request_id(&self) -> Option<&str>;
    fn target_id(&self) -> OptionId;
}

impl BatchThumbnailResultExtSealed for BatchThumbnailResult {}

impl BatchThumbnailResultExt for BatchThumbnailResult {
    fn request_id(&self) -> Option<&str> {
        match self {
            Ok(thumbnail) => thumbnail.request_id.as_deref(),
            Err(thumbnail) => thumbnail.request_id.as_deref(),
        }
    }
    fn target_id(&self) -> OptionId {
        match self {
            Ok(thumbnail) => thumbnail.target_id,
            Err(thumbnail) => thumbnail.target_id,
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<InnerBatchThumbnail> for BatchThumbnailResult {
    fn from(value: InnerBatchThumbnail) -> Self {
        match value.state {
            ThumbnailState::Completed => Ok(BatchThumbnail {
                request_id: value.request_id,
                target_id: value.target_id,
                version: value.version.unwrap(),
                image_url: value.image_url.unwrap(),
            }),
            _ => Err(BatchThumbnailError {
                request_id: value.request_id,
                target_id: value.target_id,
                error_code: value.error_code,
                error_message: value.error_message,
                state: value.state.into(),
            }),
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BatchResponse {
    data: Vec<InnerBatchThumbnail>,
}

#[derive(Serialize, Default)]
struct BatchRequestArray<
    T1: Send + Serialize,
    T2: Send + Serialize,
    T3: Send + Serialize,
    I: Iterator<Item = BatchRequest<T1, T2, T3>> + Clone,
>(#[serde(with = "serde_iter::seq")] I);

#[async_trait]
pub trait ThumbnailsApi: BaseClient {
    /// Limit of 100 thumbnails/request
    ///
    /// Rate limit: 50 requests/1.5s
    async fn get_batch_thumbnails<'a, I, T1, T2, T3>(
        &self,
        requests: I,
    ) -> RequestResult<Vec<BatchThumbnailResult>, JsonError>
    where
        T1: Serialize + Send,
        T2: Serialize + Send,
        T3: Serialize + Send,
        I: IntoIterator<Item = BatchRequest<T1, T2, T3>> + Send,
        I::IntoIter: Send + Clone,
    {
        let res = self
            .post::<BatchResponse, _>(
                add_base_url!("v1/batch"),
                Some(BatchRequestArray(requests.into_iter())),
            )
            .await?;
        Ok(res.data.into_iter().map(Into::into).collect())
    }
}

impl<T: BaseClient> ThumbnailsApi for T {}
