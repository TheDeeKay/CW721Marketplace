use cosmwasm_schema::schemars::schema_for;
use cosmwasm_schema::{export_schema, remove_schemas};
use cw721_tracks_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
}