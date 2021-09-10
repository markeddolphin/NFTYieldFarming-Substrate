package socialtoken

import (
	"encoding/json"

	"github.com/Get-Cache/Privi/contracts/coinbalance"
	"github.com/Get-Cache/Privi/utils"
	"github.com/hyperledger/fabric/core/chaincode/shim"
	"github.com/hyperledger/fabric/protos/peer"
	"github.com/shopspring/decimal"
)

type SmartContract struct{}

/* -------------------------------------------------------------------------------------------------
Init:  this function register PRIVI as the Admin of the POD Swapping Smart Contract. It initialises
       the lists of PODs and the Indexes used. Args: array containing a string:
PrivateKeyID           string   // Private Key of the admin of the smart contract
------------------------------------------------------------------------------------------------- */

func (s *SmartContract) Init(stub shim.ChaincodeStubInterface) peer.Response {

	_, args := stub.GetFunctionAndParameters()
	if len(args) > 0 && args[0] == "UPGRADE" {
		return shim.Success(nil)
	}

	return shim.Success(nil)
}

func (s *SmartContract) Invoke(stub shim.ChaincodeStubInterface) peer.Response {
	function, args := stub.GetFunctionAndParameters()

	switch function {
	case "getSocialPoolInfo": // public
		pool, err := GetSocialPoolInfo(stub, args[0])
		if err != nil {
			return shim.Error(err.Error())
		}
		poolBytes, _ := json.Marshal(pool)
		return shim.Success(poolBytes)

	case "getSocialPoolState": // public
		pool, err := GetSocialPoolState(stub, args[0])
		if err != nil {
			return shim.Error(err.Error())
		}
		poolBytes, _ := json.Marshal(pool)
		return shim.Success(poolBytes)

	case "getSocialPoolOfToken": // public
		poolAddress, err := GetSocialPoolOfToken(stub, args[0])
		if err != nil {
			return shim.Error(err.Error())
		}
		poolAddressBytes, _ := json.Marshal(poolAddress)
		return shim.Success(poolAddressBytes)

	case "getSocialTokenPrice": // public
		return GetSocialTokenPrice(stub, args)

	case "getSocialTokenPriceBySymbol": // public
		return getSocialTokenPriceBySymbol(stub, args)

	case "createSocialToken": // secure
		// check args length
		if err := utils.ValidateArgsLen(args, 1); err != nil {
			return utils.Error(err)
		}
		// validate signature
		var input SocialPool
		if err := json.Unmarshal([]byte(args[0]), &input); err != nil {
			return utils.Error(err)
		}
		// invoke function
		return CreateSocialToken(stub, &input)

	case "modifySocialPool": // secure
		// check args length
		if err := utils.ValidateArgsLen(args, 1); err != nil {
			return utils.Error(err)
		}
		// validate signature
		var input ModifySocialPoolRequest
		if err := json.Unmarshal([]byte(args[0]), &input); err != nil {
			return utils.Error(err)
		}
		// invoke function
		return ModifySocialPool(stub, &input)

	case "sellSocialToken": // secure
		// check args length
		if err := utils.ValidateArgsLen(args, 1); err != nil {
			return utils.Error(err)
		}
		// validate signature
		var input SellSocialToken
		if err := json.Unmarshal([]byte(args[0]), &input); err != nil {
			return utils.Error(err)
		}
		// invoke function
		return MakeSellSocialToken(stub, &input)

	case "buySocialToken": // secure
		// check args length
		if err := utils.ValidateArgsLen(args, 1); err != nil {
			return utils.Error(err)
		}
		// validate signature
		var input BuySocialToken
		if err := json.Unmarshal([]byte(args[0]), &input); err != nil {
			return utils.Error(err)
		}
		// invoke function
		return MakeBuySocialToken(stub, &input)

	}

	return utils.NotFound(function)
}

