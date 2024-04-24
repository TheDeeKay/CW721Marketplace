use crate::api::TrackMetadata;
use cosmwasm_std::Empty;

pub type InstantiateMsg = cw721_base::InstantiateMsg;
pub type ExecuteMsg = cw721_base::ExecuteMsg<TrackMetadata, Empty>;
pub type QueryMsg = cw721_base::QueryMsg<Empty>;
