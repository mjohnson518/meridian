// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/MeridianStablecoin.sol";
import "../src/MeridianFactory.sol";

/**
 * @title MockComplianceOracleFactory
 * @notice Mock compliance oracle for factory testing (CONTRACT-CRIT-001 test fix)
 */
contract MockComplianceOracleFactory {
    function isTransferAllowed(address, address, uint256) external pure returns (bool) {
        return true;
    }
}

/**
 * @title MeridianFactoryTest
 * @notice Test suite for MeridianFactory
 */
contract MeridianFactoryTest is Test {
    MeridianStablecoin public implementation;
    MeridianFactory public factory;
    MockComplianceOracleFactory public mockOracle;

    address public owner = address(0x1);
    address public admin = address(0x2);

    function setUp() public {
        // Deploy mock oracle (CONTRACT-CRIT-001: Must be a contract, not EOA)
        mockOracle = new MockComplianceOracleFactory();

        // Deploy implementation
        implementation = new MeridianStablecoin();

        // Deploy factory
        vm.prank(owner);
        factory = new MeridianFactory(address(implementation));
    }

    // ============ Initialization Tests ============

    function test_FactoryInitialization() public {
        assertEq(factory.implementation(), address(implementation));
        assertEq(factory.owner(), owner);
        assertEq(factory.getDeployedStablecoinsCount(), 0);
    }

    function test_RevertDeployWithZeroImplementation() public {
        vm.prank(owner);
        vm.expectRevert(MeridianFactory.InvalidImplementation.selector);
        new MeridianFactory(address(0));
    }

    // ============ Deployment Tests ============

    function test_DeployStablecoin() public {
        vm.prank(owner);
        address stablecoin = factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)  // CONTRACT-CRIT-001: Use deployed contract
        );

        assertTrue(stablecoin != address(0));
        assertEq(factory.getDeployedStablecoinsCount(), 1);
    }

    function test_DeployedStablecoinIsInitialized() public {
        vm.prank(owner);
        address stablecoinAddr = factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        MeridianStablecoin stablecoin = MeridianStablecoin(stablecoinAddr);
        
        assertEq(stablecoin.name(), "EUR Meridian");
        assertEq(stablecoin.symbol(), "EURM");
        
        (string memory basketId, MeridianStablecoin.BasketType basketType, bool isActive) = 
            stablecoin.getBasketConfig();
        
        assertEq(basketId, "EUR_BASKET");
        assertEq(uint256(basketType), uint256(MeridianStablecoin.BasketType.SingleCurrency));
        assertTrue(isActive);
    }

    function test_DeployMultipleStablecoins() public {
        vm.startPrank(owner);
        
        address eurStablecoin = factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        address gbpStablecoin = factory.deployStablecoin(
            "GBP Meridian",
            "GBPM",
            "GBP_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        vm.stopPrank();

        assertEq(factory.getDeployedStablecoinsCount(), 2);
        assertTrue(eurStablecoin != gbpStablecoin);
    }

    function test_RevertDeployDuplicateBasketId() public {
        vm.startPrank(owner);
        
        factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        // Try to deploy with same basket ID
        vm.expectRevert(MeridianFactory.BasketIdAlreadyExists.selector);
        factory.deployStablecoin(
            "EUR Meridian 2",
            "EURM2",
            "EUR_BASKET", // Same basket ID
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        vm.stopPrank();
    }

    function test_RevertDeployWithoutOwner() public {
        vm.prank(address(0x10)); // Not owner
        vm.expectRevert();
        factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );
    }

    // ============ Registry Tests ============

    function test_GetAllStablecoins() public {
        vm.startPrank(owner);
        
        address eurStablecoin = factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        address gbpStablecoin = factory.deployStablecoin(
            "GBP Meridian",
            "GBPM",
            "GBP_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        vm.stopPrank();

        address[] memory allStablecoins = factory.getAllStablecoins();
        assertEq(allStablecoins.length, 2);
        assertEq(allStablecoins[0], eurStablecoin);
        assertEq(allStablecoins[1], gbpStablecoin);
    }

    function test_GetStablecoinInfo() public {
        vm.prank(owner);
        address stablecoinAddr = factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        MeridianFactory.StablecoinInfo memory info = factory.getStablecoinInfo(stablecoinAddr);
        
        assertEq(info.name, "EUR Meridian");
        assertEq(info.symbol, "EURM");
        assertEq(info.basketId, "EUR_BASKET");
        assertEq(uint256(info.basketType), uint256(MeridianStablecoin.BasketType.SingleCurrency));
        assertEq(info.admin, admin);
        assertTrue(info.exists);
    }

    function test_GetStablecoinByBasketId() public {
        vm.prank(owner);
        address stablecoinAddr = factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        address foundStablecoin = factory.getStablecoinByBasketId("EUR_BASKET");
        assertEq(foundStablecoin, stablecoinAddr);
    }

    function test_BasketIdExists() public {
        assertFalse(factory.basketIdExists("EUR_BASKET"));

        vm.prank(owner);
        factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        assertTrue(factory.basketIdExists("EUR_BASKET"));
    }

    function test_RevertGetStablecoinInfoNonExistent() public {
        vm.expectRevert(MeridianFactory.StablecoinNotFound.selector);
        factory.getStablecoinInfo(address(0x999));
    }

    function test_RevertGetStablecoinByNonExistentBasketId() public {
        vm.expectRevert(MeridianFactory.StablecoinNotFound.selector);
        factory.getStablecoinByBasketId("NONEXISTENT");
    }

    // ============ Implementation Update Tests ============

    function test_UpdateImplementation() public {
        MeridianStablecoin newImplementation = new MeridianStablecoin();

        vm.prank(owner);
        factory.updateImplementation(address(newImplementation));

        assertEq(factory.implementation(), address(newImplementation));
    }

    function test_RevertUpdateImplementationZeroAddress() public {
        vm.prank(owner);
        vm.expectRevert(MeridianFactory.InvalidImplementation.selector);
        factory.updateImplementation(address(0));
    }

    function test_RevertUpdateImplementationWithoutOwner() public {
        MeridianStablecoin newImplementation = new MeridianStablecoin();

        vm.prank(address(0x10)); // Not owner
        vm.expectRevert();
        factory.updateImplementation(address(newImplementation));
    }

    // ============ Integration Tests ============

    function test_DeployAndMintTokens() public {
        // Deploy stablecoin
        vm.prank(owner);
        address stablecoinAddr = factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        MeridianStablecoin stablecoin = MeridianStablecoin(stablecoinAddr);

        // Grant minter role (use startPrank for proxy calls)
        address minter = address(0x10);
        vm.startPrank(admin);
        stablecoin.grantRole(stablecoin.MINTER_ROLE(), minter);
        vm.stopPrank();

        // Mint tokens
        address recipient = address(0x20);
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: recipient,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.startPrank(minter);
        stablecoin.mint(request);
        vm.stopPrank();

        assertEq(stablecoin.balanceOf(recipient), 1000 ether);
    }

    function test_DeployMultipleBasketTypes() public {
        vm.startPrank(owner);
        
        // Single currency
        address eurStablecoin = factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        // IMF SDR
        address sdrStablecoin = factory.deployStablecoin(
            "SDR Meridian",
            "SDRM",
            "SDR_BASKET",
            MeridianStablecoin.BasketType.ImfSdr,
            admin,
            address(mockOracle)
        );

        // Custom basket
        address customStablecoin = factory.deployStablecoin(
            "Custom Meridian",
            "CSTM",
            "CUSTOM_BASKET",
            MeridianStablecoin.BasketType.CustomBasket,
            admin,
            address(mockOracle)
        );

        vm.stopPrank();

        MeridianStablecoin eurSC = MeridianStablecoin(eurStablecoin);
        MeridianStablecoin sdrSC = MeridianStablecoin(sdrStablecoin);
        MeridianStablecoin customSC = MeridianStablecoin(customStablecoin);

        (, MeridianStablecoin.BasketType eurType,) = eurSC.getBasketConfig();
        (, MeridianStablecoin.BasketType sdrType,) = sdrSC.getBasketConfig();
        (, MeridianStablecoin.BasketType customType,) = customSC.getBasketConfig();

        assertEq(uint256(eurType), uint256(MeridianStablecoin.BasketType.SingleCurrency));
        assertEq(uint256(sdrType), uint256(MeridianStablecoin.BasketType.ImfSdr));
        assertEq(uint256(customType), uint256(MeridianStablecoin.BasketType.CustomBasket));
    }
}