/* -------------------------------------------------------------------------------------------------
createSocialToken: this function initialises a new Social Token with the parameters described below.
             Args is an array containing three json with the following fields:
SocialPool Entity:
              string                        // Public Id of SOCIAL TOKEN creator
AMM                  string                        // Type of AMM to use
SpreadDividend       decimal.Decimal  			   // Spread for dividends
TokenName            string  			  		   // Name of Social Token
TokenSymbol          string               		   // Symbol of Social Token
InitialSupply 	     string				  		   // Supply of Social Token minted on initialize
FundingToken		 string				  		   // Funding token symbol
TokenChain           string                        // Chain of the imported token
DividendFreq 		 string 			  		   // How often dividend will be payed for holders
Date                 decimal.Decimal               // Timestamp of the creation of the pod
//////////////////////////////////////////////////////////////////////////////////////////////////
ADDITIONAL
Hash                 string                   	   // Hash of the transaction ( args[1] )
Signature            string                        // Signature of the transaction ( args[2] )
------------------------------------------------------------------------------------------------- */

func CreateSocialToken(stub shim.ChaincodeStubInterface, input *SocialPool) peer.Response {
	updateSocialPools := make(map[string]SocialPool)
	updateSocialPoolStates := make(map[string]SocialPoolState)

	// // Verify signature address //
	// var publicAddress string
	// publicAddress, err := getAttachedAddress(stub, input.Creator)
	// if err != nil {
	// 	return shim.Error(err.Error())
	// }

	// Get Transaction Date //
	timestamp, err := stub.GetTxTimestamp()
	if err != nil {
		return shim.Error("ERROR: GETTING TIMESTAMP OF THE TRANSACTION. " +
			err.Error())
	}
	date := int64(timestamp.Seconds)
	input.Date = date

	// generate input bytes
	inputBytes, err := json.Marshal(input)
	if err != nil {
		return utils.Error(err)
	}

	// Register SocialPool Address and Social Token in the system  //
	input.PoolAddress = getUniqueAddress(inputBytes, date)
	if err := coinbalance.RegisterAddress(stub, input.PoolAddress, coinbalance.SocialTokenAddressType); err != nil {
		return shim.Error(err.Error())
	}

	// Register social token and mint initial supply to user //
	updateTokens, transactions, err := registerSocialToken(stub, *input, input.Creator)
	if err != nil {
		return shim.Error(err.Error())
	}

	// Register new Social Pool //
	err = updateSocialPoolInfo(stub, *input)
	if err != nil {
		return shim.Error(err.Error())
	}
	updateSocialPools[input.PoolAddress] = *input

	// Initialise state of the Pool
	poolState := SocialPoolState{
		SupplyReleased: input.InitialSupply,
		DividendFunds:  decimal.Zero,
	}
	err = updateSocialTokenState(stub, poolState, input.PoolAddress)
	if err != nil {
		return shim.Error(err.Error())
	}
	updateSocialPoolStates[input.PoolAddress] = poolState

	// Generate output //
	return generateOutput(updateSocialPools, updateSocialPoolStates,
		updateTokens, transactions)
}

/* -------------------------------------------------------------------------------------------------
modifySocialPool: this function initialises a new Social Token with the parameters described below.
             Args is an array containing three json with the following fields:
------------------------------------------------------------------------------------------------- */

func ModifySocialPool(stub shim.ChaincodeStubInterface, input *ModifySocialPoolRequest) peer.Response {

	var pool SocialPool
	pool, err := GetSocialPoolInfo(stub, input.Address)
	if err != nil {
		return shim.Error(err.Error())
	}

	pool.FundingToken = input.FundingToken

	// Register new Social Pool //
	err = updateSocialPoolInfo(stub, pool)
	if err != nil {
		return shim.Error(err.Error())
	}

	// Generate output //
	return shim.Success(nil)
}

/* -------------------------------------------------------------------------------------------------
sellSocialToken: this function is called when an Investor wants to sell some of its social tokens.
Investor                string               		// Id of the investor
PoolAddress       		string  			  	    // Address of the social pool
Amount                  decimal.Decimal             // Amount of social tokens to sell
Hash                    string                   	// Hash of the transaction ( args[2] )
Signature               string                      // Signature of the transaction ( args[3] )
------------------------------------------------------------------------------------------------- */

