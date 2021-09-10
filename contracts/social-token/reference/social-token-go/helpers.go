/*--------------------------------------------------------------------------
/*--------------------------------------------------------------------------
----------------------------------------------------------------------------
   HELPER FUNCTIONS CALLED SEVERAL TIMES ON MAIN SMART CONTRACT FUNCIOTNS
----------------------------------------------------------------------------
-------------------------------------------------------------------------- */

package socialtoken

import (
	"encoding/json"
	"errors"
	"fmt"

	"github.com/Get-Cache/Privi/contracts/coinbalance"
	"github.com/hyperledger/fabric/core/chaincode/shim"
	pb "github.com/hyperledger/fabric/protos/peer"
)

/* -------------------------------------------------------------------------------------------------
getAttachedAddress: this function returns the address attached to a given userID.
------------------------------------------------------------------------------------------------- */

func getAttachedAddress(stub shim.ChaincodeStubInterface, address string) (string, error) {
	actor, err := coinbalance.GetUser(stub, address)
	if err != nil {
		return "", err
	}
	return actor.PublicAddress, nil
}

/* -------------------------------------------------------------------------------------------------
getSocialPoolInfo: this function returns the information of a given pool. It takes as input the
                 address for the pool.
------------------------------------------------------------------------------------------------- */

func GetSocialPoolInfo(stub shim.ChaincodeStubInterface,
	address string) (SocialPool, error) {

	// Retrieve wallet of an user from Blockchain //
	var pool SocialPool
	poolBytes, err := stub.GetState(IndexSocialPools + address)
	if err != nil {
		return pool, errors.New("ERROR: GETTING THE POOL INFO. " + err.Error())
	}
	if poolBytes == nil {
		return pool, errors.New("ERROR: POOL " + address + " NOT REGISTERED.")
	}
	err = json.Unmarshal(poolBytes, &pool)
	if err != nil {
		return pool, err
	}
	return pool, nil
}

/* -------------------------------------------------------------------------------------------------
getSocialPoolState: this function returns the state of a given pool. It takes as input the
                 address for the pool.
------------------------------------------------------------------------------------------------- */

func GetSocialPoolState(stub shim.ChaincodeStubInterface,
	address string) (SocialPoolState, error) {

	// Retrieve wallet of an user from Blockchain //
	var poolState SocialPoolState
	poolStateBytes, err := stub.GetState(IndexSocialPoolStates + address)
	if err != nil {
		return poolState, errors.New("ERROR: GETTING THE POOL STATE. " + err.Error())
	}
	if poolStateBytes == nil {
		return poolState, errors.New("ERROR: POOL STATE " + address + " NOT REGISTERED.")
	}
	err = json.Unmarshal(poolStateBytes, &poolState)
	if err != nil {
		return poolState, err
	}
	return poolState, nil
}

/* -------------------------------------------------------------------------------------------------
updateSocialPoolInfo: this function updates a Pool info on Blockchain. Inputs: the updated pool.
------------------------------------------------------------------------------------------------- */

func updateSocialPoolInfo(stub shim.ChaincodeStubInterface,
	pool SocialPool) error {
	// Store pool on Blockchain //
	poolBytes, _ := json.Marshal(pool)
	err := stub.PutState(IndexSocialPools+pool.PoolAddress, poolBytes)
	if err != nil {
		return errors.New("ERROR: UPDATING THE POOL INFO " +
			pool.PoolAddress + ". " + err.Error())
	}
	return nil
}

/* -------------------------------------------------------------------------------------------------
updateSocialPoolState: this function updates a pool  on Blockchain. Inputs: the updated pool.
------------------------------------------------------------------------------------------------- */

func updateSocialTokenState(stub shim.ChaincodeStubInterface,
	state SocialPoolState, address string) error {
	// Store pool on Blockchain //
	stateBytes, _ := json.Marshal(state)
	err := stub.PutState(IndexSocialPoolStates+address, stateBytes)
	if err != nil {
		return errors.New("ERROR: UPDATING THE STATE OF SOCIAL POOL " +
			address + ". " + err.Error())
	}
	return nil
}

/* -------------------------------------------------------------------------------------------------
registerSocialToken: this function register a new social token in the system
------------------------------------------------------------------------------------------------- */

