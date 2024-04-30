use crate::cw20_helpers::cw20_helpers::store_and_instantiate_cw20;
use crate::cw721_tracks::cw721_tracks_helpers::store_and_instantiate_cw721_tracks;
use crate::tracks_auction::tracks_auction_helpers::store_and_instantiate_tracks_auction;
use cosmwasm_std::{Addr, BlockInfo, Coin, StdResult};
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, BankSudo, IntoAddr, SudoMsg};
use tracks_auction_api::api::PriceAssetUnchecked;

pub const ADMIN: &str = "admin";

pub const USER1: &str = "user1";
pub const USER2: &str = "user2";
pub const USER3: &str = "user3";

pub const UATOM: &str = "uatom";
pub const UANDR: &str = "uandr";

const BLOCK_TIME_SEC: u64 = 5;

pub struct ContractInfo {
    pub addr: Addr,
    pub code_id: u64,
}

impl ContractInfo {
    pub fn new(addr: Addr, code_id: u64) -> ContractInfo {
        ContractInfo { addr, code_id }
    }
}

pub struct TestFixture {
    pub app: App,
    pub cw721_tracks: ContractInfo,
    pub tracks_auction: ContractInfo,
    pub cw20: ContractInfo,
}

impl TestFixture {
    pub fn new_with_native(denom: &str) -> TestFixture {
        let mut app = App::default();

        let (cw721_tracks_code_id, cw721_tracks) =
            store_and_instantiate_cw721_tracks(&mut app).unwrap();

        let (tracks_auction_code_id, tracks_auction) = store_and_instantiate_tracks_auction(
            &mut app,
            cw721_tracks.to_string(),
            PriceAssetUnchecked::native(denom),
        )
        .unwrap();

        let (cw20_code_id, cw20) = store_and_instantiate_cw20(&mut app).unwrap();

        TestFixture {
            app,
            cw721_tracks: ContractInfo::new(cw721_tracks, cw721_tracks_code_id),
            tracks_auction: ContractInfo::new(tracks_auction, tracks_auction_code_id),
            cw20: ContractInfo::new(cw20, cw20_code_id),
        }
    }

    pub fn new_with_cw20(mut app: App, cw20_code_id: u64, cw20: Addr) -> TestFixture {
        let (cw721_tracks_code_id, cw721_tracks) =
            store_and_instantiate_cw721_tracks(&mut app).unwrap();

        let (tracks_auction_code_id, tracks_auction) = store_and_instantiate_tracks_auction(
            &mut app,
            cw721_tracks.to_string(),
            PriceAssetUnchecked::cw20(cw20.to_string()),
        )
        .unwrap();

        TestFixture {
            app,
            cw721_tracks: ContractInfo::new(cw721_tracks, cw721_tracks_code_id),
            tracks_auction: ContractInfo::new(tracks_auction, tracks_auction_code_id),
            cw20: ContractInfo {
                addr: cw20,
                code_id: cw20_code_id,
            },
        }
    }
}

pub trait NativeMInt {
    fn mint_native(&mut self, addr: &str, coins: Vec<Coin>) -> AnyResult<()>;
    fn mint_native_multi(&mut self, balances: Vec<(impl Into<String>, Vec<Coin>)>)
        -> AnyResult<()>;
}

impl NativeMInt for App {
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

impl MoveBlock for TestFixture {
    fn move_block_height(&mut self, blocks: u64) {
        let current_block = self.app.block_info();

        let new_block = BlockInfo {
            height: current_block.height + blocks,
            time: current_block.time.plus_seconds(blocks * BLOCK_TIME_SEC),
            ..current_block
        };

        self.app.set_block(new_block)
    }

    fn move_time_sec(&mut self, seconds: u64) {
        let current_block = self.app.block_info();

        let new_block = BlockInfo {
            height: current_block.height + seconds / BLOCK_TIME_SEC,
            time: current_block.time.plus_seconds(seconds),
            ..current_block
        };

        self.app.set_block(new_block)
    }
}

pub trait BalanceQuery {
    fn balance(&self, addr: &str, denom: &str) -> StdResult<u128>;
    fn assert_balance(&self, addr: &str, coins: Vec<Coin>);
}

impl BalanceQuery for TestFixture {
    fn balance(&self, addr: &str, denom: &str) -> StdResult<u128> {
        self.app
            .wrap()
            .query_balance(addr.into_addr().as_str(), denom)
            .map(|it| it.amount.u128())
    }

    fn assert_balance(&self, addr: &str, coins: Vec<Coin>) {
        for coin in coins {
            assert_eq!(self.balance(addr, &coin.denom).unwrap(), coin.amount.u128());
        }
    }
}

#[macro_export]
macro_rules! assert_is_err {
    ($result:expr) => {
        if !$result.is_err() {
            panic!("Expected an error, but got {:?}", $result);
        }
    };
}
