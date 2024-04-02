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
struct BatchUserInfoRequest<T: Iterator<Item = Id>> {
    #[serde(with = "serde_iter::seq")]
    user_ids: CloneOnce<Id, T>,
    exclude_banned_users: bool,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchUser {
    pub id: Id,
    pub name: String,
    pub display_name: String,
    pub has_verified_badge: bool,
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
    async fn get_user_info_batch<T>(
        &self,
        users: T,
        exclude_banned_users: bool,
    ) -> RequestResult<BatchUser>
    where
        T: IntoIterator<Item = Id> + Send,
        T::IntoIter: Send,
    {
        self.post::<BatchUser, BatchUserInfoRequest<T::IntoIter>>(
            add_base_url!("v1/users"),
            BatchUserInfoRequest {
                user_ids: users.into_iter().into(),
                exclude_banned_users,
            },
        )
        .await
    }
}
