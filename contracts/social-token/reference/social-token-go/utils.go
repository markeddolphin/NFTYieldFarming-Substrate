package socialtoken

import (
	"crypto/x509"
	"encoding/json"
	"encoding/pem"
	"errors"
	"fmt"
	"math"
	"strconv"
	"time"

	"github.com/Get-Cache/Privi/contracts/coinbalance"
	"github.com/ethereum/go-ethereum/common"
	"github.com/golang/protobuf/proto"
	"github.com/hyperledger/fabric/core/chaincode/shim"
	"github.com/hyperledger/fabric/protos/msp"
	"github.com/shopspring/decimal"
)

/* -------------------------------------------------------------------------------------------------
-------------------------------------------------------------------------------------------------*/

func parsePEM(certPEM string) (*x509.Certificate, error) {
	block, _ := pem.Decode([]byte(certPEM))
	if block == nil {
		return nil, errors.New("Failed to parse PEM certificate")
	}

	return x509.ParseCertificate(block.Bytes)
}

// Extracts CN from an x509 certificate //
func CNFromX509(certPEM string) (string, error) {
	cert, err := parsePEM(certPEM)
	if err != nil {
		return "", errors.New("Failed to parse certificate: " + err.Error())
	}

	return cert.Subject.CommonName, nil
}

// Extracts CN from caller of a chaincode function //
func CallerCN(stub shim.ChaincodeStubInterface) (string, error) {
	data, _ := stub.GetCreator()
	fmt.Println(string(data), "data")
	serializedId := msp.SerializedIdentity{}
	err := proto.Unmarshal(data, &serializedId)
	if err != nil {
		return "", errors.New("Could not unmarshal creator")
	}
	cn, err := CNFromX509(string(serializedId.IdBytes))
	if err != nil {
		return "", err
	}
	fmt.Println(cn, "cn")
	return cn, nil
}

/*--------------------------------------------------
	Convert inputs to chaincodearguments
----------------------------------------------------*/
func ToChaincodeArgs(args []string) [][]byte {
	bargs := make([][]byte, len(args))
	for i, arg := range args {
		bargs[i] = []byte(arg)
	}
	return bargs
}

/* -------------------------------------------------------------------------------------------------
These are utility functions
 -------------------------------------------------------------------------------------------------*/

func getTimeNow() string {
	var formatedTime string
	t := time.Now()
	formatedTime = t.Format(time.RFC1123)
	return formatedTime
}

func stringInSlice(a string, list []string) bool {
	for _, b := range list {
		if b == a {
			return true
		}
	}
	return false
}

func saveSubstraction(main decimal.Decimal, amount decimal.Decimal) (decimal.Decimal, error) {
	main = main.Sub(amount)
	if main.IsNegative() {
		return main, errors.New("ERROR: INSUFFICIENT FUNDS ON BALANCE")
	}
	return main, nil
}

func saveAddition(main decimal.Decimal, amount decimal.Decimal) (decimal.Decimal, error) {
	if main.Add(amount).LessThan(main) {
		return main, errors.New("ERROR: OVERFLOW ON RECEIVER BALANCE")
	}
	main = main.Add(amount)
	return main, nil
}

func checkRange(number decimal.Decimal, lowerBound decimal.Decimal, upperBound decimal.Decimal) bool {
	if number.GreaterThan(upperBound) || number.LessThan(lowerBound) {
		return false
	}
	return true
}

func toStringMethod(object interface{}) string {
	objectBytes, _ := json.Marshal(object)
	return string(objectBytes)
}

func mergeMaps(map1 map[string]coinbalance.Balance,
	map2 map[string]coinbalance.Balance) map[string]coinbalance.Balance {

	for k, v := range map2 {
		map1[k] = v
	}
	return map1
}

func int64ToByte(input int) []byte {
	return []byte(strconv.Itoa(input))
}

func getUniqueAddress(input []byte, currentTime int64) string {
	generator := append(input, int64ToByte(int(currentTime))...)
	return common.BytesToAddress(generator).String()
}

/* -------------------------------------------------------------------------------------------------
-------------------------------------------------------------------------------------------------*/

func Exponent(base decimal.Decimal, exponent decimal.Decimal) decimal.Decimal {

	f1, _ := base.Float64()
	f2, _ := exponent.Float64()

	return decimal.NewFromFloat(math.Pow(f1, f2))
}

/* -------------------------------------------------------------------------------------------------
-------------------------------------------------------------------------------------------------*/