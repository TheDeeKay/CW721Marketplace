use crate::assert_is_err;
use crate::cw721_tracks::cw721_tracks_helpers::{Cw721TracksExecute, Cw721TracksQueries, default_track_metadata};
use crate::helpers::{BalanceQuery, MoveBlock, TestFixture, ADMIN, UATOM, USER1, USER2, USER3};
use crate::tracks_auction::tracks_auction_helpers::{TracksAuctionExecute, TracksAuctionQuery};
use cosmwasm_std::{coin, coins};
use cw_utils::Duration;
use Duration::Time;

#[test]
fn nft_auction_with_several_bids() -> anyhow::Result<()> {
    let mut fixture = TestFixture::new();

    let token_id = "tokenID";

    fixture.mint_nft(USER1, token_id, None, default_track_metadata())?;

    fixture.create_nft_auction(USER1, token_id, Time(100), 100, None)?;

    // lower than minimum bid fails
    assert_is_err!(fixture.bid_on_auction(USER2, 0, coin(99, UATOM)));

    // first over the minimum bid becomes the active bid
    fixture.bid_on_auction(USER2, 0, coin(100, UATOM))?;
    fixture.assert_active_bid(0, USER2, coin(100, UATOM), None);

    fixture.move_time_sec(20);

    // resolving before auction is over fails
    assert_is_err!(fixture.resolve_auction(ADMIN, 0));

    // next higher bid becomes the new active bid and refunds previous bid
    fixture.bid_on_auction(USER3, 0, coin(101, UATOM))?;
    fixture.assert_active_bid(0, USER3, coin(101, UATOM), None);
    fixture.assert_balance(USER2, coins(100, UATOM));

    // next bid must be higher than the current
    assert_is_err!(fixture.bid_on_auction(USER2, 0, coin(101, UATOM)));

    // auction expires
    fixture.move_time_sec(81);

    // after auction expires, no bids are accepted
    assert_is_err!(fixture.bid_on_auction(USER2, 0, coin(102, UATOM)));

    // resolving auction sends bid to auction owner, NFT to auction winner
    fixture.resolve_auction(ADMIN, 0)?;
    fixture.assert_nft_owner(token_id, USER3);
    fixture.assert_balance(USER1, coins(101, UATOM));

    Ok(())
}

#[test]
fn nft_auction_with_instant_buyout() -> anyhow::Result<()> {
    let mut fixture = TestFixture::new();

    let token_id = "tokenID";

    fixture.mint_nft(USER1, token_id, None, default_track_metadata())?;

    fixture.create_nft_auction(USER1, token_id, Time(100), 100, Some(200))?;

    // bid minimum
    fixture.bid_on_auction(USER2, 0, coin(100, UATOM))?;

    // bidding the buyout price refunds previous bid and instantly finishes the auction
    fixture.bid_on_auction(USER3, 0, coin(200, UATOM))?;

    // NFT is transferred to the buyer, buyout amount to the auction creator
    fixture.assert_nft_owner(token_id, USER3);
    fixture.assert_balance(USER1, coins(200, UATOM));

    // previous bid is refunded
    fixture.assert_balance(USER2, coins(100, UATOM));

    // resolving or canceling auction after buyout fails
    assert_is_err!(fixture.resolve_auction(ADMIN, 0));
    assert_is_err!(fixture.cancel_auction(ADMIN, 0));

    Ok(())
}

#[test]
fn nft_auction_cancel() -> anyhow::Result<()> {
    let mut fixture = TestFixture::new();

    let token_id = "tokenID";

    fixture.mint_nft(USER1, token_id, None, default_track_metadata())?;

    fixture.create_nft_auction(USER1, token_id, Time(100), 100, None)?;

    // make a bid
    fixture.bid_on_auction(USER2, 0, coin(105, UATOM))?;

    // non-owner cannot cancel the auction
    assert_is_err!(fixture.cancel_auction(ADMIN, 0));

    // cancel the auction
    fixture.cancel_auction(USER1, 0)?;

    // NFT is transferred to the original owner, active bid is refunded
    fixture.assert_nft_owner(token_id, USER1);
    fixture.assert_balance(USER2, coins(105, UATOM));

    // resolving or canceling auction after canceling fails
    assert_is_err!(fixture.resolve_auction(ADMIN, 0));
    assert_is_err!(fixture.cancel_auction(USER1, 0));

    Ok(())
}