func MakeSellSocialToken(stub shim.ChaincodeStubInterface, input *SellSocialToken) peer.Response {

	updateSocialPoolStates := make(map[string]SocialPoolState)

	// // Verify signature //
	// var publicAddress string
	// publicAddress, err := getAttachedAddress(stub, input.Investor)
	// if err != nil {
	// 	return shim.Error(err.Error())
	// }

	// Retrieve pod info and pod state //
	var pool SocialPool
	pool, err := GetSocialPoolInfo(stub, input.PoolAddress)
	if err != nil {
		return shim.Error(err.Error())
	}
	var poolState SocialPoolState
	poolState, err = GetSocialPoolState(stub, input.PoolAddress)
	if err != nil {
		return shim.Error(err.Error())
	}

	// Get the amount of funding token what we need to receive given amount of social tokens //
	fundingAmount, err := selling_social_tokens(pool.AMM, poolState.SupplyReleased,
		pool.InitialSupply, input.Amount, pool.SpreadDividend, pool.TargetPrice, pool.TargetSupply)
	if err != nil {
		return shim.Error(err.Error())
	}
	poolState.SupplyReleased, err = saveSubstraction(poolState.SupplyReleased, input.Amount)
	if err != nil {
		return shim.Error(err.Error())
	}

	// Burn Social Tokens //
	burningSocialToken := coinbalance.TransferRequest{
		Type:   "Social_Token_Burning",
		Token:  pool.TokenSymbol,
		Amount: input.Amount,
		From:   input.Investor,
		To:     pool.PoolAddress,
	}
	transactions, err := burnSocialPoolTokens(stub, &burningSocialToken)
	if err != nil {
		return shim.Error(err.Error())
	}

	// Transfer the funding token to Seller //
	sellingTransfer := coinbalance.TransferRequest{
		Type:   "Social_Token_Selling",
		Token:  pool.FundingToken,
		Amount: fundingAmount,
		From:   pool.PoolAddress,
		To:     input.Investor,
	}

	transactions2, err := multiTransfer(stub, sellingTransfer)
	if err != nil {
		return shim.Error(err.Error())
	}
	transactions = append(transactions, transactions2[:]...)

	// Update pool state //
	err = updateSocialTokenState(stub, poolState, pool.PoolAddress)
	if err != nil {
		return shim.Error(err.Error())
	}
	updateSocialPoolStates[pool.PoolAddress] = poolState

	// Generate output //
	return generateOutput(nil, updateSocialPoolStates, nil, transactions)
}

/* -------------------------------------------------------------------------------------------------
buySocialToken: this function is called when an Investor wants to buy some of its social tokens.
Investor             string                        // Id of the investor
PoolAddress          string  			           // Address of the social pool
Amount               decimal.Decimal               // Amount of social tokens to buy
Hash                 string                   	   // Hash of the transaction
Signature            string                        // Signature of the transaction
------------------------------------------------------------------------------------------------- */

