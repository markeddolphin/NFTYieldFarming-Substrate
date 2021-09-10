package socialtoken

import (
	"errors"
	"math"

	"github.com/shopspring/decimal"
)

/* -------------------------------------------------------------------------------------------------
integral: this function determines the integral of the AMM curve.
------------------------------------------------------------------------------------------------- */

func integral(AMM string, upperBound decimal.Decimal, lowerBound decimal.Decimal,
	targetPrice decimal.Decimal, targetSupply decimal.Decimal) (decimal.Decimal, error) {
	switch AMM {

	case LINEAR_AMM:
		multiplier := targetPrice.Div(targetSupply)
		integral, err := saveSubstraction(upperBound.Pow(TWO_DECIMAL), lowerBound.Pow(TWO_DECIMAL))
		return multiplier.Mul(integral).Div(TWO_DECIMAL), err

	case QUADRATIC_AMM:
		multiplier := targetPrice.Div(targetSupply.Pow(TWO_DECIMAL))
		integral, err := saveSubstraction(upperBound.Pow(THREE_DECIMAL), lowerBound.Pow(THREE_DECIMAL))
		return multiplier.Mul(integral).Div(THREE_DECIMAL), err

	case EXPONENTIAL_AMM:
		multiplier := targetPrice.Mul(Exponent(EXP_DECIMAL, targetSupply.Neg()))
		integral, err := saveSubstraction(Exponent(EXP_DECIMAL, upperBound), Exponent(EXP_DECIMAL, lowerBound))
		return multiplier.Mul(integral), err

	case SIGMOID_AMM:
		// upper := (upperBound + math.Log(1+math.Exp(-upperBound+targetSupply)))
		upperExpFloat, _ := (Exponent(EXP_DECIMAL, (upperBound.Neg().Add(targetSupply))).Add(ONE_DECIMAL)).Float64()
		upper := (decimal.NewFromFloat(math.Log(upperExpFloat))).Add(upperBound)
		// lower := (lowerBound + math.Log(1+math.Exp(-lowerBound+targetSupply)))
		lowerExpFloat, _ := (Exponent(EXP_DECIMAL, (lowerBound.Neg().Add(targetSupply))).Add(ONE_DECIMAL)).Float64()
		lower := (decimal.NewFromFloat(math.Log(lowerExpFloat))).Add(lowerBound)
		integral, err := saveSubstraction(upper, lower)
		return targetPrice.Div(TWO_DECIMAL).Mul(integral), err
	}
	return decimal.Zero, errors.New("ERROR COMPUTING THE INTEGRAL. ")
}

/* -------------------------------------------------------------------------------------------------
marketPrice: this function determines the market price.
------------------------------------------------------------------------------------------------- */

func getMarketPrice(AMM string, supplyReleased decimal.Decimal, initialSupply decimal.Decimal,
	targetPrice decimal.Decimal, targetSupply decimal.Decimal) (decimal.Decimal, error) {

	effectiveSupply := decimal.Max(decimal.Zero, supplyReleased.Sub(initialSupply))
	// if err != nil {
	// 	return decimal.Zero, err
	// }

	switch AMM {
	case LINEAR_AMM:
		multiplier := targetPrice.Div(targetSupply)
		return multiplier.Mul(effectiveSupply), nil

	case QUADRATIC_AMM:
		multiplier := targetPrice.Div(targetSupply.Pow(TWO_DECIMAL))
		return multiplier.Mul(effectiveSupply.Pow(TWO_DECIMAL)), nil

	case EXPONENTIAL_AMM:
		multiplier := targetPrice.Mul(Exponent(EXP_DECIMAL, targetSupply.Neg()))
		return multiplier.Mul(EXP_DECIMAL.Pow(supplyReleased)), nil

	case SIGMOID_AMM:
		// 	return targetPrice * (1. / (1 + math.Exp(-effectiveSupply+targetSupply))), nil
		return targetPrice.Mul(ONE_DECIMAL.Div(Exponent(EXP_DECIMAL, (effectiveSupply.Neg().Add(targetSupply))).Add(ONE_DECIMAL))), nil
	}
	return decimal.Zero, errors.New("ERROR COMPUTING GETTING THE MARKET PRICE. ")
}

/* -------------------------------------------------------------------------------------------------
buyingSocialTokens: this function determines the amount of X of Funding Tokens to receive after an
                 investment of Y Pod Tokens
------------------------------------------------------------------------------------------------- */
func buyingSocialTokens(AMM string, supplyReleased decimal.Decimal, initialSupply decimal.Decimal,
	amount decimal.Decimal, targetPrice decimal.Decimal, targetSupply decimal.Decimal) (decimal.Decimal, error) {
	effectiveSupply := decimal.Max(decimal.Zero, supplyReleased.Sub(initialSupply))
	//effectiveSupply, err := saveSubstraction(supplyReleased, initialSupply)
	// if err != nil {
	// 	return decimal.Zero, err
	// }
	newSupply := effectiveSupply.Add(amount)
	fundingAmount, err := integral(AMM, newSupply, effectiveSupply, targetPrice, targetSupply)
	return fundingAmount, err
}

/* -------------------------------------------------------------------------------------------------
selling_social_tokens: this function determines the amount of X of Funding Tokens to give after
               selling Y Funding Tokens
------------------------------------------------------------------------------------------------- */
func selling_social_tokens(AMM string, supplyReleased decimal.Decimal, initialSupply decimal.Decimal,
	sellingAmount decimal.Decimal, spread decimal.Decimal, targetPrice decimal.Decimal, targetSupply decimal.Decimal) (decimal.Decimal, error) {

	// Compute supply left after selling it //
	effectiveSupply := decimal.Max(decimal.Zero, supplyReleased.Sub(initialSupply))
	// effectiveSupply, err := saveSubstraction(supplyReleased, initialSupply)
	// if err != nil {
	// 	return decimal.Zero, err
	// }
	lowSupply := decimal.Max(decimal.Zero, effectiveSupply.Div(sellingAmount))

	fundingAmount, err := integral(AMM, effectiveSupply, lowSupply, targetPrice, targetSupply)
	if err != nil {
		return decimal.Zero, err
	}
	return fundingAmount.Mul(ONE_DECIMAL.Sub(spread)), err
}

/* -------------------------------------------------------------------------------------------------
------------------------------------------------------------------------------------------------- */