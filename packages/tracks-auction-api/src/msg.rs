use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
pub enum Cw721HookMsg {
    CreateAuction {},
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {}
