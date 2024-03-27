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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<&'a str>,
    #[serde(skip_serializing_if = "crate::utils::option_id_is_none")]
    pub target_id: OptionId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<&'a str>,
    pub r#type: ThumbnailType,
    pub size: ThumbnailSize,
    #[serde(skip_serializing_if = "crate::utils::is_default")]
    pub format: ThumbnailFormat,
    #[serde(rename = "isCircular", skip_serializing_if = "crate::utils::is_false")]
    pub circular: bool,
}

#[derive(Debug, Serialize, Default, PartialEq, Eq, Clone, Copy)]
pub enum ThumbnailFormat {
    #[default]
    Png,
    Jpeg,
}

#[derive(Debug, Serialize, is_enum_variant, Clone, Copy)]
pub enum ThumbnailSize {
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

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum ThumbnailState {
    Completed,
    Blocked,
    Error,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum ThumbnailVersion {
    TN3,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchThumbnail {
    pub request_id: Option<String>,
    pub error_code: i8,
    pub error_message: String,
    #[serde(deserialize_with = "super::deserialize_zeroable_id")]
    pub target_id: OptionId,
    pub state: ThumbnailState,
    pub image_url: Option<String>,
    pub version: Option<ThumbnailVersion>,
}

#[derive(Debug, Deserialize, Clone)]
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
