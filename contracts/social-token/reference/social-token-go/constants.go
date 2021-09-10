///////////////////////////////////////////////////////////////////
// File containing the constants for the Cache Coin Smart Contract
///////////////////////////////////////////////////////////////////

package socialtoken

import (
	"math"

	"github.com/shopspring/decimal"
)

/*--------------------------------------------------
 SMART CONTRACT INDEXES
--------------------------------------------------*/

const IndexSocialPools = "SOCIAL_POOL"
const IndexSocialPoolStates = "SOCIAL_POOL_STATES"

/*--------------------------------------------------
 SYSTEM ROLES
--------------------------------------------------*/

const ADMIN_ROLE = "ADMIN"
const USER_ROLE = "USER"
const BUSINESS_ROLE = "BUSINESS"
const GUARANTOR_ROLE = "GUARANTOR"
const COURTMEMBER_ROLE = "COURT_MEMBER"
const EXCHANGE_ROLE = "EXCHANGE"

/*--------------------------------------------------
 AMM TYPES
--------------------------------------------------*/

const QUADRATIC_AMM = "QUADRATIC"
const LINEAR_AMM = "LINEAR"
const EXPONENTIAL_AMM = "EXPONENTIAL"
const SIGMOID_AMM = "SIGMOID"

var AMM_TYPES = []string{
	QUADRATIC_AMM,
	LINEAR_AMM,
	EXPONENTIAL_AMM,
	SIGMOID_AMM}

/*--------------------------------------------------
 TOKEN TYPES
--------------------------------------------------*/

const CRYPTO_TOKEN = "CRYPTO"
const SOCIAL_TOKEN = "SOCIAL"
const FT_POD_TOKEN = "FTPOD"
const NFT_POD_TOKEN = "NFTPOD"

var TOKEN_TYPES = []string{
	CRYPTO_TOKEN, SOCIAL_TOKEN,
	FT_POD_TOKEN, NFT_POD_TOKEN}

/*--------------------------------------------------
 PAYMENT FREQUENCY
--------------------------------------------------*/

const DAILY_PAYMENT = "DAILY"
const WEEKLY_PAYMENT = "WEEKLY"
const MONTHLY_PAYMENT = "MONTHLY"

/*--------------------------------------------------
 SMART CONTRACT INVOKATIONS
--------------------------------------------------*/

const COIN_BALANCE_CHAINCODE = "CoinBalance"
const CHANNEL_NAME = "broadcast"
const DATA_PROTOCOL_CHAINCODE = "DataProtocol"
const SOCIAL_TOKEN_ADDRESS_TYPE = "SOCIAL_TOKEN"

/*--------------------------------------------------
 DECIMAL CONSTANTS
--------------------------------------------------*/

var ONE_DECIMAL = decimal.NewFromInt(1)
var TWO_DECIMAL = decimal.NewFromInt(2)
var THREE_DECIMAL = decimal.NewFromInt(3)
var EXP_DECIMAL = decimal.NewFromFloat(math.E)

/*--------------------------------------------------
--------------------------------------------------*/