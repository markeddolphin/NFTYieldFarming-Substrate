pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

import "./FundingToken.sol";

contract SocialToken is ERC20, Ownable, ReentrancyGuard {
    enum AMMType {LINEAR, QUADRATIC, EXPONENTIAL, SIGMOID}

    // constants
    uint256 constant eN = 271828 * 1e10; // mathmatical e
    uint256 constant eD = 100000 * 1e10;

    uint256 constant MAX_PERCENTAGE = 100;
    uint256 constant TWO_DECIMAL = 2;
    uint256 constant THREE_DECIMAL = 3;
    uint256 private constant PRECISION = 1e18;

    // token
    AMMType public ammType;
    uint256 public maxSupply;
    uint256 public initialSupply;
    uint256 private tradingSpread;
    uint256 private createDate;

    uint256 private targetPrice;
    uint256 private targetSupply;
    FundingToken public fundingToken;
    uint256 public tradeAccumulatedFee;

    event Minted(uint256 amount, uint256 totalCost);
    event Burned(uint256 amount, uint256 reward);
    event WithdrawSocialToken(address indexed user, uint256 amount);
    event WithdrawFundingToken(address indexed user, uint256 amount);
    event AirdropSocialToken(address indexed user, uint256 amount);
    event RewardEarnings(address indexed user, uint256 amount);

    /**
     * @dev                     Initializes the social token
     * @param _ammType          The AMM type of bonding curve selected. It can be linear, quadratic, exponential and sigmoid
     * @param _tradingSpread    The spread charged for each trade
     * @param _tokenName        The name of the social token
     * @param _tokenSymbol      The symbol of the social token
     * @param _initialSupply    The initialSupply of the social token
     * @param _fundingToken     The fundingToken of the social token
     */
    constructor(
        AMMType _ammType,
        uint256 _tradingSpread,
        string memory _tokenName,
        string memory _tokenSymbol,
        uint256 _initialSupply,
        uint256 _targetPrice,
        uint256 _targetSupply,
        FundingToken _fundingToken
    ) ERC20(_tokenName, _tokenSymbol) {
        maxSupply = 10000000 * (10**18);
        require(_initialSupply <= maxSupply, "Supply is max!");

        ammType = _ammType;
        _mint(address(this), _initialSupply);
        initialSupply = _initialSupply;
        tradingSpread = _tradingSpread;
        fundingToken = _fundingToken;
        createDate = block.timestamp;
        targetPrice = _targetPrice;
        targetSupply = _targetSupply;
    }

    /**
     * @dev                 Calculate the exponent value
     * @param  baseN        The N of base
     * @param  baseD        The D of base
     * @param  exponential  The lowerBound
     */
    function pow(
        uint256 baseN,
        uint256 baseD,
        uint256 exponential
    ) internal returns (uint256) {
        if (exponential == 0) {
            return baseD;
        }
        if (exponential == 1) {
            return baseN;
        }

        uint256 half = exponential / 2;
        uint256 halfValueN = pow(baseN, baseD, half);

        if (half * 2 != exponential) {
            halfValueN = (halfValueN * baseN) / baseD;
        }

        return halfValueN;
    }

    /**
     * @dev                 Calculate the curve integral
     * @param  upperBound   The upperBound
     * @param  lowerBound   The lowerBound
     */
    function curveIntegral(uint256 upperBound, uint256 lowerBound)
        internal
        returns (uint256)
    {
        uint256 multiplier;
        uint256 integral;

        // Calculate integral of t^exponent
        if (ammType == AMMType.LINEAR) {
            multiplier = (PRECISION * targetPrice) / targetSupply;
            if (upperBound**TWO_DECIMAL > lowerBound**TWO_DECIMAL) {
                integral =
                    (upperBound**TWO_DECIMAL - lowerBound**TWO_DECIMAL) /
                    PRECISION;
            } else {
                integral =
                    (lowerBound**TWO_DECIMAL - upperBound**TWO_DECIMAL) /
                    PRECISION;
            }

            return (multiplier * integral) / (TWO_DECIMAL * PRECISION);
        }
        if (ammType == AMMType.QUADRATIC) {
            multiplier =
                ((PRECISION**TWO_DECIMAL) * targetPrice) /
                (targetSupply**TWO_DECIMAL);

            if (upperBound**THREE_DECIMAL > lowerBound**THREE_DECIMAL) {
                integral =
                    (upperBound**THREE_DECIMAL - lowerBound**THREE_DECIMAL) /
                    PRECISION**TWO_DECIMAL;
            } else {
                integral =
                    (lowerBound**THREE_DECIMAL - upperBound**THREE_DECIMAL) /
                    PRECISION**TWO_DECIMAL;
            }

            return
                (multiplier * integral) /
                (THREE_DECIMAL * (PRECISION**TWO_DECIMAL));
        }
        if (ammType == AMMType.EXPONENTIAL) {
            multiplier = PRECISION * targetPrice * pow(eD, eN, targetSupply);
            if (pow(eN, eD, upperBound) > pow(eN, eD, lowerBound)) {
                integral = pow(eN, eD, upperBound) - pow(eN, eD, lowerBound);
            } else {
                integral = pow(eN, eD, lowerBound) - pow(eN, eD, upperBound);
            }

            return (multiplier * integral) / PRECISION / eD / eN;
        }
        return multiplier = targetPrice / targetSupply;
    }

    /**
     * @dev                Get the price in funding tokens to mint social tokens
     * @param amount       The amount of tokens to calculate price for
     */
    function priceToMint(uint256 amount) internal returns (uint256) {
        uint256 effectiveSupply;
        uint256 newSupply;

        if (initialSupply >= totalSupply()) {
            effectiveSupply = initialSupply - totalSupply();
        } else {
            effectiveSupply = totalSupply() - initialSupply;
        }
        newSupply = effectiveSupply + amount;
        return curveIntegral(newSupply, effectiveSupply);
    }

    /**
     * @dev                Get the reward in funding tokens to burn social tokens
     * @param amount       The amount of tokens to calculate reward for
     */
    function rewardForBurn(uint256 amount) internal returns (uint256) {
        uint256 effectiveSupply;
        uint256 lowSupply;

        if (initialSupply >= totalSupply()) {
            effectiveSupply = initialSupply - totalSupply();
        } else {
            effectiveSupply = totalSupply() - initialSupply;
        }
        lowSupply = effectiveSupply - amount;

        return curveIntegral(lowSupply, effectiveSupply);
    }

    /**
     * @dev                Mint new tokens with funding token
     * @param amount       The amount of tokens you want to mint
     */
    function buyToken(uint256 amount) external {
        uint256 priceForTokens = priceToMint(amount);
        uint256 tradeActivityFee =
            (priceForTokens * tradingSpread) / MAX_PERCENTAGE;

        tradeAccumulatedFee = tradeAccumulatedFee + tradeActivityFee;

        require(
            IERC20(fundingToken).transferFrom(
                msg.sender,
                address(this),
                priceForTokens + tradeActivityFee
            ),
            "Sender does not have enough funding token"
        );
        _mint(msg.sender, amount);

        emit Minted(amount, priceForTokens);
    }

    /**
     * @dev                Burn tokens to receive funding token
     * @param amount       The amount of tokens that you want to burn
     */
    function sellToken(uint256 amount) external {
        require(
            balanceOf(msg.sender) >= amount,
            "Sender does not have enough social token"
        );

        uint256 fundingTokensToReturn = rewardForBurn(amount);
        uint256 tradeActivityFee =
            (fundingTokensToReturn * tradingSpread) / MAX_PERCENTAGE;
        uint256 fundingTokensToReturnWithoutFee =
            fundingTokensToReturn - tradeActivityFee;

        require(
            IERC20(fundingToken).balanceOf(address(this)) >=
                fundingTokensToReturnWithoutFee,
            "There is not enough social token to be returned to user"
        );

        tradeAccumulatedFee = tradeAccumulatedFee + tradeActivityFee;

        IERC20(fundingToken).transfer(
            msg.sender,
            fundingTokensToReturn - tradeActivityFee
        );

        _burn(msg.sender, amount);

        emit Burned(amount, fundingTokensToReturn);
    }

    /**
     * @dev                Withdraw some amount of the InitialSupply minted of Social Token to owner address
     * @param amount       The amount of social tokens that you want to withdraw from initialSupply
     */
    function withdrawSocialToken(uint256 amount)
        external
        onlyOwner
        nonReentrant
    {
        require(
            initialSupply >= amount,
            "Remaining Initial supply is not enough"
        );

        initialSupply = initialSupply - amount;
        _transfer(address(this), owner(), amount);

        emit WithdrawSocialToken(msg.sender, amount);
    }

    /**
     * @dev                Airdrop a specific amount of social token to a given address
     * @param amount       The amount of social tokens that you want to withdraw from initialSupply
     */
    function airDropSocialToken(uint256 amount, address to)
        external
        onlyOwner
        nonReentrant
    {
        require(
            initialSupply >= amount,
            "Remaining Initial supply is not enough"
        );

        initialSupply = initialSupply - amount;
        _transfer(address(this), to, amount);

        emit AirdropSocialToken(to, amount);
    }

    /**
     * @dev                Withdraw some amount of the Funding Token
     * @param amount       The amount of funding tokens that you want to withdraw
     */
    function withdrawFundingToken(uint256 amount)
        external
        onlyOwner
        nonReentrant
    {
        require(
            tradeAccumulatedFee >= amount,
            "Withdrawal amount of accumulated funding token is not sufficient"
        );

        tradeAccumulatedFee = tradeAccumulatedFee - amount;
        IERC20(fundingToken).transfer(msg.sender, amount);

        emit WithdrawFundingToken(msg.sender, amount);
    }

    /**
     * @dev                Airdrop a specific amount of funding token to a given address
     * @param amount       The amount of funding token that you want to withdraw from tradeAccumulatedFee
     */
    function rewardEarnings(uint256 amount, address to)
        external
        onlyOwner
        nonReentrant
    {
        require(
            tradeAccumulatedFee >= amount,
            "Withdrawal amount of accumulated funding token is not sufficient"
        );

        tradeAccumulatedFee = tradeAccumulatedFee - amount;
        IERC20(fundingToken).transfer(to, amount);

        emit RewardEarnings(to, amount);
    }
}