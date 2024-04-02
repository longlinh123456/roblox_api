use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_iter::CloneOnce;

use crate::{AuthenticatedClient, BaseClient, RequestResult};

use super::Id;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatedUser {
    pub id: Id,
    pub name: String,
    pub display_name: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchUserInfoFromIdRequest<T: Iterator<Item = Id>> {
    #[serde(with = "serde_iter::seq")]
    user_ids: CloneOnce<Id, T>,
    exclude_banned_users: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchUserInfoFromUsernameRequest<'a, T: Iterator<Item = &'a str>> {
    #[serde(with = "serde_iter::seq")]
    usernames: CloneOnce<&'a str, T>,
    exclude_banned_users: bool,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchUserInfoFromId {
    pub id: Id,
    pub name: String,
    pub display_name: String,
    pub has_verified_badge: bool,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct BatchUserInfoFromIdResponse {
    data: Vec<BatchUserInfoFromId>,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchUserInfoFromUsername {
    pub requested_username: String,
    pub id: Id,
    pub name: String,
    pub display_name: String,
    pub has_verified_badge: bool,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct BatchUserInfoFromUsernameResponse {
    data: Vec<BatchUserInfoFromUsername>,
}
macro_rules! add_base_url {
    ($api_route: literal) => {
        concat!("https://users.roblox.com/", $api_route)
    };
    ($api_format_string: expr, $($args:expr),+) => {
        format!(concat!("https://users.roblox.com/", $api_format_string), $($args),+)
    };
}

#[async_trait]
pub trait UsersAuthenticatedApi: AuthenticatedClient {
    async fn get_authenticated(&self) -> RequestResult<AuthenticatedUser> {
        self.authenticated_get::<AuthenticatedUser, ()>(
            add_base_url!("v1/users/authenticated"),
            None,
        )
        .await
    }
}

impl<T: AuthenticatedClient> UsersAuthenticatedApi for T {}

#[async_trait]
pub trait UsersApi: BaseClient {
    /// Limit of 200 users/request
    async fn get_user_info_from_id_batch<T>(
        &self,
        users: T,
        exclude_banned_users: bool,
    ) -> RequestResult<Vec<BatchUserInfoFromId>>
    where
        T: IntoIterator<Item = Id> + Send,
        T::IntoIter: Send,
    {
        let res = self
            .post::<BatchUserInfoFromIdResponse, BatchUserInfoFromIdRequest<T::IntoIter>>(
                add_base_url!("v1/users"),
                BatchUserInfoFromIdRequest {
                    user_ids: users.into_iter().into(),
                    exclude_banned_users,
                },
            )
            .await?;
        Ok(res.data)
    }
    /// Limit of 200 users/request
    async fn get_user_info_from_username_batch<'a, T>(
        &self,
        users: T,
        exclude_banned_users: bool,
    ) -> RequestResult<Vec<BatchUserInfoFromUsername>>
    where
        T: IntoIterator<Item = &'a str> + Send,
        T::IntoIter: Send,
    {
        let res = self
            .post::<BatchUserInfoFromUsernameResponse, BatchUserInfoFromUsernameRequest<T::IntoIter>>(
                add_base_url!("v1/usernames/users"),
                BatchUserInfoFromUsernameRequest {
                    usernames: users.into_iter().into(),
                    exclude_banned_users,
                },
            )
            .await?;
        Ok(res.data)
    }
}
