use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{AuthenticatedClient, BaseClient, RequestResult};

use super::{Id, JsonError};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatedUser {
    pub id: Id,
    pub name: String,
    pub display_name: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchUserInfoFromIdRequest<T: Iterator<Item = Id> + Clone> {
    #[serde(with = "serde_iter::seq")]
    user_ids: T,
    exclude_banned_users: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchUserInfoFromUsernameRequest<'a, T: Iterator<Item = &'a str> + Clone> {
    #[serde(with = "serde_iter::seq")]
    usernames: T,
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
    async fn get_authenticated(&self) -> RequestResult<AuthenticatedUser, JsonError> {
        self.authenticated_get::<_, (), _>(add_base_url!("v1/users/authenticated"), None)
            .await
    }
}

impl<T: AuthenticatedClient> UsersAuthenticatedApi for T {}

#[async_trait]
pub trait UsersApi: BaseClient {
    /// Limit of 200 users/request
    ///
    /// Very large or no rate limit
    async fn get_user_info_from_id_batch<T>(
        &self,
        users: T,
        exclude_banned_users: bool,
    ) -> RequestResult<Vec<BatchUserInfoFromId>, JsonError>
    where
        T: IntoIterator<Item = Id> + Send,
        T::IntoIter: Send + Clone,
    {
        let res = self
            .post::<BatchUserInfoFromIdResponse, _, _>(
                add_base_url!("v1/users"),
                BatchUserInfoFromIdRequest {
                    user_ids: users.into_iter(),
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
    ) -> RequestResult<Vec<BatchUserInfoFromUsername>, JsonError>
    where
        T: IntoIterator<Item = &'a str> + Send,
        T::IntoIter: Send + Clone,
    {
        let res = self
            .post::<BatchUserInfoFromUsernameResponse, _, _>(
                add_base_url!("v1/usernames/users"),
                BatchUserInfoFromUsernameRequest {
                    usernames: users.into_iter(),
                    exclude_banned_users,
                },
            )
            .await?;
        Ok(res.data)
    }
}

impl<T: BaseClient> UsersApi for T {}