func registerSocialToken(stub shim.ChaincodeStubInterface, input SocialPool, address string) (map[string]coinbalance.Token, []coinbalance.Transfer, error) {

	// Define Pool Token //
	poolToken := coinbalance.Token{
		Name: input.TokenName, TokenType: SOCIAL_TOKEN,
		Symbol: input.TokenSymbol, Supply: input.InitialSupply,
		LockUpDate: input.LockUpDate}

	// Register Pool Token on CoinBalance Chaincode //
	r, err := coinbalance.RegisterToken(stub, &poolToken, address)

	if err != nil {
		return nil, []coinbalance.Transfer{}, err
	}

	return r.UpdateTokens, r.Transactions, nil
}

/* -------------------------------------------------------------------------------------------------
registerAddress: this function register a balance
------------------------------------------------------------------------------------------------- */

/* -------------------------------------------------------------------------------------------------
multiTransfer: this function computes all the transfers taking place on the smart contract
------------------------------------------------------------------------------------------------- */

func multiTransfer(stub shim.ChaincodeStubInterface, multitransfers ...coinbalance.TransferRequest) ([]coinbalance.Transfer, error) {
	r, err := coinbalance.Multitransfer(stub, multitransfers...)
	if err != nil {
		return []coinbalance.Transfer{}, err
	}
	return r.Transactions, nil
}

/* -------------------------------------------------------------------------------------------------
mintSocialPoolTokens: this function mints pool tokens for a user
------------------------------------------------------------------------------------------------- */

func mintSocialPoolTokens(stub shim.ChaincodeStubInterface, input *coinbalance.TransferRequest) ([]coinbalance.Transfer, error) {
	r, err := coinbalance.Mint(stub, input)
	if err != nil {
		return []coinbalance.Transfer{}, err
	}
	return r.Transactions, nil
}

/* -------------------------------------------------------------------------------------------------
burnSocialPoolTokens: this function mints pool tokens for a user
------------------------------------------------------------------------------------------------- */

func burnSocialPoolTokens(stub shim.ChaincodeStubInterface, input *coinbalance.TransferRequest) ([]coinbalance.Transfer, error) {
	r, err := coinbalance.Burn(stub, input)
	if err != nil {
		return []coinbalance.Transfer{}, err
	}
	return r.Transactions, nil
}

/* -------------------------------------------------------------------------------------------------
getSocialPoolOfToken: returns the social pool address given the token symbol
------------------------------------------------------------------------------------------------- */

func GetSocialPoolOfToken(stub shim.ChaincodeStubInterface,
	tokenSymbol string) (string, error) {
	queryString := fmt.Sprintf(`{"selector":{"TokenSymbol":"%s"}}`, tokenSymbol)
	it, err := stub.GetQueryResult(queryString)
	if err != nil {
		return "", errors.New("ERROR: unable to get an iterator over the social tokens.")
	}
	defer it.Close()
	pool := SocialPool{}
	for it.HasNext() {
		response, error := it.Next()
		if error != nil {
			message := fmt.Sprintf("unable to get the next element: %s", error.Error())
			return "", errors.New(message)
		}
		if err = json.Unmarshal(response.Value, &pool); err != nil {
			message := fmt.Sprintf("ERROR: unable to parse the response: %s", err.Error())
			return "", errors.New(message)
		}
	}
	return pool.PoolAddress, nil
}

/* -------------------------------------------------------------------------------------------------
generateOutput: this function generates the output.
------------------------------------------------------------------------------------------------- */

func generateOutput(
	pools map[string]SocialPool,
	poolStates map[string]SocialPoolState,
	tokens map[string]coinbalance.Token,
	transactions []coinbalance.Transfer) pb.Response {

	// Output object //
	output := Output{
		UpdateSocialPools:      pools,
		UpdateSocialPoolStates: poolStates,
		UpdateTokens:           tokens,
		Transactions:           transactions,
	}
	outputBytes, err := json.Marshal(output)

	if err != nil {
		return shim.Error("ERROR: GENERATING OUTPUT " + err.Error())
	}
	return shim.Success(outputBytes)
}

/* ------------------------------------------------------------------------------------------------
------------------------------------------------------------------------------------------------- */