use async_trait::async_trait;
use derive_is_enum_variant::is_enum_variant;
use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;

use crate::BaseClient;

use super::{OptionId, RequestResult};

macro_rules! add_base_url {
    ($api_route: literal) => {
        concat!("https://thumbnails.roblox.com/", $api_route)
    };
    ($api_format_string: literal, $($args:expr),+) => {
        format!(concat!("https://thumbnails.roblox.com/", $api_format_string), $($args),+)
    };
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "crate::utils::option_id_is_none")]
    pub target_id: OptionId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub r#type: ThumbnailType,
    pub size: ThumbnailSize,
    #[serde(skip_serializing_if = "crate::utils::is_default")]
    pub format: ThumbnailFormat,
    #[serde(rename = "isCircular")]
    pub circular: bool,
}

#[derive(Debug, Serialize, Default, PartialEq, Eq)]
pub enum ThumbnailFormat {
    #[default]
    Png,
    Jpeg,
}

#[derive(Debug, Serialize, is_enum_variant)]
#[serde(rename_all = "camelCase")]
pub enum ThumbnailSize {
    _30x30,
    _42x42,
    _48x48,
    _50x50,
    _60x60,
    _60x62,
    _75x75,
    _100x100,
    _110x110,
    _140x140,
    _150x150,
    _180x180,
    _160x100,
    _160x600,
    _250x250,
    _256x144,
    _300x250,
    _352x352,
    _304x166,
    _384x216,
    _396x216,
    _420x420,
    _480x270,
    _512x512,
    _576x324,
    _700x700,
    _720x720,
    _728x90,
    _768x432,
    _1200x80,
    _256x256,
    _128x128,
}

#[derive(Debug, Serialize_repr, Clone, Copy)]
#[repr(u8)]
pub enum ThumbnailType {
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

#[derive(Debug, Deserialize)]
pub enum ThumbnailState {
    Completed,
    Blocked,
    Error,
}

#[derive(Debug, Deserialize)]
pub enum ThumbnailVersion {
    TN3,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchThumbnail {
    pub request_id: String,
    pub error_code: i8,
    pub error_message: String,
    #[serde(deserialize_with = "super::deserialize_zeroable_id")]
    pub target_id: OptionId,
    pub state: ThumbnailState,
    pub image_url: Option<String>,
    pub version: ThumbnailVersion,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchResponse {
    data: Vec<BatchThumbnail>,
}

#[async_trait]
pub trait ThumbnailsApi: BaseClient {
    async fn get_batch_thumbnails(
        &self,
        requests: &[BatchRequest],
    ) -> RequestResult<Vec<BatchThumbnail>> {
        let response = self
            .post::<BatchResponse, &[BatchRequest]>(add_base_url!("v1/batch"), requests)
            .await?;
        Ok(response.data)
    }
}

impl<T: BaseClient> ThumbnailsApi for T {}
