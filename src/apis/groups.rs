use crate::{AuthenticatedClient, BaseClient, RequestResult};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use deranged::RangedU32;
use futures::Stream;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{
    Empty, Id, JsonError, MembershipType, Page, RequestLimit, SortOrder, SortOrderDefaultAscending,
};

#[derive(Deserialize, Debug, Clone)]
pub struct GroupShout {
    pub body: String,
    pub poster: DetailedGroupUser,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct BatchGroupUser {
    pub id: Id,
    #[serde(flatten)]
    pub r#type: GroupOwnerType,
}

#[derive(Deserialize, Default, Debug, Clone, Copy)]
#[serde(tag = "type")]
pub enum GroupOwnerType {
    #[default]
    User,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DetailedGroupUser {
    #[serde(rename = "buildersClubMembershipType")]
    pub membership_type: MembershipType,
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
    pub owner: Option<BatchGroupUser>,
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
    pub owner: Option<DetailedGroupUser>,
    pub shout: Option<GroupShout>,
    pub member_count: u32,
    pub is_builders_club_only: bool,
    pub public_entry_allowed: bool,
    pub has_verified_badge: bool,
}

#[derive(Serialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolvedCaptcha<T1: Send, T2: Send, T3: Send, T4: Send, T5: Send, T6: Send> {
    pub session_id: T1,
    pub redemption_token: T2,
    pub captcha_id: T3,
    pub captcha_token: T4,
    pub captcha_provider: T5,
    pub challenge_id: T6,
}

#[derive(Deserialize, Default, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct GroupMetadata {
    pub group_limit: u32,
    pub current_group_count: u32,
    pub group_status_max_length: u32,
    pub group_post_max_length: u32,

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

pub type GroupRoleRank = RangedU32<0, { i32::MAX as u32 }>;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupRole {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub rank: GroupRoleRank,
    pub member_count: u64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupMember {
    pub user: DetailedGroupUser,
    pub role: GroupRole,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GetGroupMembersParameters<T: Send> {
    group: Id,
    #[serde(skip_serializing_if = "crate::utils::is_default")]
    limit: RequestLimit,
    #[serde(skip_serializing_if = "Option::is_none")]
    cursor: Option<T>,
    #[serde(skip_serializing_if = "crate::utils::is_default")]
    sort_order: SortOrderDefaultAscending,
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
    async fn get_group_metadata(&self) -> RequestResult<GroupMetadata, JsonError> {
        self.get(add_base_url!("v1/groups/metadata"), None::<()>)
            .await
    }
    fn get_group_members(
        &self,
        group: Id,
        limit: RequestLimit,
        cursor: Option<impl Into<String>>,
        sort_order: SortOrder,
    ) -> impl Stream<Item = RequestResult<Page<GroupMember>, JsonError>> {
        super::paginate(
            async move |cursor| {
                self.get_group_members_manual(group, limit, cursor, sort_order)
                    .await
            },
            cursor,
        )
    }
    async fn get_group_members_manual(
        &self,
        group: Id,
        limit: RequestLimit,
        cursor: Option<impl Serialize + Send>,
        sort_order: SortOrder,
    ) -> RequestResult<Page<GroupMember>, JsonError> {
        self.get(
            add_base_url!("/v1/groups/{}/users"),
            Some(GetGroupMembersParameters {
                group,
                limit,
                cursor,
                sort_order: sort_order.into(),
            }),
        )
        .await
    }
}
impl<T: BaseClient> GroupsApi for T {}

#[async_trait]
pub trait GroupsAuthenticatedApi: AuthenticatedClient {
    async fn join_group<'a>(
        &self,
        group: Id,
        solved_captcha: Option<
            SolvedCaptcha<
                impl Serialize + Send,
                impl Serialize + Send,
                impl Serialize + Send,
                impl Serialize + Send,
                impl Serialize + Send,
                impl Serialize + Send,
            >,
        >,
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
