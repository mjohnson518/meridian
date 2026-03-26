// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "@openzeppelin/contracts/governance/TimelockController.sol";

/**
 * @title MeridianTimelock
 * @author Meridian Team
 * @notice Timelock controller enforcing delay tiers for different operation categories.
 *
 * @dev Tiered delays:
 *  - UPGRADE_DELAY  (48 hours): UUPS upgrades, implementation changes
 *  - PARAM_DELAY    (24 hours): reserve ratio, compliance oracle address, blacklist
 *  - EMERGENCY_DELAY (0 hours): pause/unpause (no delay — must respond immediately to exploits)
 *
 * Deployment pattern:
 *  1. Deploy MeridianTimelock with a Gnosis Safe as proposer+executor
 *  2. Transfer DEFAULT_ADMIN_ROLE on MeridianStablecoin from EOA to this timelock address
 *  3. The Safe proposes operations; they execute after the appropriate delay
 *
 * Multi-sig configuration (recommended):
 *  - proposers: [GnosisSafe_3_of_5]
 *  - executors: [GnosisSafe_3_of_5]  (or address(0) for open execution after delay)
 *  - minDelay: PARAM_DELAY (enforces minimum; upgrade delay enforced in schedule override)
 *
 * Function selectors that trigger UPGRADE_DELAY (48h):
 *  - upgradeToAndCall(address,bytes)
 *  - upgradeTo(address)          [v4 compatibility]
 *
 * Function selectors that trigger EMERGENCY_DELAY (0h, no wait):
 *  - pause()
 *  - unpause()
 *
 * All other admin calls (setMinReserveRatio, setComplianceOracle, blacklist, etc.) use PARAM_DELAY.
 */
contract MeridianTimelock is TimelockController {
    // ============ Delay Constants ============

    /// @notice 48-hour delay for contract upgrades
    uint256 public constant UPGRADE_DELAY = 48 hours;

    /// @notice 24-hour delay for parameter changes (reserve ratio, oracle address, blacklist)
    uint256 public constant PARAM_DELAY = 24 hours;

    /// @notice 0 delay for emergency pause/unpause — immediate execution required for security incidents
    uint256 public constant EMERGENCY_DELAY = 0;

    // ============ Function Selector Constants ============

    /// @notice upgradeToAndCall(address,bytes) — OZ v5 UUPS upgrade entrypoint
    bytes4 private constant UPGRADE_TO_AND_CALL_SELECTOR =
        bytes4(keccak256("upgradeToAndCall(address,bytes)"));

    /// @notice upgradeTo(address) — v4 compatibility
    bytes4 private constant UPGRADE_TO_SELECTOR =
        bytes4(keccak256("upgradeTo(address)"));

    /// @notice pause()
    bytes4 private constant PAUSE_SELECTOR = bytes4(keccak256("pause()"));

    /// @notice unpause()
    bytes4 private constant UNPAUSE_SELECTOR = bytes4(keccak256("unpause()"));

    // ============ Events ============

    /// @notice Emitted when an operation is scheduled with an enforced minimum delay
    event DelayEnforced(
        bytes32 indexed id,
        bytes4 selector,
        uint256 requestedDelay,
        uint256 enforcedDelay
    );

    // ============ Constructor ============

    /**
     * @notice Deploy the Meridian timelock
     * @param proposers Addresses allowed to schedule operations (typically a Gnosis Safe)
     * @param executors Addresses allowed to execute operations (typically same Safe, or address(0))
     * @param admin    Initial admin; set to address(0) after setup to renounce admin control
     *
     * Note: `minDelay` is set to 0 so that our `schedule` override has full control over
     * per-operation delay enforcement. Emergency ops (pause/unpause) get 0 delay while
     * upgrades get 48h and other admin ops get 24h — all enforced in _requiredDelay().
     */
    constructor(
        address[] memory proposers,
        address[] memory executors,
        address admin
    ) TimelockController(0, proposers, executors, admin) {}

    // ============ Schedule Override ============

    /**
     * @notice Schedule a single operation with delay enforcement by operation category.
     * @dev Overrides TimelockController.schedule to enforce tiered delays:
     *      - Upgrade operations: minimum UPGRADE_DELAY (48h)
     *      - Emergency (pause/unpause): enforced delay of 0
     *      - All other admin ops: minimum PARAM_DELAY (24h, inherited minDelay)
     *
     * If the caller provides a shorter delay than the tier minimum, it is silently
     * extended to the required minimum. If the caller provides a longer delay, it is honoured.
     */
    function schedule(
        address target,
        uint256 value,
        bytes calldata data,
        bytes32 predecessor,
        bytes32 salt,
        uint256 delay
    ) public override {
        uint256 requiredDelay = _requiredDelay(data);
        uint256 enforcedDelay = delay < requiredDelay ? requiredDelay : delay;

        if (enforcedDelay != delay) {
            bytes32 id = hashOperation(target, value, data, predecessor, salt);
            bytes4 sel = data.length >= 4 ? bytes4(data[:4]) : bytes4(0);
            emit DelayEnforced(id, sel, delay, enforcedDelay);
        }

        super.schedule(target, value, data, predecessor, salt, enforcedDelay);
    }

    /**
     * @notice Schedule a batch of operations with per-operation delay enforcement.
     * @dev The maximum required delay across all operations in the batch is enforced.
     */
    function scheduleBatch(
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata payloads,
        bytes32 predecessor,
        bytes32 salt,
        uint256 delay
    ) public override {
        uint256 maxRequiredDelay = 0;
        for (uint256 i = 0; i < payloads.length; i++) {
            uint256 required = _requiredDelay(payloads[i]);
            if (required > maxRequiredDelay) {
                maxRequiredDelay = required;
            }
        }

        uint256 enforcedDelay = delay < maxRequiredDelay ? maxRequiredDelay : delay;
        super.scheduleBatch(targets, values, payloads, predecessor, salt, enforcedDelay);
    }

    // ============ Internal Helpers ============

    /**
     * @notice Determine the required minimum delay for a given calldata payload.
     * @param data Calldata for the operation
     * @return Minimum required delay in seconds
     */
    function _requiredDelay(bytes calldata data) internal pure returns (uint256) {
        if (data.length < 4) {
            return PARAM_DELAY;
        }

        bytes4 selector = bytes4(data[:4]);

        if (selector == UPGRADE_TO_AND_CALL_SELECTOR || selector == UPGRADE_TO_SELECTOR) {
            return UPGRADE_DELAY;
        }

        if (selector == PAUSE_SELECTOR || selector == UNPAUSE_SELECTOR) {
            return EMERGENCY_DELAY;
        }

        return PARAM_DELAY;
    }

    // ============ Convenience View ============

    /**
     * @notice Preview the delay that would be enforced for a given calldata.
     * @param data Calldata to inspect
     * @return The minimum delay that will be enforced
     */
    function previewDelay(bytes calldata data) external pure returns (uint256) {
        return _requiredDelay(data);
    }

}
// Note: getMinDelay() intentionally returns 0 (the stored minDelay) so that the base
// TimelockController does not double-enforce a minimum. All delay enforcement is performed
// in the schedule()/scheduleBatch() overrides above via _requiredDelay(). External tooling
// and UIs should read PARAM_DELAY and UPGRADE_DELAY constants directly to understand the
// effective delay tiers.
