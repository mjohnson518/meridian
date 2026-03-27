// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/MeridianStablecoin.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract MockComplianceOracleInvariant {
    function isTransferAllowed(address, address, uint256) external pure returns (bool) {
        return true;
    }
}

/// @title MintBurnHandler
/// @notice Stateful handler — Foundry calls its functions randomly to generate action sequences
contract MintBurnHandler is Test {
    MeridianStablecoin public stablecoin;
    address public minter;

    address[] public actors;
    mapping(address => uint256) public nonces;

    uint256 public totalMinted;
    uint256 public totalBurned;

    constructor(MeridianStablecoin _stablecoin, address _minter) {
        stablecoin = _stablecoin;
        minter = _minter;

        actors.push(address(0x100));
        actors.push(address(0x101));
        actors.push(address(0x102));
    }

    function mint(uint128 amount, uint8 actorIdx) public {
        amount = uint128(bound(amount, 1, 100_000 ether));
        address actor = actors[actorIdx % actors.length];

        MeridianStablecoin.MintRequest memory req = MeridianStablecoin.MintRequest({
            recipient: actor,
            amount: amount,
            reserveValue: amount, // 1:1 backing
            deadline: block.timestamp + 1 hours,
            nonce: nonces[actor]
        });

        vm.prank(minter);
        stablecoin.mint(req);

        nonces[actor]++;
        totalMinted += amount;
    }

    function burn(uint128 amount, uint8 actorIdx) public {
        address actor = actors[actorIdx % actors.length];
        uint256 balance = stablecoin.balanceOf(actor);
        if (balance == 0) return;

        amount = uint128(bound(amount, 1, balance));

        vm.prank(actor);
        stablecoin.burn(amount);

        totalBurned += amount;
    }
}

/// @title MeridianStablecoinInvariantTest
/// @notice Invariant tests: properties that must hold across all action sequences
contract MeridianStablecoinInvariantTest is Test {
    MeridianStablecoin public stablecoin;
    MintBurnHandler public handler;

    address public admin = address(0x1);
    address public minter = address(0x2);

    function setUp() public {
        MockComplianceOracleInvariant mockOracle = new MockComplianceOracleInvariant();
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
        stablecoin.grantRole(stablecoin.BURNER_ROLE(), minter);
        vm.stopPrank();

        handler = new MintBurnHandler(stablecoin, minter);

        // Also grant BURNER_ROLE to actors so they can self-burn
        vm.startPrank(admin);
        stablecoin.grantRole(stablecoin.BURNER_ROLE(), address(0x100));
        stablecoin.grantRole(stablecoin.BURNER_ROLE(), address(0x101));
        stablecoin.grantRole(stablecoin.BURNER_ROLE(), address(0x102));
        vm.stopPrank();

        targetContract(address(handler));
    }

    /// @notice totalReserveValue must equal totalMinted - totalBurned (1:1 backing used in handler)
    function invariant_ReserveValueMatchesMints() public view {
        uint256 expectedReserve = handler.totalMinted() - handler.totalBurned();
        assertEq(
            stablecoin.totalReserveValue(),
            expectedReserve,
            "reserve value must equal net minted"
        );
    }

    /// @notice totalSupply must equal totalMinted - totalBurned
    function invariant_SupplyMatchesBalances() public view {
        uint256 expectedSupply = handler.totalMinted() - handler.totalBurned();
        assertEq(
            stablecoin.totalSupply(),
            expectedSupply,
            "supply must equal net minted"
        );
    }

    /// @notice totalSupply must never exceed totalReserveValue (solvency)
    function invariant_AlwaysSolvent() public view {
        assertGe(
            stablecoin.totalReserveValue(),
            stablecoin.totalSupply(),
            "contract must always be solvent"
        );
    }

    /// @notice Sum of actor balances must equal totalSupply
    function invariant_BalanceSumEqualsSupply() public view {
        uint256 sum;
        for (uint256 i = 0; i < 3; i++) {
            sum += stablecoin.balanceOf(handler.actors(i));
        }
        assertEq(sum, stablecoin.totalSupply(), "sum of balances must equal total supply");
    }
}