func MakeBuySocialToken(stub shim.ChaincodeStubInterface, input *BuySocialToken) peer.Response {
	updateSocialPoolStates := make(map[string]SocialPoolState)

	// Verify signature //
	// var publicAddress string
	// publicAddress, err := getAttachedAddress(stub, input.Investor)
	// if err != nil {
	// 	return shim.Error(err.Error())
	// }

	// Retrieve pod info and pod state //
	var pool SocialPool
	pool, err := GetSocialPoolInfo(stub, input.PoolAddress)
	if err != nil {
		return shim.Error(err.Error())
	}
	var poolState SocialPoolState
	poolState, err = GetSocialPoolState(stub, input.PoolAddress)
	if err != nil {
		return shim.Error(err.Error())
	}

	// Get the amount of funding token what we need to buy given amount of social tokens //
	fundingAmount, err := buyingSocialTokens(pool.AMM, poolState.SupplyReleased,
		pool.InitialSupply, input.Amount, pool.TargetPrice, pool.TargetSupply)
	if err != nil {
		return shim.Error(err.Error())
	}
	poolState.SupplyReleased = poolState.SupplyReleased.Add(input.Amount)
	poolState.DividendFunds = poolState.DividendFunds.Add(fundingAmount.Mul(pool.SpreadDividend))

	// Transfer the funding token from purchaser //
	buyingTransfer := coinbalance.TransferRequest{
		Type:   "Social_Token_Buying",
		Token:  pool.FundingToken,
		Amount: fundingAmount,
		From:   input.Investor,
		To:     pool.PoolAddress,
	}

	transactions, err := multiTransfer(stub, buyingTransfer)
	if err != nil {
		return shim.Error(err.Error())
	}

	// Mint Social Tokens //
	mintingSocialToken := coinbalance.TransferRequest{
		Type:   "Social_Token_Minting",
		Token:  pool.TokenSymbol,
		Amount: input.Amount,
		From:   pool.PoolAddress,
		To:     input.Investor,
	}
	transactions2, err := mintSocialPoolTokens(stub, &mintingSocialToken)
	if err != nil {
		return shim.Error(err.Error())
	}
	transactions = append(transactions, transactions2[:]...)

	// Update pool state //
	err = updateSocialTokenState(stub, poolState, pool.PoolAddress)
	if err != nil {
		return shim.Error(err.Error())
	}
	updateSocialPoolStates[pool.PoolAddress] = poolState

	// Generate output //
	return generateOutput(nil, updateSocialPoolStates, nil, transactions)

}

/* -------------------------------------------------------------------------------------------------
getSocialTokenPrice: this function is called to get the market price of a pod token
PodAddress              string  			  // Id of the POD to delete
------------------------------------------------------------------------------------------------- */

func GetSocialTokenPrice(stub shim.ChaincodeStubInterface,
	args []string) peer.Response {

	// Retrieve pod info and pod state //
	pool, err := GetSocialPoolInfo(stub, args[0])
	if err != nil {
		return shim.Error(err.Error())
	}
	var poolState SocialPoolState
	poolState, err = GetSocialPoolState(stub, args[0])
	if err != nil {
		return shim.Error(err.Error())
	}

	// Get market price //
	marketPrice, err := getMarketPrice(pool.AMM, poolState.SupplyReleased, pool.InitialSupply,
		pool.TargetPrice, pool.TargetSupply)
	if err != nil {
		return shim.Error(err.Error())
	}

	res, err := json.Marshal(marketPrice)
	if err != nil {
		return shim.Error(err.Error())
	}
	return shim.Success(res)
}

/* -------------------------------------------------------------------------------------------------
getSocialTokenPriceBySymbol: this function is called to get the market price of a pod token
TokenSymbol              string  			  // Symbol of the social token
------------------------------------------------------------------------------------------------- */

func getSocialTokenPriceBySymbol(stub shim.ChaincodeStubInterface,
	args []string) peer.Response {

	poolAddress, err := GetSocialPoolOfToken(stub, args[0])
	if err != nil {
		return shim.Error(err.Error())
	}

	// Retrieve pod info and pod state //
	pool, err := GetSocialPoolInfo(stub, poolAddress)
	if err != nil {
		return shim.Error(err.Error())
	}
	var poolState SocialPoolState
	poolState, err = GetSocialPoolState(stub, poolAddress)
	if err != nil {
		return shim.Error(err.Error())
	}

	// Get market price //
	marketPrice, err := getMarketPrice(pool.AMM, poolState.SupplyReleased, pool.InitialSupply,
		pool.TargetPrice, pool.TargetSupply)
	if err != nil {
		return shim.Error(err.Error())
	}

	output := PriceBySymbol{
		FundingToken: pool.FundingToken,
		Price:        marketPrice,
	}

	res, err := json.Marshal(output)
	if err != nil {
		return shim.Error(err.Error())
	}

	return shim.Success(res)
}

/* -------------------------------------------------------------------------------------------------
------------------------------------------------------------------------------------------------- */