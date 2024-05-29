use crate::{
    apis::{RequestLimit, SortOrder},
    BaseClient,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Id, JsonError, Page, Paginator, RequestResult, StringError};

#[derive(Debug, Clone, Copy)]
pub enum ServerType {
    Public = 0,
    Friend = 1,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicServer {
    pub id: Uuid,
    pub max_players: u16,
    pub playing: u16,
    pub player_tokens: Vec<String>,
    pub players: Vec<ServerListPlayer>,
    pub fps: f32,
    pub ping: u16,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerListPlayer {
    pub player_token: String,
    pub id: Id,
    pub name: String,
    pub display_name: String,
}

macro_rules! add_base_url {
    ($api_route: literal) => {
        concat!("https://games.roblox.com/", $api_route)
    };
    ($api_format_string: expr, $($args:expr),+) => {
        format!(concat!("https://games.roblox.com/", $api_format_string), $($args),+)
    };
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BatchParameters<'a> {
    #[serde(skip_serializing_if = "crate::utils::is_default")]
    sort_order: SortOrder,
    #[serde(
        rename = "excludeFullGames",
        skip_serializing_if = "crate::utils::is_default"
    )]
    exclude_full_servers: bool,
    #[serde(skip_serializing_if = "crate::utils::is_default")]
    limit: RequestLimit,
    #[serde(skip_serializing_if = "Option::is_none")]
    cursor: Option<&'a str>,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PlaceDetails {
    pub asset_id: Id,
    pub name: String,
    pub description: String,
    #[serde(deserialize_with = "super::deserialize_date")]
    pub created: NaiveDate,
    #[serde(deserialize_with = "super::deserialize_date")]
    pub updated: NaiveDate,
    pub favorited_count: u64,
    pub url: String,
    pub report_abuse_absolute_url: String,
    pub is_favorited_by_user: bool,
    pub is_favorites_unavailable: bool,
    pub user_can_manage_place: bool,
    pub visited_count: u64,
    pub max_players: u16,
    pub builder: String,
    pub builder_id: Id,
    pub builder_absolute_url: String,
    pub is_playable: bool,
    pub reason_prohibited: String,
    pub reason_prohibited_message: String,
    pub is_copying_allowed: bool,
    pub play_button_type: String,
    pub asset_genre: String,
    pub asset_genre_view_model: AssetGenreViewModel,
    pub online_count: u32,
    pub universe_id: Id,
    pub universe_root_place_id: Id,
    pub total_up_votes: u64,
    pub total_down_votes: u64,
    pub user_vote: Option<bool>,
    pub overrides_default_avatar: bool,
    pub use_portrait_mode: bool,
    pub price: u32,
    pub voice_enabled: bool,
    pub camera_enabled: bool,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AssetGenreViewModel {
    pub display_name: String,
    pub id: u8,
}

#[async_trait]
pub trait GamesApi: BaseClient {
    /// Very large or no rate limit
    async fn get_place_details(&self, place_id: Id) -> RequestResult<PlaceDetails, StringError> {
        self.get(
            "https://www.roblox.com/places/api-get-details",
            Some([("assetId", place_id)]),
        )
        .await
    }
    /// Rate limit: 10 requests/3.5s
    fn get_public_servers(
        &self,
        place_id: Id,
        server_type: ServerType,
        sort_order: SortOrder,
        exclude_full_servers: bool,
        limit: RequestLimit,
        cursor: Option<impl Into<String>>,
    ) -> Paginator<'_, PublicServer, JsonError> {
        super::paginate(
            move |cursor| {
                self.get_public_servers_manual(
                    place_id,
                    server_type,
                    sort_order,
                    exclude_full_servers,
                    limit,
                    cursor,
                )
            },
            cursor,
        )
    }
    /// Rate limit: 10 requests/3.5s
    async fn get_public_servers_manual(
        &self,
        place_id: Id,
        server_type: ServerType,
        sort_order: SortOrder,
        exclude_full_servers: bool,
        limit: RequestLimit,
        cursor: Option<impl AsRef<str> + Send>,
    ) -> RequestResult<Page<PublicServer>, JsonError> {
        let a = cursor.as_ref().map(AsRef::as_ref);
        self.get(
            add_base_url!("v1/games/{}/servers/{}", place_id, server_type as u8),
            Some(BatchParameters {
                sort_order,
                exclude_full_servers,
                limit,
                cursor: a,
            }),
        )
        .await
    }
}

impl<T: BaseClient> GamesApi for T {}
