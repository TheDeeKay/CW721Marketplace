use crate::helpers::{TestFixture, ADMIN};
use cosmwasm_std::{Addr, StdResult};
use cw20::Cw20ExecuteMsg::{Burn, Mint};
use cw20::Cw20QueryMsg::Balance;
use cw20::{BalanceResponse, MinterResponse};
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor, IntoAddr};

pub const CW20_MINTER: &str = "cw20_minter";

pub fn store_cw20_code(app: &mut App) -> u64 {
    app.store_code(Box::new(ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    )))
}

pub fn instantiate_cw20(app: &mut App, code_id: u64) -> AnyResult<Addr> {
    let msg = cw20_base::msg::InstantiateMsg {
        name: "CW20".to_string(),
        symbol: "TKN".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: CW20_MINTER.into_addr().to_string(),
            cap: None,
        }),
        marketing: None,
    };

    app.instantiate_contract(
        code_id,
        ADMIN.into_addr(),
        &msg,
        &[],
        "CW20",
        Some(ADMIN.into_addr().to_string()),
    )
}

pub fn store_and_instantiate_cw20(app: &mut App) -> AnyResult<(u64, Addr)> {
    let code_id = store_cw20_code(app);
    let addr = instantiate_cw20(app, code_id);

    addr.map(|address| (code_id, address))
}

pub trait Cw20Query {
    fn cw20_balance(&self, addr: &str, cw20_addr: Addr) -> StdResult<u128>;
    fn assert_cw20_balance(&self, addr: &str, cw20_addr: Addr, amount: u128);
}

impl Cw20Query for TestFixture {
    fn cw20_balance(&self, addr: &str, cw20_addr: Addr) -> StdResult<u128> {
        let balance: BalanceResponse = self.app.wrap().query_wasm_smart(
            cw20_addr,
            &Balance {
                address: addr.into_addr().to_string(),
            },
        )?;

        Ok(balance.balance.u128())
    }

    fn assert_cw20_balance(&self, addr: &str, cw20_addr: Addr, amount: u128) {
        assert_eq!(self.cw20_balance(addr, cw20_addr).unwrap(), amount)
    }
}

pub trait Cw20Mint {
    fn mint_cw20(
        &mut self,
        cw20_addr: Addr,
        receiver: &str,
        amount: u128,
    ) -> AnyResult<AppResponse>;
}

impl Cw20Mint for TestFixture {
    fn mint_cw20(
        &mut self,
        cw20_addr: Addr,
        receiver: &str,
        amount: u128,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            CW20_MINTER.into_addr(),
            cw20_addr,
            &Mint {
                recipient: receiver.into_addr().to_string(),
                amount: amount.into(),
            },
            &vec![],
        )
    }
}

pub trait Cw20Burn {
    fn burn_cw20(&mut self, cw20_addr: Addr, burner: &str, amount: u128) -> AnyResult<AppResponse>;
}

impl Cw20Burn for TestFixture {
    fn burn_cw20(&mut self, cw20_addr: Addr, burner: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            burner.into_addr(),
            cw20_addr,
            &Burn {
                amount: amount.into(),
            },
            &vec![],
        )
    }
}
