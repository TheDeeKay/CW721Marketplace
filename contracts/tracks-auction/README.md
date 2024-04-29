## Tracks auction

This smart contract allows users to create auctions for their NFTs.

NFT owners can post an auction with a minimum bid and a duration, and other users can bid or outright buy the NFT.


### Auction flow

1. An NFT owner posts up an auction for it. Their NFT is sent to this contract and is held there until the auction ends.
2. While the auction is active, other users can bid for the NFT, starting with the minimum bid specified by the auction creator. Highest bid is escrowed in the contract, and the previous ones refunded.

The auction can then end in 3 ways:
- The auction expires with no bids. Anyone can execute its resolution, which returns the NFT to the original owner.
- The auction expires with at least one bid. The bid amount goes to the original NFT owner, and the NFT is transferred to the highest bidder.
- The creator initially specified a buyout price. If this price is reached by a bid, the NFT is instantly sold to the bidder.