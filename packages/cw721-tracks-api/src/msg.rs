use cosmwasm_std::Empty;
use cw721_base::Extension;

pub type InstantiateMsg = cw721_base::InstantiateMsg;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;
pub type QueryMsg = cw721_base::QueryMsg<Empty>;
