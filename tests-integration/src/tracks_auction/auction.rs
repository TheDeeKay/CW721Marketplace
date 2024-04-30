use crate::assert_is_err;
use crate::cw721_tracks::cw721_tracks_helpers::{Cw721TracksExecute, Cw721TracksQueries};
use crate::helpers::{BalanceQuery, MoveBlock, TestFixture, ADMIN, UATOM, USER1, USER2, USER3};
use crate::tracks_auction::tracks_auction_helpers::{TracksAuctionExecute, TracksAuctionQuery};
use cosmwasm_std::{coin, coins};
use cw721_tracks_api::api::TrackMetadata;
use cw_utils::Duration;
use Duration::Time;

#[test]
fn single_nft_auction_with_bidders() -> anyhow::Result<()> {
    let mut fixture = TestFixture::new();

    let token_id = "tokenID";

    fixture.mint_nft(
        USER1,
        token_id,
        None,
        TrackMetadata {
            artist_name: "Boden".to_string(),
            album: None,
            track_name: "Debt Spiral".to_string(),
            audio_track_url: "https://www.usdebtclock.org/".to_string(),
        }
        .clone(),
    )?;

    fixture.create_nft_auction(USER1, token_id, Time(100), 100, None)?;

    // lower than minimum bid fails
    assert_is_err!(fixture.bid_on_auction(USER2, 0, coin(99, UATOM)));

    // first over the minimum bid
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
