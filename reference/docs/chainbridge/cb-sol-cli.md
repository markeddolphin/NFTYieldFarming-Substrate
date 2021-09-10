Contract Addresses
================================================================
Bridge:             0x75A5Bc3CBcc23f55725C3182D358513984253BA1
----------------------------------------------------------------
Erc20 Handler:      0x72D5a11350A004C075867e99029D28ac2B591458
----------------------------------------------------------------
Erc721 Handler:     0xdA40274eB479F3496b0044cc85Dc65Bbc5909dE4
----------------------------------------------------------------
Generic Handler:    0x662F644cdc9BD4e53ebEb31B476Ef10EAa9Ae7b9
----------------------------------------------------------------
Erc20:              0x6916A7Ac3D759B0E514450b132Bc651e8008874C
----------------------------------------------------------------
Erc721:             0x1D090369f1B3Ad4bd5b25c2DF360aC3af5fdCa22
----------------------------------------------------------------
Centrifuge Asset:   Not Deployed
----------------------------------------------------------------
WETC:               Not Deployed
================================================================


# Register fungible resource ID with erc20 contract
cb-sol-cli bridge register-resource --bridge "0x75A5Bc3CBcc23f55725C3182D358513984253BA1" --resourceId "0x000000000000000000000000000000c76ebe4a02bbc34786d860b355f5a5ce00" --targetContract "0x6916A7Ac3D759B0E514450b132Bc651e8008874C" --handler "0x72D5a11350A004C075867e99029D28ac2B591458"

# Register non-fungible resource ID with erc721 contract
cb-sol-cli bridge register-resource --resourceId "0x000000000000000000000000000000e389d61c11e5fe32ec1735b3cd38c69501" --targetContract "0x1D090369f1B3Ad4bd5b25c2DF360aC3af5fdCa22" --handler "0xdA40274eB479F3496b0044cc85Dc65Bbc5909dE4"

# Register generic resource ID
cb-sol-cli bridge register-generic-resource --resourceId "0x000000000000000000000000000000f44be64d2de895454c3467021928e55e01" --targetContract "0xc279648CE5cAa25B9bA753dAb0Dfef44A069BaF4" --handler "0x662F644cdc9BD4e53ebEb31B476Ef10EAa9Ae7b9" --hash --deposit "" --execute "store(bytes32)"
# need to check about the targetContract of generic resource ID

Specify Token Semantics¶
To allow for a variety of use cases, the Ethereum contracts support both the transfer and the mint/burn ERC methods.

For simplicity's sake the following examples only make use of the mint/burn method:

# Register the erc20 contract as mintable/burnable
cb-sol-cli bridge set-burn --bridge "0x75A5Bc3CBcc23f55725C3182D358513984253BA1" --handler "0x72D5a11350A004C075867e99029D28ac2B591458" --tokenContract "0x6916A7Ac3D759B0E514450b132Bc651e8008874C"

# Register the associated handler as a minter
cb-sol-cli erc20 add-minter --erc20Address "0x6916A7Ac3D759B0E514450b132Bc651e8008874C" --minter "0x72D5a11350A004C075867e99029D28ac2B591458"

# Register the erc721 contract as mintable/burnable
cb-sol-cli bridge set-burn --bridge "0x75A5Bc3CBcc23f55725C3182D358513984253BA1" --tokenContract "0x1D090369f1B3Ad4bd5b25c2DF360aC3af5fdCa22" --handler "0xdA40274eB479F3496b0044cc85Dc65Bbc5909dE4"

# Add the handler as a minter
cb-sol-cli erc721 add-minter --erc721Address "0x1D090369f1B3Ad4bd5b25c2DF360aC3af5fdCa22" --minter "0xdA40274eB479F3496b0044cc85Dc65Bbc5909dE4"

# You can query the recipients balance on ethereum with this:

cb-sol-cli erc20 balance --address "0xeC44513a4204b031d5A6D562E09cf0a229e35ae5" --erc20Address "0x6916A7Ac3D759B0E514450b132Bc651e8008874C"

ERC20 => Substrate Native Token

cb-sol-cli erc20 mint --erc20Address 0x6916A7Ac3D759B0E514450b132Bc651e8008874C --amount 1000
cb-sol-cli erc20 approve --amount 1000 --recipient "0x72D5a11350A004C075867e99029D28ac2B591458" --erc20Address 0x6916A7Ac3D759B0E514450b132Bc651e8008874C
cb-sol-cli erc20 deposit --amount 1 --dest 1 --recipient "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d" --resourceId "0x000000000000000000000000000000c76ebe4a02bbc34786d860b355f5a5ce00" --bridge "0x75A5Bc3CBcc23f55725C3182D358513984253BA1"

ERC721 ⇒ Substrate NFT

cb-sol-cli erc721 mint --erc721Address "0x1D090369f1B3Ad4bd5b25c2DF360aC3af5fdCa22" --id 0x11
cb-sol-cli erc721 approve --id 0x11 --recipient "0xdA40274eB479F3496b0044cc85Dc65Bbc5909dE4" --erc721Address "0x1D090369f1B3Ad4bd5b25c2DF360aC3af5fdCa22"
cb-sol-cli erc721 deposit --id 0x11 --dest 1 --resourceId "0x000000000000000000000000000000e389d61c11e5fe32ec1735b3cd38c69501" --recipient "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d" --bridge "0x75A5Bc3CBcc23f55725C3182D358513984253BA1"