///////////////////////////////////////////////////////////////
// File containing the model structs for the POD definition,
// and blockchain inputs and outputs
///////////////////////////////////////////////////////////////

package socialtoken

import (
	"github.com/Get-Cache/Privi/contracts/coinbalance"
	"github.com/shopspring/decimal"
)

// Define instance of a Pod Swap on Blockchain //
type Output struct {
	UpdateSocialPools      map[string]SocialPool        `json:"UpdateSocialPools"`
	UpdateSocialPoolStates map[string]SocialPoolState   `json:"UpdateSocialPoolStates"`
	UpdateTokens           map[string]coinbalance.Token `json:"UpdateTokens"`
	Transactions           []coinbalance.Transfer       `json:"Transactions"`
}

/*---------------------------------------------------------------------------
SMART CONTRACT MODELS FOR SOCIAL TOKENS
-----------------------------------------------------------------------------*/

// Define model of Social Token creation //
type SocialPool struct {
	Creator        string          `json:"Creator"`
	PoolAddress    string          `json:"PoolAddress"`
	AMM            string          `json:"AMM"`
	SpreadDividend decimal.Decimal `json:"SpreadDividend"`
	TokenSymbol    string          `json:"TokenSymbol"`
	TokenName      string          `json:"TokenName"`
	InitialSupply  decimal.Decimal `json:"InitialSupply"`
	FundingToken   string          `json:"FundingToken"`
	DividendFreq   string          `json:"DividendFreq"`
	LockUpDate     int64           `json:"LockUpDate"`
	TargetSupply   decimal.Decimal `json:"TargetSupply"`
	TargetPrice    decimal.Decimal `json:"TargetPrice"`
	TokenChain     string          `json:"TokenChain"`
	Date           int64           `json:"Date"`
}

// Define model of the state of a POD //
type SocialPoolState struct {
	SupplyReleased decimal.Decimal `json:"SupplyReleased"`
	DividendFunds  decimal.Decimal `json:"DividendFunds"`
}

// Define instance of a buying the social token on Blockchain //
type BuySocialToken struct {
	Investor    string          `json:"Investor"`
	PoolAddress string          `json:"PoolAddress"`
	Amount      decimal.Decimal `json:"Amount"`
	Hash        string          `json:"Hash"`
	Signature   string          `json:"Signature"`
}

// Define instance of a buying the social token on Blockchain //
type SellSocialToken struct {
	Investor    string          `json:"Investor"`
	PoolAddress string          `json:"PoolAddress"`
	Amount      decimal.Decimal `json:"Amount"`
	Hash        string          `json:"Hash"`
	Signature   string          `json:"Signature"`
}

// Definition of Price By Symbol in Blockchain //
type PriceBySymbol struct {
	FundingToken string          `json:"FundingToken"`
	Price        decimal.Decimal `json:"Price"`
}

type ModifySocialPoolRequest struct {
	Address      string `json:"Address"`
	FundingToken string `json:"FundingToken"`
}