use cosmwasm_std::{BlockInfo, Coin, StdResult};
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, BankSudo, IntoAddr, SudoMsg};

pub const ADMIN: &str = "admin";

pub const USER1: &str = "user1";
pub const USER2: &str = "user2";
pub const USER3: &str = "user3";

pub const UATOM: &str = "uatom";

const BLOCK_TIME_SEC: u64 = 5;

pub trait ImplApp {
    fn mint_native(&mut self, addr: &str, coins: Vec<Coin>) -> AnyResult<()>;
    fn mint_native_multi(&mut self, balances: Vec<(impl Into<String>, Vec<Coin>)>)
        -> AnyResult<()>;
}

impl ImplApp for App {
    fn mint_native(&mut self, addr: &str, coins: Vec<Coin>) -> AnyResult<()> {
        self.sudo(SudoMsg::Bank(BankSudo::Mint {
            to_address: addr.into_addr().to_string(),
            amount: coins,
        }))?;

        Ok(())
    }

    fn mint_native_multi(
        &mut self,
        balances: Vec<(impl Into<String>, Vec<Coin>)>,
    ) -> AnyResult<()> {
        for (addr, coins) in balances {
            self.mint_native(&addr.into(), coins)?
        }
        Ok(())
    }
}

pub trait MoveBlock {
    fn move_block_height(&mut self, blocks: u64);
    fn move_time_sec(&mut self, seconds: u64);
}

impl MoveBlock for App {
    fn move_block_height(&mut self, blocks: u64) {
        let current_block = self.block_info();

        let new_block = BlockInfo {
            height: current_block.height + blocks,
            time: current_block.time.plus_seconds(blocks * BLOCK_TIME_SEC),
            ..current_block
        };

        self.set_block(new_block)
    }

    fn move_time_sec(&mut self, seconds: u64) {
        let current_block = self.block_info();

        let new_block = BlockInfo {
            height: current_block.height + seconds / BLOCK_TIME_SEC,
            time: current_block.time.plus_seconds(seconds),
            ..current_block
        };

        self.set_block(new_block)
    }
}

pub trait BalanceQuery {
    fn balance(&self, addr: &str, denom: &str) -> StdResult<u128>;
    fn assert_balance(&self, addr: &str, coins: Vec<Coin>);
}

impl BalanceQuery for App {
    fn balance(&self, addr: &str, denom: &str) -> StdResult<u128> {
        self.wrap()
            .query_balance(addr.into_addr().as_str(), denom)
            .map(|it| it.amount.u128())
    }

    fn assert_balance(&self, addr: &str, coins: Vec<Coin>) {
        for coin in coins {
            assert_eq!(self.balance(addr, &coin.denom).unwrap(), coin.amount.u128());
        }
    }
}
