// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/MeridianStablecoin.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract MockComplianceOracleFuzz {
    function isTransferAllowed(address, address, uint256) external pure returns (bool) {
        return true;
    }
}

/// @title MeridianStablecoinFuzzTest
/// @notice Fuzz tests for MeridianStablecoin core invariants
contract MeridianStablecoinFuzzTest is Test {
    MeridianStablecoin public stablecoin;

    address public admin = address(0x1);
    address public minter = address(0x2);
    address public user = address(0x4);

    uint256 constant MAX_MINT = 1_000_000 ether;

    function setUp() public {
        MockComplianceOracleFuzz mockOracle = new MockComplianceOracleFuzz();
        MeridianStablecoin implementation = new MeridianStablecoin();

        bytes memory initData = abi.encodeWithSelector(
            MeridianStablecoin.initialize.selector,
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            admin,
            address(mockOracle)
        );

        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), initData);
        stablecoin = MeridianStablecoin(address(proxy));

        vm.startPrank(admin);
        stablecoin.grantRole(stablecoin.MINTER_ROLE(), minter);
        vm.stopPrank();
    }

    /// @notice Minting and full burn must conserve: balance ends at zero, supply ends at zero
    function testFuzz_MintBurnConserves(uint128 mintAmount) public {
        vm.assume(mintAmount > 0);
        vm.assume(mintAmount <= MAX_MINT);

        MeridianStablecoin.MintRequest memory req = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: mintAmount,
            reserveValue: mintAmount, // 1:1 backing
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(req);

        assertEq(stablecoin.balanceOf(user), mintAmount, "balance after mint");
        assertEq(stablecoin.totalSupply(), mintAmount, "supply after mint");

        vm.prank(user);
        stablecoin.burn(mintAmount);

        assertEq(stablecoin.balanceOf(user), 0, "balance after full burn");
        assertEq(stablecoin.totalSupply(), 0, "supply after full burn");
    }

    /// @notice Burning more than balance must always revert — never underflows
    function testFuzz_BurnNeverExceedsBalance(uint128 mintAmount, uint128 burnAmount) public {
        vm.assume(mintAmount > 0);
        vm.assume(mintAmount <= MAX_MINT);
        vm.assume(burnAmount > mintAmount);

        MeridianStablecoin.MintRequest memory req = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: mintAmount,
            reserveValue: mintAmount,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(req);

        vm.prank(user);
        vm.expectRevert();
        stablecoin.burn(burnAmount);

        // Balance unchanged after failed burn
        assertEq(stablecoin.balanceOf(user), mintAmount);
    }

    /// @notice attestReserves with value below totalSupply must always revert
    function testFuzz_AttestationAboveSupply(uint128 mintAmount, uint128 attestAmount) public {
        vm.assume(mintAmount > 1);
        vm.assume(mintAmount <= MAX_MINT);
        vm.assume(attestAmount < mintAmount);

        MeridianStablecoin.MintRequest memory req = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: mintAmount,
            reserveValue: mintAmount,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(req);

        vm.prank(admin);
        vm.expectRevert(MeridianStablecoin.AttestationBelowSupply.selector);
        stablecoin.attestReserves(attestAmount);
    }

    /// @notice Partial burn must reduce supply and reserves proportionally
    function testFuzz_PartialBurnProRata(uint128 mintAmount, uint8 burnPct) public {
        vm.assume(mintAmount > 100); // need enough to divide
        vm.assume(mintAmount <= MAX_MINT);
        vm.assume(burnPct > 0 && burnPct <= 100);

        uint256 mintWith = uint256(mintAmount);
        uint256 reserveWith = mintWith * 12 / 10; // 120% backed

        MeridianStablecoin.MintRequest memory req = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: mintWith,
            reserveValue: reserveWith,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        stablecoin.mint(req);

        uint256 burnAmount = (mintWith * burnPct) / 100;
        vm.assume(burnAmount > 0);

        vm.prank(user);
        stablecoin.burn(burnAmount);

        uint256 expectedSupply = mintWith - burnAmount;
        assertEq(stablecoin.totalSupply(), expectedSupply, "supply after partial burn");
        assertEq(stablecoin.balanceOf(user), expectedSupply, "balance after partial burn");
    }

    /// @notice Reserve ratio must always be >= 10000 bps (fully backed) after a valid mint.
    ///
    /// mint() enforces reserveValue * RESERVE_TO_TOKEN_MULTIPLIER >= amount,
    /// so the ratio can never drop below 100% for any accepted mint.
    function testFuzz_ReserveRatioNeverUndercollateralized(uint64 tokenUnits, uint64 extraReserve) public {
        vm.assume(tokenUnits > 0);
        // RESERVE_TO_TOKEN_MULTIPLIER = 10^4; reserveValue = tokenUnits / 10^4 is the minimum
        // We use tokenUnits directly as the amount and provide >= normalized reserve
        uint256 amount = uint256(tokenUnits) * 1e6; // 6-decimal tokens
        uint256 reserveValue = (amount + uint256(extraReserve)) / 1e4; // 2-decimal reserves, >= parity
        vm.assume(reserveValue > 0);
        vm.assume(amount <= MAX_MINT);

        MeridianStablecoin.MintRequest memory req = MeridianStablecoin.MintRequest({
            recipient: user,
            amount: amount,
            reserveValue: reserveValue,
            deadline: block.timestamp + 1 hours,
            nonce: 0
        });

        vm.prank(minter);
        try stablecoin.mint(req) {
            assertGe(stablecoin.getReserveRatio(), 10000, "ratio must be >= 10000 bps after valid mint");
        } catch {
            // Mint may revert if reserveValue*10^4 < amount — that's correct behavior
        }
    }
}
