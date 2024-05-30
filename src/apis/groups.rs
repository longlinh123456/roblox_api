use crate::{AuthenticatedClient, BaseClient, RequestResult};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{Empty, Id, JsonError};

#[derive(Deserialize, Debug, Clone)]
pub struct Shout {
    pub body: String,
    pub poster: DetailedOwner,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct BatchOwner {
    pub id: Id,
    #[serde(flatten)]
    pub r#type: OwnerType,
}

#[derive(Deserialize, Default, Debug, Clone, Copy)]
#[serde(tag = "type")]
pub enum OwnerType {
    #[default]
    User,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DetailedOwner {
    pub has_verified_badge: bool,
    pub user_id: Id,
    pub username: String,
    pub display_name: String,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchGroupInfo {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub owner: Option<BatchOwner>,
    pub created: DateTime<Utc>,
    pub has_verified_badge: bool,
}
#[derive(Deserialize, Default, Debug, Clone)]
struct BatchResponse {
    data: Vec<BatchGroupInfo>,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct SingleGroupInfo {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub owner: Option<DetailedOwner>,
    pub shout: Option<Shout>,
    pub member_count: u64,
    pub is_builders_club_only: bool,
    pub public_entry_allowed: bool,
    pub has_verified_badge: bool,
}

#[derive(Serialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolvedCaptcha<'a> {
    pub session_id: &'a str,
    pub redemption_token: &'a str,
    pub captcha_id: &'a str,
    pub captcha_token: &'a str,
    pub captcha_provider: &'a str,
    pub challenge_id: &'a str,
}

#[derive(Deserialize, Default, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct GroupMetadata {
    pub group_limit: u16,
    pub current_group_count: u16,
    pub group_status_max_length: u16,
    pub group_post_max_length: u16,

    #[serde(rename = "isGroupWallNotificationsEnabled")]
    pub group_wall_notifications_enabled: bool,

    #[serde(rename = "groupWallNotificationsSubscribeIntervalInMilliseconds")]
    pub group_wall_notifications_subscribe_interval: u32,

    #[serde(rename = "areProfileGroupsHidden")]
    pub profile_groups_hidden: bool,

    #[serde(rename = "isGroupDetailsPolicyEnabled")]
    pub group_details_policy_enabled: bool,
    pub show_previous_group_names: bool,
}

macro_rules! add_base_url {
    ($api_route: literal) => {
        concat!("https://groups.roblox.com/", $api_route)
    };
    ($api_format_string: expr, $($args:expr),+) => {
        format!(concat!("https://groups.roblox.com/", $api_format_string), $($args),+)
    };
}
#[async_trait]
pub trait GroupsApi: BaseClient {
    /// Limit of 100 groups/request
    ///
    /// Rate limit: 100 requests/min
    async fn get_group_info_batch(
        &self,
        groups: impl IntoIterator<Item = Id> + Send,
    ) -> RequestResult<Vec<BatchGroupInfo>, JsonError> {
        let query_ids = groups.into_iter().join(",");
        let response = self
            .get::<BatchResponse, _>(
                add_base_url!("v2/groups"),
                Some([("groupIds", query_ids.as_str())]),
            )
            .await?;
        Ok(response.data)
    }
    async fn get_group_info(&self, group: Id) -> RequestResult<SingleGroupInfo, JsonError> {
        self.get(add_base_url!("v1/groups/{}", group), None::<()>)
            .await
    }
    async fn get_metadata(&self) -> RequestResult<GroupMetadata, JsonError> {
        self.get(add_base_url!("v1/groups/metadata"), None::<()>)
            .await
    }
}
impl<T: BaseClient> GroupsApi for T {}

#[async_trait]
pub trait GroupsAuthenticatedApi: AuthenticatedClient {
    async fn join_group<'a>(
        &self,
        group: Id,
        solved_captcha: Option<SolvedCaptcha<'a>>,
    ) -> RequestResult<Empty, JsonError> {
        self.authenticated_post(add_base_url!("v1/groups/{}/users", group), solved_captcha)
            .await
    }
    async fn claim_group(&self, group: Id) -> RequestResult<Empty, JsonError> {
        self.authenticated_post(
            add_base_url!("v1/groups/{}/claim-ownership", group),
            None::<()>,
        )
        .await
    }
    async fn remove_user_from_group(
        &self,
        group: Id,
        target: Id,
    ) -> RequestResult<Empty, JsonError> {
        self.authenticated_delete(
            add_base_url!("v1/groups/{}/users/{}", group, target),
            None::<()>,
        )
        .await
    }
}
impl<T: AuthenticatedClient> GroupsAuthenticatedApi for T {}
