// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./MeridianStablecoin.sol";

/**
 * @title MeridianFactory
 * @author Meridian Team
 * @notice Factory contract for deploying new Meridian stablecoins
 * 
 * @dev Deploys stablecoins using UUPS proxy pattern for upgradeability
 * Maintains a registry of all deployed stablecoins
 */
contract MeridianFactory is Ownable {
    // ============ State Variables ============

    /// @notice Address of the stablecoin implementation
    address public implementation;

    /// @notice Array of all deployed stablecoin proxies
    address[] public deployedStablecoins;

    /// @notice Mapping from stablecoin address to deployment info
    mapping(address => StablecoinInfo) public stablecoinInfo;

    /// @notice Mapping from basket ID to stablecoin address
    mapping(string => address) public basketIdToStablecoin;

    // ============ Structs ============

    /**
     * @notice Information about a deployed stablecoin
     */
    struct StablecoinInfo {
        string name;
        string symbol;
        string basketId;
        MeridianStablecoin.BasketType basketType;
        address admin;
        uint256 deployedAt;
        bool exists;
    }

    // ============ Events ============

    /**
     * @notice Emitted when a new stablecoin is deployed
     * @param stablecoin Address of the deployed stablecoin proxy
     * @param name Token name
     * @param symbol Token symbol
     * @param basketId Basket identifier
     * @param basketType Type of basket
     * @param admin Admin address
     */
    event StablecoinDeployed(
        address indexed stablecoin,
        string name,
        string symbol,
        string basketId,
        MeridianStablecoin.BasketType basketType,
        address indexed admin
    );

    /**
     * @notice Emitted when implementation is updated
     * @param oldImplementation Previous implementation address
     * @param newImplementation New implementation address
     */
    event ImplementationUpdated(
        address indexed oldImplementation,
        address indexed newImplementation
    );

    // ============ Errors ============

    error BasketIdAlreadyExists();
    error InvalidImplementation();
    error StablecoinNotFound();

    // ============ Constructor ============

    /**
     * @notice Deploy the factory with an initial implementation
     * @param implementation_ Address of the MeridianStablecoin implementation
     */
    constructor(address implementation_) {
        if (implementation_ == address(0)) revert InvalidImplementation();
        implementation = implementation_;
    }

    // ============ Deployment Functions ============

    /**
     * @notice Deploy a new stablecoin
     * @param name_ Token name
     * @param symbol_ Token symbol
     * @param basketId_ Basket identifier (must be unique)
     * @param basketType_ Type of basket
     * @param admin_ Admin address for the new stablecoin
     * @param complianceOracle_ Compliance oracle address
     * @return Address of the deployed stablecoin proxy
     */
    function deployStablecoin(
        string memory name_,
        string memory symbol_,
        string memory basketId_,
        MeridianStablecoin.BasketType basketType_,
        address admin_,
        address complianceOracle_
    ) external onlyOwner returns (address) {
        // Check basket ID is unique
        if (basketIdToStablecoin[basketId_] != address(0)) {
            revert BasketIdAlreadyExists();
        }

        // Encode initialization data
        bytes memory initData = abi.encodeWithSelector(
            MeridianStablecoin.initialize.selector,
            name_,
            symbol_,
            basketId_,
            basketType_,
            admin_,
            complianceOracle_
        );

        // Deploy proxy
        ERC1967Proxy proxy = new ERC1967Proxy(implementation, initData);
        address stablecoinAddress = address(proxy);

        // Store deployment info
        StablecoinInfo memory info = StablecoinInfo({
            name: name_,
            symbol: symbol_,
            basketId: basketId_,
            basketType: basketType_,
            admin: admin_,
            deployedAt: block.timestamp,
            exists: true
        });

        deployedStablecoins.push(stablecoinAddress);
        stablecoinInfo[stablecoinAddress] = info;
        basketIdToStablecoin[basketId_] = stablecoinAddress;

        emit StablecoinDeployed(
            stablecoinAddress,
            name_,
            symbol_,
            basketId_,
            basketType_,
            admin_
        );

        return stablecoinAddress;
    }

    // ============ Admin Functions ============

    /**
     * @notice Update the implementation contract
     * @param newImplementation Address of the new implementation
     * @dev Existing stablecoins must be upgraded individually
     */
    function updateImplementation(address newImplementation) 
        external 
        onlyOwner 
    {
        if (newImplementation == address(0)) revert InvalidImplementation();
        
        address oldImplementation = implementation;
        implementation = newImplementation;

        emit ImplementationUpdated(oldImplementation, newImplementation);
    }

    // ============ View Functions ============

    /**
     * @notice Get the total number of deployed stablecoins
     * @return Number of deployed stablecoins
     */
    function getDeployedStablecoinsCount() external view returns (uint256) {
        return deployedStablecoins.length;
    }

    /**
     * @notice Get all deployed stablecoin addresses
     * @return Array of stablecoin addresses
     */
    function getAllStablecoins() external view returns (address[] memory) {
        return deployedStablecoins;
    }

    /**
     * @notice Get stablecoin info by address
     * @param stablecoin Address of the stablecoin
     * @return info StablecoinInfo struct
     */
    function getStablecoinInfo(address stablecoin) 
        external 
        view 
        returns (StablecoinInfo memory info) 
    {
        if (!stablecoinInfo[stablecoin].exists) revert StablecoinNotFound();
        return stablecoinInfo[stablecoin];
    }

    /**
     * @notice Get stablecoin address by basket ID
     * @param basketId Basket identifier
     * @return Address of the stablecoin
     */
    function getStablecoinByBasketId(string memory basketId) 
        external 
        view 
        returns (address) 
    {
        address stablecoin = basketIdToStablecoin[basketId];
        if (stablecoin == address(0)) revert StablecoinNotFound();
        return stablecoin;
    }

    /**
     * @notice Check if a basket ID is already in use
     * @param basketId Basket identifier to check
     * @return True if the basket ID exists
     */
    function basketIdExists(string memory basketId) 
        external 
        view 
        returns (bool) 
    {
        return basketIdToStablecoin[basketId] != address(0);
    }
}

