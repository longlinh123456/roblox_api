use async_trait::async_trait;
use serde::Deserialize;

use crate::BaseClient;

use super::{Id, JsonError, OptionId, RequestResult};

macro_rules! add_base_url {
    ($api_route: literal) => {
        concat!("https://apis.roblox.com/", $api_route)
    };
    ($api_format_string: expr, $($args:expr),+) => {
        format!(concat!("https://apis.roblox.com/", $api_format_string), $($args),+)
    };
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PlaceResponse {
    universe_id: OptionId,
}

#[async_trait]
pub trait GeneralApi: BaseClient {
    async fn get_universe_from_place(&self, place: Id) -> RequestResult<OptionId, JsonError> {
        let res = self
            .get::<PlaceResponse, (), _>(
                add_base_url!("universes/v1/places/{}/universe", place),
                None,
            )
            .await?;
        Ok(res.universe_id)
    }
}

impl<T: BaseClient> GeneralApi for T {}
