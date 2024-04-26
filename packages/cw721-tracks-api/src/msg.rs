use crate::api::TrackMetadata;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
}

pub type ExecuteMsg = cw721_base::ExecuteMsg<TrackMetadata, Empty>;
pub type QueryMsg = cw721_base::QueryMsg<Empty>;
