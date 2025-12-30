// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

/**
 * @title IComplianceOracle
 * @notice Interface for external compliance checks during transfers
 */
interface IComplianceOracle {
    /**
     * @notice Check if a transfer is compliant
     * @param from Sender address
     * @param to Recipient address
     * @param amount Transfer amount
     * @return True if transfer is allowed, false otherwise
     */
    function isTransferAllowed(address from, address to, uint256 amount) external view returns (bool);
}

/**
 * @title MeridianMultiCurrencyStablecoin
 * @author Meridian Team
 * @notice ERC-20 stablecoin with multi-currency basket support
 * 
 * @dev Features:
 * - Multi-currency basket backing (single currency, IMF SDR, or custom)
 * - UUPS upgradeable proxy pattern
 * - Role-based access control
 * - Compliance (blacklist/whitelist)
 * - Reserve attestation tracking
 * - Emergency pause functionality
 * - GENIUS Act / MiCA compliant
 */
contract MeridianStablecoin is 
    Initializable,
    ERC20Upgradeable,
    AccessControlUpgradeable,
    PausableUpgradeable,
    UUPSUpgradeable 
{
    // ============ Roles ============

    /// @notice Role for minting new tokens
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    
    /// @notice Role for burning tokens
    bytes32 public constant BURNER_ROLE = keccak256("BURNER_ROLE");
    
    /// @notice Role for pausing contract
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    
    /// @notice Role for upgrading contract
    bytes32 public constant UPGRADER_ROLE = keccak256("UPGRADER_ROLE");

    // ============ Constants ============

    /// @notice Token decimals (6, like USDC)
    uint8 public constant TOKEN_DECIMALS = 6;

    /// @notice Reserve value decimals (2, e.g., 100 = $1.00)
    uint8 public constant RESERVE_DECIMALS = 2;

    /// @notice Multiplier to convert reserve value (2 decimals) to token decimals (6)
    uint256 public constant RESERVE_TO_TOKEN_MULTIPLIER = 10 ** (TOKEN_DECIMALS - RESERVE_DECIMALS);

    // ============ State Variables ============

    /// @notice Type of currency basket
    enum BasketType {
        SingleCurrency,  // Single currency stablecoin (e.g., EUR, GBP)
        ImfSdr,          // IMF Special Drawing Rights basket
        CustomBasket     // Custom multi-currency basket
    }

    /// @notice Basket configuration
    struct BasketConfig {
        string basketId;      // Unique identifier for the basket
        BasketType basketType; // Type of basket
        bool isActive;        // Whether the basket is currently active
    }

    /// @notice Current basket configuration
    BasketConfig public basketConfig;

    /// @notice Total reserve value in USD (with 2 decimals, e.g., 100 = $1.00)
    uint256 public totalReserveValue;

    /// @notice Mapping of currency code to reserve amount
    mapping(string => uint256) public currencyReserves;

    /// @notice Address of the compliance oracle
    address public complianceOracle;

    /// @notice Mapping of blacklisted addresses
    mapping(address => bool) public isBlacklisted;

    /// @notice Timestamp of last reserve attestation
    uint256 public lastAttestation;

    /// @notice Nonces for replay protection in mint operations
    mapping(address => uint256) public nonces;

    /// @notice Minimum reserve ratio (basis points, e.g., 10000 = 100%)
    uint256 public minReserveRatio;

    // ============ Events ============

    /**
     * @notice Emitted when tokens are minted
     * @param recipient Address receiving the tokens
     * @param amount Amount of tokens minted
     * @param basketId Basket identifier
     * @param reserveValue USD value of reserves backing the mint
     */
    event TokensMinted(
        address indexed recipient,
        uint256 amount,
        string basketId,
        uint256 reserveValue
    );

    /**
     * @notice Emitted when tokens are burned
     * @param account Address burning tokens
     * @param amount Amount of tokens burned
     * @param basketId Basket identifier
     * @param reserveValue USD value of reserves released
     */
    event TokensBurned(
        address indexed account,
        uint256 amount,
        string basketId,
        uint256 reserveValue
    );

    /**
     * @notice Emitted when reserves are attested
     * @param attestationId Unique attestation identifier
     * @param totalReserve Total reserve value attested
     * @param totalSupply Total token supply at time of attestation
     * @param timestamp Time of attestation
     */
    event ReserveAttested(
        uint256 indexed attestationId,
        uint256 totalReserve,
        uint256 totalSupply,
        uint256 timestamp
    );

    /**
     * @notice Emitted when basket is rebalanced
     * @param basketId Basket identifier
     * @param timestamp Time of rebalancing
     */
    event BasketRebalanced(
        string indexed basketId,
        uint256 timestamp
    );

    /**
     * @notice Emitted when an address is blacklisted
     * @param account Address that was blacklisted
     * @param reason Reason for blacklisting
     */
    event AddressBlacklisted(address indexed account, string reason);

    /**
     * @notice Emitted when an address is whitelisted
     * @param account Address that was whitelisted
     */
    event AddressWhitelisted(address indexed account);

    /**
     * @notice Emitted when reserve ratio is updated
     * @param oldRatio Previous reserve ratio
     * @param newRatio New reserve ratio
     */
    event ReserveRatioUpdated(uint256 oldRatio, uint256 newRatio);

    /**
     * @notice Emitted when compliance oracle is updated
     * @param oldOracle Previous oracle address
     * @param newOracle New oracle address
     */
    event ComplianceOracleUpdated(address indexed oldOracle, address indexed newOracle);

    // ============ Errors ============

    error InsufficientReserveBacking();
    error InvalidNonce();
    error RequestExpired();
    error RecipientBlacklisted();
    error SenderBlacklisted();
    error InvalidReserveRatio();
    error AttestationBelowSupply();
    error TransferNotCompliant();
    error InvalidAdminAddress();
    error InsufficientBurnBalance();
    error NoSupplyToBurn();

    // ============ Initialization ============

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    /**
     * @notice Initialize the stablecoin contract
     * @param name_ Token name
     * @param symbol_ Token symbol
     * @param basketId_ Basket identifier
     * @param basketType_ Type of basket
     * @param admin_ Admin address
     * @param complianceOracle_ Compliance oracle address
     */
    function initialize(
        string memory name_,
        string memory symbol_,
        string memory basketId_,
        BasketType basketType_,
        address admin_,
        address complianceOracle_
    ) public initializer {
        // HIGH-003: Validate admin address to prevent permanently locked contract
        if (admin_ == address(0)) revert InvalidAdminAddress();

        __ERC20_init(name_, symbol_);
        __AccessControl_init();
        __Pausable_init();
        __UUPSUpgradeable_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin_);
        _grantRole(MINTER_ROLE, admin_);
        _grantRole(BURNER_ROLE, admin_);
        _grantRole(PAUSER_ROLE, admin_);
        _grantRole(UPGRADER_ROLE, admin_);

        basketConfig = BasketConfig({
            basketId: basketId_,
            basketType: basketType_,
            isActive: true
        });

        complianceOracle = complianceOracle_;
        lastAttestation = block.timestamp;
        minReserveRatio = 10000; // 100% reserve ratio
    }

    // ============ Minting ============

    /**
     * @notice Mint request structure
     * @dev Used to ensure atomic minting with reserve verification
     */
    struct MintRequest {
        address recipient;      // Address to receive tokens
        uint256 amount;         // Amount of tokens to mint
        uint256 reserveValue;   // USD value of backing reserves (2 decimals)
        uint256 deadline;       // Timestamp after which request expires
        uint256 nonce;          // Nonce for replay protection
    }

    /**
     * @notice Mint new tokens with reserve backing verification
     * @param request Mint request containing recipient, amount, and reserve details
     * @dev Requires MINTER_ROLE and verifies 1:1 reserve backing
     */
    function mint(MintRequest calldata request) 
        external 
        onlyRole(MINTER_ROLE) 
        whenNotPaused 
    {
        // Validate request
        if (block.timestamp > request.deadline) revert RequestExpired();
        if (request.nonce != nonces[request.recipient]++) revert InvalidNonce();
        if (isBlacklisted[request.recipient]) revert RecipientBlacklisted();

        // Verify reserve backing (must be at least 1:1)
        // Normalize: reserveValue (2 decimals) * 10^4 -> token decimals (6 decimals)
        uint256 normalizedReserveValue = request.reserveValue * RESERVE_TO_TOKEN_MULTIPLIER;
        if (normalizedReserveValue < request.amount) revert InsufficientReserveBacking();

        // Update reserve tracking
        totalReserveValue += request.reserveValue;

        // Mint tokens
        _mint(request.recipient, request.amount);

        emit TokensMinted(
            request.recipient,
            request.amount,
            basketConfig.basketId,
            request.reserveValue
        );
    }

    // ============ Burning ============

    /**
     * @notice Burn tokens and release pro-rata reserves
     * @param amount Amount of tokens to burn
     * @dev Anyone can burn their own tokens
     * @dev Follows CEI pattern: Checks -> Effects (burn) -> Effects (reserve update)
     */
    function burn(uint256 amount) external whenNotPaused {
        // CHECKS
        if (isBlacklisted[msg.sender]) revert SenderBlacklisted();

        uint256 currentSupply = totalSupply();
        if (balanceOf(msg.sender) < amount) revert InsufficientBurnBalance();
        if (currentSupply == 0) revert NoSupplyToBurn();

        // Calculate reserve to release BEFORE any state changes (pro-rata)
        // Note: This calculation must happen before _burn() changes the supply
        uint256 reserveToRelease = (amount * totalReserveValue) / currentSupply;

        // EFFECTS (Part 1): Burn tokens first
        // This calls _update() which may make external call to compliance oracle
        _burn(msg.sender, amount);

        // EFFECTS (Part 2): Update reserve tracking AFTER burn succeeds
        // If _burn reverts (e.g., compliance check fails), this line is never reached
        totalReserveValue -= reserveToRelease;

        // INTERACTIONS: Emit event
        emit TokensBurned(
            msg.sender,
            amount,
            basketConfig.basketId,
            reserveToRelease
        );
    }

    // ============ Compliance ============

    /**
     * @notice Blacklist an address for compliance reasons
     * @param account Address to blacklist
     * @param reason Reason for blacklisting
     * @dev NOT protected by pause - compliance actions must work during emergencies
     */
    function blacklistAddress(address account, string memory reason)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        isBlacklisted[account] = true;
        emit AddressBlacklisted(account, reason);
    }

    /**
     * @notice Remove an address from the blacklist
     * @param account Address to whitelist
     * @dev NOT protected by pause - compliance actions must work during emergencies
     */
    function whitelistAddress(address account)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        isBlacklisted[account] = false;
        emit AddressWhitelisted(account);
    }

    /**
     * @notice Attest to the current reserve backing
     * @param attestedReserveValue Total USD value of reserves (2 decimals)
     * @dev Reserves must be at least equal to total supply
     * @dev Protected by pause mechanism for emergency scenarios
     */
    function attestReserves(uint256 attestedReserveValue)
        external
        whenNotPaused
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        uint256 currentSupply = totalSupply();
        uint256 requiredReserve = (currentSupply * minReserveRatio) / 10000;

        if (attestedReserveValue < requiredReserve) revert AttestationBelowSupply();

        lastAttestation = block.timestamp;

        emit ReserveAttested(
            block.timestamp,
            attestedReserveValue,
            currentSupply,
            block.timestamp
        );
    }

    /**
     * @notice Update the minimum reserve ratio
     * @param newRatio New reserve ratio in basis points (10000 = 100%)
     * @dev Protected by pause mechanism for emergency scenarios
     */
    function setMinReserveRatio(uint256 newRatio)
        external
        whenNotPaused
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        if (newRatio < 10000) revert InvalidReserveRatio();

        uint256 oldRatio = minReserveRatio;
        minReserveRatio = newRatio;

        emit ReserveRatioUpdated(oldRatio, newRatio);
    }

    // ============ Emergency Functions ============

    /**
     * @notice Pause all token operations
     */
    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }

    /**
     * @notice Unpause token operations
     */
    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }

    // ============ Upgradeability ============

    /**
     * @notice Authorize contract upgrade
     * @param newImplementation Address of new implementation
     */
    function _authorizeUpgrade(address newImplementation) 
        internal 
        override 
        onlyRole(UPGRADER_ROLE) 
    {}

    // ============ Transfer Overrides ============

    /**
     * @notice Hook that is called on any transfer of tokens (including mint and burn)
     * @dev Enforces pause, blacklist, and compliance oracle checks
     */
    function _update(
        address from,
        address to,
        uint256 amount
    ) internal virtual override whenNotPaused {
        if (isBlacklisted[from]) revert SenderBlacklisted();
        if (isBlacklisted[to]) revert RecipientBlacklisted();

        // Call compliance oracle if configured (skip for mint/burn where from/to is zero)
        // HIGH-002: Use gas limit and try/catch to prevent DoS from malicious oracle
        if (complianceOracle != address(0) && from != address(0) && to != address(0)) {
            try IComplianceOracle(complianceOracle).isTransferAllowed{gas: 100000}(from, to, amount) returns (bool allowed) {
                if (!allowed) {
                    revert TransferNotCompliant();
                }
            } catch {
                // Oracle call failed - fail closed for security (reject transfer)
                revert TransferNotCompliant();
            }
        }

        super._update(from, to, amount);
    }

    /**
     * @notice Update the compliance oracle address
     * @param newOracle Address of the new compliance oracle (or address(0) to disable)
     */
    function setComplianceOracle(address newOracle) external onlyRole(DEFAULT_ADMIN_ROLE) {
        address oldOracle = complianceOracle;
        complianceOracle = newOracle;
        // HIGH-001: Emit event for critical parameter update
        emit ComplianceOracleUpdated(oldOracle, newOracle);
    }

    // ============ View Functions ============

    /**
     * @notice Override decimals to return 6 (like USDC)
     * @return 6 decimals
     */
    function decimals() public pure override returns (uint8) {
        return TOKEN_DECIMALS;
    }

    /**
     * @notice Get the current reserve ratio in basis points
     * @dev Normalizes reserve value (2 decimals) to token supply (6 decimals) for comparison
     * @return Reserve ratio (e.g., 10000 = 100%)
     */
    function getReserveRatio() external view returns (uint256) {
        uint256 supply = totalSupply();
        if (supply == 0) return 0;
        // Normalize: reserveValue * 10^4 converts from 2 decimals to 6 decimals
        return (totalReserveValue * RESERVE_TO_TOKEN_MULTIPLIER * 10000) / supply;
    }

    /**
     * @notice Get days since last reserve attestation
     * @return Number of days
     */
    function daysSinceLastAttestation() external view returns (uint256) {
        return (block.timestamp - lastAttestation) / 1 days;
    }

    /**
     * @notice Get the basket configuration
     * @return basketId Basket identifier
     * @return basketType Type of basket
     * @return isActive Whether basket is active
     */
    function getBasketConfig() external view returns (
        string memory basketId,
        BasketType basketType,
        bool isActive
    ) {
        return (
            basketConfig.basketId,
            basketConfig.basketType,
            basketConfig.isActive
        );
    }

    // ============ Storage Gap ============

    /**
     * @dev Reserved storage space for future upgrades
     * @dev MED-007: Prevents storage collision when adding new state variables
     * See: https://docs.openzeppelin.com/contracts/4.x/upgradeable#storage_gaps
     */
    uint256[50] private __gap;
}

