// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

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

    // ============ Errors ============

    error InsufficientReserveBacking();
    error InvalidNonce();
    error RequestExpired();
    error RecipientBlacklisted();
    error SenderBlacklisted();
    error InvalidReserveRatio();
    error AttestationBelowSupply();

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
        __ERC20_init(name_, symbol_);
        __AccessControl_init();
        __Pausable_init();
        __UUPSUpgradeable_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin_);
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
        if (request.reserveValue < request.amount) revert InsufficientReserveBacking();

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
     */
    function burn(uint256 amount) external whenNotPaused {
        if (isBlacklisted[msg.sender]) revert SenderBlacklisted();
        
        uint256 currentSupply = totalSupply();
        require(balanceOf(msg.sender) >= amount, "Insufficient balance");
        require(currentSupply > 0, "No supply to burn");

        // Calculate reserve to release (pro-rata)
        uint256 reserveToRelease = (amount * totalReserveValue) / currentSupply;

        // Update reserve tracking
        totalReserveValue -= reserveToRelease;

        // Burn tokens
        _burn(msg.sender, amount);

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
     */
    function attestReserves(uint256 attestedReserveValue) 
        external 
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
     */
    function setMinReserveRatio(uint256 newRatio) 
        external 
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
     * @dev Enforces pause and blacklist checks
     */
    function _update(
        address from,
        address to,
        uint256 amount
    ) internal virtual override whenNotPaused {
        if (isBlacklisted[from]) revert SenderBlacklisted();
        if (isBlacklisted[to]) revert RecipientBlacklisted();
        super._update(from, to, amount);
    }

    // ============ View Functions ============

    /**
     * @notice Get the current reserve ratio in basis points
     * @return Reserve ratio (e.g., 10000 = 100%)
     */
    function getReserveRatio() external view returns (uint256) {
        uint256 supply = totalSupply();
        if (supply == 0) return 0;
        return (totalReserveValue * 10000) / supply;
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
}

