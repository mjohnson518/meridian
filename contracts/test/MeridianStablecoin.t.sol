// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/MeridianStablecoin.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

/**
 * @title MockComplianceOracle
 * @notice Mock compliance oracle for testing (CONTRACT-CRIT-001 test fix)
 */
contract MockComplianceOracle {
    bool public allowAll = true;

    function setAllowAll(bool _allow) external {
        allowAll = _allow;
    }

    function isTransferAllowed(address, address, uint256) external view returns (bool) {
        return allowAll;
    }
}

/**
 * @title MeridianStablecoinTest
 * @notice Comprehensive test suite for MeridianStablecoin
 */
contract MeridianStablecoinTest is Test {
    MeridianStablecoin public implementation;
    MeridianStablecoin public stablecoin;
    ERC1967Proxy public proxy;

    address public admin = address(0x1);
    address public minter = address(0x2);
    address public burner = address(0x3);
    address public user = address(0x4);
    MockComplianceOracle public mockOracle;

    // Decimal scaling helpers
    uint256 constant TOKEN_UNIT = 10 ** 6;    // 6 decimals for tokens
    uint256 constant RESERVE_UNIT = 10 ** 2;  // 2 decimals for reserves

    function setUp() public {
        // Deploy mock oracle (CONTRACT-CRIT-001: Must be a contract, not EOA)
        mockOracle = new MockComplianceOracle();

        // Deploy implementation
        implementation = new MeridianStablecoin();

        // Prepare initialization data
        bytes memory initData = abi.encodeWithSelector(
            MeridianStablecoin.initialize.selector,
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)  // CONTRACT-CRIT-001: Use deployed contract address
        );

        // Deploy proxy
        proxy = new ERC1967Proxy(address(implementation), initData);
        stablecoin = MeridianStablecoin(address(proxy));

        // Grant roles
        vm.startPrank(admin);
        stablecoin.grantRole(stablecoin.MINTER_ROLE(), minter);
        stablecoin.grantRole(stablecoin.BURNER_ROLE(), burner);
        vm.stopPrank();
    }

    // ============ Initialization Tests ============

    function test_Initialization() public {
        assertEq(stablecoin.name(), "EUR Meridian");
        assertEq(stablecoin.symbol(), "EURM");
        assertEq(stablecoin.totalSupply(), 0);
        
        (string memory basketId, MeridianStablecoin.BasketType basketType, bool isActive) = 
            stablecoin.getBasketConfig();
        
        assertEq(basketId, "EUR_BASKET");
        assertEq(uint256(basketType), uint256(MeridianStablecoin.BasketType.SingleCurrency));
        assertTrue(isActive);
    }

    function test_AdminHasRoles() public {
        assertTrue(stablecoin.hasRole(stablecoin.DEFAULT_ADMIN_ROLE(), admin));
        assertTrue(stablecoin.hasRole(stablecoin.PAUSER_ROLE(), admin));
        assertTrue(stablecoin.hasRole(stablecoin.UPGRADER_ROLE(), admin));
    }

    function test_CannotReinitialize() public {
        vm.expectRevert();
        stablecoin.initialize(
            "Test",
            "TST",
            "TEST",
            MeridianStablecoin.BasketType.CustomBasket,
            admin,
            address(mockOracle)  // CONTRACT-CRIT-001: Use deployed contract
        );
    }

    // ============ Minting Tests ============

    function test_MintWithSufficientReserve() public {
        uint256 mintAmount = 1000 ether;
        uint256 reserveValue = 1000 ether;

        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: mintAmount,
            reserveValue: reserveValue,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(request);

        assertEq(stablecoin.balanceOf(user), mintAmount);
        assertEq(stablecoin.totalSupply(), mintAmount);
        assertEq(stablecoin.totalReserveValue(), reserveValue);
    }

    function test_MintIncreasesNonce() public {
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 100 ether,
            reserveValue: 100 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(request);

        assertEq(stablecoin.nonces(user), 1);
    }

    function test_RevertMintInsufficientReserve() public {
        // Use proper decimal formats: TOKEN_UNIT (6 decimals), RESERVE_UNIT (2 decimals)
        // 1000 tokens = 1000 * 10^6, 999 USD reserves = 999 * 10^2 = 99900
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 * TOKEN_UNIT,
            reserveValue: 999 * RESERVE_UNIT, // Less than amount when normalized
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        vm.expectRevert(MeridianStablecoin.InsufficientReserveBacking.selector);
        stablecoin.mint(request);
    }

    function test_RevertMintExpiredRequest() public {
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp - 1, // Expired
            nonce: 0
        });

        vm.prank(minter);
        vm.expectRevert(MeridianStablecoin.RequestExpired.selector);
        stablecoin.mint(request);
    }

    function test_RevertMintInvalidNonce() public {
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 5 // Wrong nonce
        });

        vm.prank(minter);
        vm.expectRevert(MeridianStablecoin.InvalidNonce.selector);
        stablecoin.mint(request);
    }

    function test_RevertMintWithoutRole() public {
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(user);
        vm.expectRevert();
        stablecoin.mint(request);
    }

    // ============ Burning Tests ============

    function test_BurnTokens() public {
        // First mint some tokens
        MeridianStablecoin.MintRequest memory mintReq = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(mintReq);

        // Burn half
        uint256 burnAmount = 500 ether;
        vm.prank(user);
        stablecoin.burn(burnAmount);

        assertEq(stablecoin.balanceOf(user), 500 ether);
        assertEq(stablecoin.totalSupply(), 500 ether);
        assertEq(stablecoin.totalReserveValue(), 500 ether);
    }

    function test_BurnProRataReserveCalculation() public {
        // Mint to user1
        MeridianStablecoin.MintRequest memory mintReq1 = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1200 ether, // 120% backed
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(mintReq1);

        // Burn 250 tokens (25% of supply)
        // Should release 25% of reserves = 300 ether
        vm.prank(user);
        stablecoin.burn(250 ether);

        // Remaining reserves should be 900 ether (1200 - 300)
        assertEq(stablecoin.totalReserveValue(), 900 ether);
        assertEq(stablecoin.totalSupply(), 750 ether);
    }

    function test_RevertBurnInsufficientBalance() public {
        vm.prank(user);
        vm.expectRevert();
        stablecoin.burn(100 ether);
    }

    // ============ Blacklist Tests ============

    function test_BlacklistAddress() public {
        vm.prank(admin);
        stablecoin.blacklistAddress(user, "Compliance violation");

        assertTrue(stablecoin.isBlacklisted(user));
    }

    function test_WhitelistAddress() public {
        // First blacklist
        vm.prank(admin);
        stablecoin.blacklistAddress(user, "Test");

        // Then whitelist
        vm.prank(admin);
        stablecoin.whitelistAddress(user);

        assertFalse(stablecoin.isBlacklisted(user));
    }

    function test_RevertMintToBlacklistedAddress() public {
        // Blacklist user
        vm.prank(admin);
        stablecoin.blacklistAddress(user, "Test");

        // Try to mint
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        vm.expectRevert(MeridianStablecoin.RecipientBlacklisted.selector);
        stablecoin.mint(request);
    }

    function test_RevertTransferFromBlacklistedAddress() public {
        // Mint to user first
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(request);

        // Blacklist user
        vm.prank(admin);
        stablecoin.blacklistAddress(user, "Test");

        // Try to transfer
        vm.prank(user);
        vm.expectRevert(MeridianStablecoin.SenderBlacklisted.selector);
        stablecoin.transfer(address(0x10), 100 ether);
    }

    // ============ Pause Tests ============

    function test_PauseContract() public {
        vm.prank(admin);
        stablecoin.pause();

        assertTrue(stablecoin.paused());
    }

    function test_UnpauseContract() public {
        vm.prank(admin);
        stablecoin.pause();

        vm.prank(admin);
        stablecoin.unpause();

        assertFalse(stablecoin.paused());
    }

    function test_RevertMintWhenPaused() public {
        vm.prank(admin);
        stablecoin.pause();

        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        vm.expectRevert();
        stablecoin.mint(request);
    }

    function test_RevertTransferWhenPaused() public {
        // Mint first
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(request);

        // Pause
        vm.prank(admin);
        stablecoin.pause();

        // Try transfer
        vm.prank(user);
        vm.expectRevert();
        stablecoin.transfer(address(0x10), 100 ether);
    }

    // ============ Reserve Attestation Tests ============

    function test_AttestReserves() public {
        // Mint first
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(request);

        // Attest
        vm.prank(admin);
        stablecoin.attestReserves(1000 ether);

        assertEq(stablecoin.lastAttestation(), block.timestamp);
    }

    function test_RevertAttestBelowSupply() public {
        // Mint first
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 ether,
            reserveValue: 1000 ether,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(request);

        // Try to attest with insufficient reserves
        vm.prank(admin);
        vm.expectRevert(MeridianStablecoin.AttestationBelowSupply.selector);
        stablecoin.attestReserves(999 ether);
    }

    function test_DaysSinceLastAttestation() public {
        uint256 initialDays = stablecoin.daysSinceLastAttestation();

        // Fast forward 5 days
        vm.warp(block.timestamp + 5 days);

        uint256 daysPassed = stablecoin.daysSinceLastAttestation();
        assertEq(daysPassed, initialDays + 5);
    }

    // ============ Reserve Ratio Tests ============

    function test_GetReserveRatio() public {
        // Mint with exact 1:1 backing using proper decimal formats
        // 1000 tokens (6 decimals) backed by 1000 USD (2 decimals)
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 * TOKEN_UNIT,
            reserveValue: 1000 * RESERVE_UNIT,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(request);

        // Should be 100% = 10000 basis points
        assertEq(stablecoin.getReserveRatio(), 10000);
    }

    function test_GetReserveRatioOverCollateralized() public {
        // Mint with 120% backing using proper decimal formats
        // 1000 tokens (6 decimals) backed by 1200 USD (2 decimals)
        MeridianStablecoin.MintRequest memory request = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: 1000 * TOKEN_UNIT,
            reserveValue: 1200 * RESERVE_UNIT,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(request);

        // Should be 120% = 12000 basis points
        assertEq(stablecoin.getReserveRatio(), 12000);
    }

    function test_SetMinReserveRatio() public {
        vm.prank(admin);
        stablecoin.setMinReserveRatio(11000); // 110%

        assertEq(stablecoin.minReserveRatio(), 11000);
    }

    function test_RevertSetMinReserveRatioBelowMinimum() public {
        vm.prank(admin);
        vm.expectRevert(MeridianStablecoin.InvalidReserveRatio.selector);
        stablecoin.setMinReserveRatio(9000); // Below 100%
    }

    // ============ View Function Tests ============

    function test_GetBasketConfig() public {
        (string memory basketId, MeridianStablecoin.BasketType basketType, bool isActive) = 
            stablecoin.getBasketConfig();

        assertEq(basketId, "EUR_BASKET");
        assertEq(uint256(basketType), uint256(MeridianStablecoin.BasketType.SingleCurrency));
        assertTrue(isActive);
    }

    // ============ Access Control Tests ============

    function test_RevertPauseWithoutRole() public {
        vm.prank(user);
        vm.expectRevert();
        stablecoin.pause();
    }

    function test_RevertBlacklistWithoutRole() public {
        vm.prank(user);
        vm.expectRevert();
        stablecoin.blacklistAddress(address(0x10), "Test");
    }

    function test_RevertAttestWithoutRole() public {
        vm.prank(user);
        vm.expectRevert();
        stablecoin.attestReserves(1000 ether);
    }
}

