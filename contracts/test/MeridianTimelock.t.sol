// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Test.sol";
import "../src/MeridianTimelock.sol";
import "../src/MeridianStablecoin.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract MeridianTimelockTest is Test {
    MeridianTimelock public timelock;
    MeridianStablecoin public implementation;
    MeridianStablecoin public stablecoin;

    address public multisig = address(0xBEEF);
    address public deployer = address(this);
    address public executor = address(0xDEAD);

    function setUp() public {
        // Advance past block.timestamp=1 to avoid collision with OZ's _DONE_TIMESTAMP sentinel
        vm.warp(1_700_000_000);

        // Deploy timelock
        address[] memory proposers = new address[](1);
        proposers[0] = multisig;
        address[] memory executors = new address[](1);
        executors[0] = address(0); // Open execution

        timelock = new MeridianTimelock(proposers, executors, deployer);

        // Deploy stablecoin with timelock as admin
        implementation = new MeridianStablecoin();
        bytes memory initData = abi.encodeWithSelector(
            MeridianStablecoin.initialize.selector,
            "EUR Meridian", "EURM", "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            address(timelock), // admin is the timelock
            address(0)         // no compliance oracle
        );
        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), initData);
        stablecoin = MeridianStablecoin(address(proxy));

        // Renounce deployer admin from timelock
        timelock.renounceRole(timelock.DEFAULT_ADMIN_ROLE(), deployer);
    }

    // ============ Delay Constants ============

    function test_DelayConstants() public view {
        assertEq(timelock.UPGRADE_DELAY(), 48 hours, "Upgrade delay should be 48h");
        assertEq(timelock.PARAM_DELAY(), 24 hours, "Param delay should be 24h");
        assertEq(timelock.EMERGENCY_DELAY(), 0, "Emergency delay should be 0");
    }

    function test_MinDelayStoredAsZero() public view {
        // minDelay is stored as 0 so the base contract doesn't enforce a floor —
        // all tier enforcement is done in our schedule() override.
        assertEq(timelock.getMinDelay(), 0, "Stored minDelay should be 0 (tier enforcement is in override)");
        // But the effective param delay is still accessible via the constant
        assertEq(timelock.PARAM_DELAY(), 24 hours, "PARAM_DELAY constant should be 24h");
    }

    // ============ previewDelay ============

    function test_PreviewDelayForUpgrade() public view {
        bytes memory upgradeCalldata = abi.encodeWithSignature(
            "upgradeToAndCall(address,bytes)", address(0x123), bytes("")
        );
        assertEq(timelock.previewDelay(upgradeCalldata), 48 hours, "Upgrade should require 48h");
    }

    function test_PreviewDelayForPause() public view {
        bytes memory pauseCalldata = abi.encodeWithSignature("pause()");
        assertEq(timelock.previewDelay(pauseCalldata), 0, "Pause should require 0 delay");
    }

    function test_PreviewDelayForUnpause() public view {
        bytes memory unpauseCalldata = abi.encodeWithSignature("unpause()");
        assertEq(timelock.previewDelay(unpauseCalldata), 0, "Unpause should require 0 delay");
    }

    function test_PreviewDelayForSetReserveRatio() public view {
        bytes memory paramCalldata = abi.encodeWithSignature("setMinReserveRatio(uint256)", 11000);
        assertEq(timelock.previewDelay(paramCalldata), 24 hours, "Reserve ratio change should require 24h");
    }

    function test_PreviewDelayForEmptyCalldata() public view {
        assertEq(timelock.previewDelay(""), 24 hours, "Empty calldata should use PARAM_DELAY");
    }

    // ============ Schedule Enforcement ============

    function test_ScheduleUpgradeEnforcesMinimumDelay() public {
        bytes memory upgradeCalldata = abi.encodeWithSignature(
            "upgradeToAndCall(address,bytes)", address(0x999), bytes("")
        );
        bytes32 salt = keccak256("upgrade-test");

        // Try to schedule with only 1 hour (less than UPGRADE_DELAY)
        vm.prank(multisig);
        // Should NOT revert — schedule() bumps the delay to UPGRADE_DELAY
        timelock.schedule(
            address(stablecoin),
            0,
            upgradeCalldata,
            bytes32(0),
            salt,
            1 hours // Too short — should be bumped to 48h
        );

        // Verify the operation is scheduled with the enforced delay
        bytes32 opId = timelock.hashOperation(
            address(stablecoin), 0, upgradeCalldata, bytes32(0), salt
        );
        assertTrue(timelock.isOperationPending(opId), "Upgrade op should be pending");

        // Should NOT be ready yet after only 1 hour
        vm.warp(block.timestamp + 1 hours);
        assertFalse(timelock.isOperationReady(opId), "Upgrade op should not be ready after 1h");

        // Should be ready after 48 hours
        vm.warp(block.timestamp + 47 hours);
        assertTrue(timelock.isOperationReady(opId), "Upgrade op should be ready after 48h");
    }

    function test_SchedulePauseWithZeroDelay() public {
        bytes memory pauseCalldata = abi.encodeWithSignature("pause()");
        bytes32 salt = keccak256("pause-test");

        vm.prank(multisig);
        timelock.schedule(
            address(stablecoin),
            0,
            pauseCalldata,
            bytes32(0),
            salt,
            0 // Zero delay — must be honoured for emergency
        );

        bytes32 opId = timelock.hashOperation(
            address(stablecoin), 0, pauseCalldata, bytes32(0), salt
        );

        // Should be immediately ready
        assertTrue(timelock.isOperationReady(opId), "Pause op should be immediately ready");
    }

    function test_ScheduleParamChangeEnforcesMinimumDelay() public {
        bytes memory paramCalldata = abi.encodeWithSignature("setMinReserveRatio(uint256)", 11000);
        bytes32 salt = keccak256("param-test");

        vm.prank(multisig);
        timelock.schedule(
            address(stablecoin),
            0,
            paramCalldata,
            bytes32(0),
            salt,
            1 hours // Too short — should be bumped to 24h
        );

        bytes32 opId = timelock.hashOperation(
            address(stablecoin), 0, paramCalldata, bytes32(0), salt
        );

        // Should NOT be ready after 1 hour
        vm.warp(block.timestamp + 1 hours);
        assertFalse(timelock.isOperationReady(opId), "Param op should not be ready after 1h");

        // Should be ready after 24 hours
        vm.warp(block.timestamp + 23 hours);
        assertTrue(timelock.isOperationReady(opId), "Param op should be ready after 24h");
    }

    function test_LongerDelayThanMinimumIsHonoured() public {
        bytes memory pauseCalldata = abi.encodeWithSignature("pause()");
        bytes32 salt = keccak256("longer-delay-test");

        // Schedule pause with 6 hours even though emergency delay is 0
        vm.prank(multisig);
        timelock.schedule(
            address(stablecoin), 0, pauseCalldata, bytes32(0), salt, 6 hours
        );

        bytes32 opId = timelock.hashOperation(
            address(stablecoin), 0, pauseCalldata, bytes32(0), salt
        );

        // Not ready yet (user chose 6h)
        assertFalse(timelock.isOperationReady(opId), "Should respect user-provided longer delay");

        vm.warp(block.timestamp + 6 hours);
        assertTrue(timelock.isOperationReady(opId), "Should be ready after user-provided delay");
    }

    function test_OnlyProposerCanSchedule() public {
        bytes memory pauseCalldata = abi.encodeWithSignature("pause()");

        // Non-proposer should revert
        vm.prank(address(0xCAFE));
        vm.expectRevert();
        timelock.schedule(address(stablecoin), 0, pauseCalldata, bytes32(0), keccak256("test"), 0);
    }

    // ============ DelayEnforced Event ============

    function test_DelayEnforcedEventEmittedWhenDelayBumped() public {
        bytes memory upgradeCalldata = abi.encodeWithSignature(
            "upgradeToAndCall(address,bytes)", address(0x999), bytes("")
        );
        bytes32 salt = keccak256("event-test");

        vm.prank(multisig);
        vm.expectEmit(false, false, false, false); // Just check event is emitted
        emit MeridianTimelock.DelayEnforced(bytes32(0), bytes4(0), 0, 0);
        timelock.schedule(address(stablecoin), 0, upgradeCalldata, bytes32(0), salt, 1 hours);
    }

    // ============ Batch Schedule ============

    function test_BatchScheduleEnforcesMaxDelay() public {
        address[] memory targets = new address[](2);
        targets[0] = address(stablecoin);
        targets[1] = address(stablecoin);

        uint256[] memory values = new uint256[](2);

        bytes[] memory payloads = new bytes[](2);
        payloads[0] = abi.encodeWithSignature("pause()");          // 0 delay
        payloads[1] = abi.encodeWithSignature("upgradeToAndCall(address,bytes)", address(0x999), bytes("")); // 48h

        bytes32 salt = keccak256("batch-test");

        uint256 scheduledAt = block.timestamp;

        vm.prank(multisig);
        // Despite pause having 0 delay, the batch should enforce 48h (max of all ops)
        timelock.scheduleBatch(targets, values, payloads, bytes32(0), salt, 1 hours);

        bytes32 opId = timelock.hashOperationBatch(targets, values, payloads, bytes32(0), salt);

        vm.warp(scheduledAt + 24 hours);
        assertFalse(timelock.isOperationReady(opId), "Batch should not be ready at 24h (max is 48h)");

        vm.warp(scheduledAt + 48 hours);
        assertTrue(timelock.isOperationReady(opId), "Batch should be ready at 48h");
    }
}
