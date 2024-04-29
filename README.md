# CW721 marketplace

A set of contracts allowing artists to mint NFTs representing their tracks, and then auction them to the highest bidder,
optionally allowing instant buyout.

Each auction will run for a specified amount of time, either ending up with a sale or not.
Additionally, auction creator can specify a buyout price at which the auction will immediately end and result in a sale.

The auction contract will dictate which currency is used in auctions.

| Contract                                                                                           | Description                                                                                                |                                                                                                                                 
|----------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------|
| [cw721-tracks](https://github.com/TheDeeKay/CW721Marketplace/tree/main/contracts/cw721-tracks)     | Modified CW721 contract with permissionless minting and on-chain metadata for tracks.                      |
| [tracks-auction](https://github.com/TheDeeKay/CW721Marketplace/tree/main/contracts/tracks-auction) | Contract enabling creation and management of auctions for track NFTs, allowing artists to sell their work. |  

## Product design choices

### Permissionless NFT minting, no track-uniqueness check
Unless there is a mechanism to mark trusted artists, we have to make minting NFTs open to anyone, while making sure
nobody can come and 'block' a track by minting it first, even though they may not be its rightful owner.
Artists have to use traditional channels to communicate to their audience which NFTs are legit.

### No contract fees
Real-world application would almost certainly have some form of fees. The feature was not requested, so it was scoped
out in the interest of time.

### No auction spam prevention
There is no maximum number of open auctions, auction deposit, maximum auction duration, etc. In the real world, those
mechanisms would be necessary to avoid malicious spam.

## Technical implementation choices and details

### No bid history kept on-chain
In the interest of saving time, bid history wasn't implemented, even though it would be useful in real-world scenario.
Although, arguably, it would best be kept in an indexer, to keep the contract as light and performant as possible.

### No OpenSea metadata standard compatibility
Due to no such requirement, [OpenSea compatibility](https://docs.opensea.io/docs/metadata-standards) is absent. 
In the real world, it would be advisable to implement it.

### NFTs are escrowed during auctions
This means that e.g. creating and canceling an auction will return the NFT, but it will erase any allowances or other
operators of that NFT.

### Custom 'asset' structure instead of cw-assets
PriceAsset structure introduced because CW1155 wouldn't be supported, so cw-asset wasn't as good of a fit.

### Global setting for auction price denomination
Although this could be defined per-auction, it makes more sense to keep it uniform across all auctions.

### Using CW20 and CW721 callback hooks instead of allowance
Using CW20 and CW721 allowances would enable us to skip hooks and use 'regular' execute messages. However, I picked a
more 'traditional' approach.

### No handles for changing configuration
tracks-auction features no admin or handles to change configuration. This was chosen to keep the scope smaller, although
a true production contract would need this.