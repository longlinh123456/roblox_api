use crate::{
    apis::{RequestLimit, SortOrder},
    BaseClient,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Id, Page, Paginator, RequestResult};

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

#[async_trait]
pub trait GamesApi: BaseClient {
    fn get_public_servers<S: Into<String>>(
        &self,
        place_id: Id,
        server_type: ServerType,
        sort_order: SortOrder,
        exclude_full_servers: bool,
        limit: RequestLimit,
        cursor: impl Into<Option<S>> + Send,
    ) -> Paginator<'_, PublicServer> {
        super::paginate::<_, S, _, _>(
            move |cursor| {
                self.get_public_servers_manual::<String>(
                    place_id,
                    server_type,
                    sort_order,
                    exclude_full_servers,
                    limit,
                    cursor,
                )
            },
            cursor.into(),
        )
    }
    async fn get_public_servers_manual<S: AsRef<str> + Send>(
        &self,
        place_id: Id,
        server_type: ServerType,
        sort_order: SortOrder,
        exclude_full_servers: bool,
        limit: RequestLimit,
        cursor: impl Into<Option<S>> + Send,
    ) -> RequestResult<Page<PublicServer>> {
        let cursor = cursor.into();
        let a = cursor.as_ref().map(AsRef::as_ref);
        self.get::<Page<PublicServer>, BatchParameters>(
            add_base_url!("v1/games/{}/servers/{}", place_id, server_type as u8),
            BatchParameters {
                sort_order,
                exclude_full_servers,
                limit,
                cursor: a,
            },
        )
        .await
    }
}

impl<T: BaseClient> GamesApi for T {}
