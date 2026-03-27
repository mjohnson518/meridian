// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Script.sol";
import "../src/MeridianStablecoin.sol";
import "../src/MeridianFactory.sol";
import "../src/MeridianTimelock.sol";

/**
 * @title DeployScript
 * @notice Deployment script for Meridian contracts
 * 
 * Usage:
 * forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast --verify
 */
contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        vm.startBroadcast(deployerPrivateKey);

        // Deploy implementation
        console.log("Deploying MeridianStablecoin implementation...");
        MeridianStablecoin implementation = new MeridianStablecoin();
        console.log("Implementation deployed at:", address(implementation));

        // Deploy factory
        console.log("Deploying MeridianFactory...");
        MeridianFactory factory = new MeridianFactory(address(implementation));
        console.log("Factory deployed at:", address(factory));

        vm.stopBroadcast();

        console.log("\n=== Deployment Complete ===");
        console.log("Implementation:", address(implementation));
        console.log("Factory:", address(factory));
    }
}

/**
 * @title DeployTimelockScript
 * @notice Deploy the MeridianTimelock with a multi-sig as proposer/executor.
 *
 * Required env vars:
 *   PRIVATE_KEY          — deployer key (temp admin, renounced after setup)
 *   MULTISIG_ADDRESS     — Gnosis Safe address (proposer + executor)
 *
 * After deployment, call grantRole(DEFAULT_ADMIN_ROLE, timelockAddress) on each
 * MeridianStablecoin, then revokeRole(DEFAULT_ADMIN_ROLE, currentEOA).
 *
 * Usage:
 * forge script script/Deploy.s.sol:DeployTimelockScript --rpc-url $RPC_URL --broadcast --verify
 */
contract DeployTimelockScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployerAddress = vm.addr(deployerPrivateKey);
        address multisig = vm.envAddress("MULTISIG_ADDRESS");

        address[] memory proposers = new address[](1);
        proposers[0] = multisig;

        // address(0) as executor allows anyone to execute after the delay —
        // safer than locking execution to a single address that could lose access
        address[] memory executors = new address[](1);
        executors[0] = address(0);

        vm.startBroadcast(deployerPrivateKey);

        console.log("Deploying MeridianTimelock...");
        console.log("  Proposer (multi-sig):", multisig);
        console.log("  Executor: open (address(0))");
        console.log("  Param delay:", MeridianTimelock(payable(address(0))).PARAM_DELAY() / 3600, "hours");
        console.log("  Upgrade delay:", MeridianTimelock(payable(address(0))).UPGRADE_DELAY() / 3600, "hours");

        MeridianTimelock timelock = new MeridianTimelock(
            proposers,
            executors,
            deployerAddress // Temporary admin for initial role setup
        );

        console.log("MeridianTimelock deployed at:", address(timelock));

        // Renounce deployer's admin role — timelock is now self-governing
        timelock.renounceRole(timelock.DEFAULT_ADMIN_ROLE(), deployerAddress);
        console.log("Deployer admin role renounced. Timelock is now self-governed.");

        vm.stopBroadcast();

        console.log("\n=== Timelock Deployment Complete ===");
        console.log("Timelock:", address(timelock));
        console.log("\nNext steps:");
        console.log("1. On each MeridianStablecoin, call:");
        console.log("   grantRole(DEFAULT_ADMIN_ROLE, ", address(timelock), ")");
        console.log("   revokeRole(DEFAULT_ADMIN_ROLE, <current-EOA>)");
        console.log("2. Verify: all admin ops now require Gnosis Safe + ", MeridianTimelock(payable(address(timelock))).PARAM_DELAY() / 3600, "h delay");
    }
}

/**
 * @title DeployProductionScript
 * @notice Full production deployment: Implementation + Timelock + Factory wired together.
 *
 * Deployment order:
 *  1. Deploy MeridianStablecoin implementation
 *  2. Deploy MeridianTimelock (proposer = MULTISIG_ADDRESS, executor = open)
 *  3. Deploy MeridianFactory (deployer as initial owner)
 *  4. Transfer Factory ownership to Timelock
 *  5. Renounce deployer's timelock admin role
 *
 * After this script, all Factory operations (deployStablecoin, updateImplementation)
 * require a Gnosis Safe proposal + 24h timelock delay.
 *
 * Required env vars:
 *   PRIVATE_KEY        — deployer EOA (used only during this script, no residual authority)
 *   MULTISIG_ADDRESS   — Gnosis Safe that will govern the system
 *
 * Usage:
 * forge script script/Deploy.s.sol:DeployProductionScript --rpc-url $RPC_URL --broadcast --verify
 */
contract DeployProductionScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployerAddress = vm.addr(deployerPrivateKey);
        address multisig = vm.envAddress("MULTISIG_ADDRESS");

        vm.startBroadcast(deployerPrivateKey);

        // 1. Deploy implementation
        console.log("Deploying MeridianStablecoin implementation...");
        MeridianStablecoin implementation = new MeridianStablecoin();
        console.log("  Implementation:", address(implementation));

        // 2. Deploy Timelock (multisig is proposer; open execution after delay)
        address[] memory proposers = new address[](1);
        proposers[0] = multisig;
        address[] memory executors = new address[](1);
        executors[0] = address(0); // open execution — anyone can execute after delay

        console.log("Deploying MeridianTimelock...");
        MeridianTimelock timelock = new MeridianTimelock(
            proposers,
            executors,
            deployerAddress // temporary admin, renounced below
        );
        console.log("  Timelock:", address(timelock));

        // 3. Deploy Factory (deployer is initial owner)
        console.log("Deploying MeridianFactory...");
        MeridianFactory factory = new MeridianFactory(address(implementation));
        console.log("  Factory:", address(factory));

        // 4. Transfer Factory ownership to Timelock
        factory.transferOwnership(address(timelock));
        console.log("  Factory ownership transferred to Timelock");

        // 5. Renounce deployer's timelock admin — timelock is now self-governed
        timelock.renounceRole(timelock.DEFAULT_ADMIN_ROLE(), deployerAddress);
        console.log("  Deployer timelock admin role renounced");

        vm.stopBroadcast();

        console.log("\n=== Production Deployment Complete ===");
        console.log("Implementation:", address(implementation));
        console.log("Timelock:      ", address(timelock));
        console.log("Factory:       ", address(factory));
        console.log("\nGovernance:");
        console.log("  Factory owner:    ", address(timelock), "(Timelock)");
        console.log("  Timelock proposer:", multisig, "(Gnosis Safe)");
        console.log("  Upgrade delay:    48h  (UPGRADE_DELAY)");
        console.log("  Param delay:      24h  (PARAM_DELAY)");
        console.log("  Emergency delay:   0h  (pause/unpause)");
        console.log("\nNext step: deploy stablecoins via Gnosis Safe proposing to Factory");
    }
}

/**
 * @title DeployEURScript
 * @notice Script to deploy a EUR stablecoin instance
 * 
 * Usage:
 * forge script script/Deploy.s.sol:DeployEURScript --rpc-url $RPC_URL --broadcast
 */
contract DeployEURScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address factoryAddress = vm.envAddress("FACTORY_ADDRESS");
        address adminAddress = vm.envAddress("ADMIN_ADDRESS");
        address complianceOracleAddress = vm.envAddress("COMPLIANCE_ORACLE_ADDRESS");
        
        MeridianFactory factory = MeridianFactory(factoryAddress);

        vm.startBroadcast(deployerPrivateKey);

        console.log("Deploying EUR stablecoin...");
        address eurStablecoin = factory.deployStablecoin(
            "EUR Meridian",
            "EURM",
            "EUR_BASKET",
            MeridianStablecoin.BasketType.SingleCurrency,
            adminAddress,
            complianceOracleAddress
        );

        console.log("EUR Stablecoin deployed at:", eurStablecoin);

        vm.stopBroadcast();
    }
}

/**
 * @title DeploySDRScript
 * @notice Script to deploy an IMF SDR stablecoin instance
 * 
 * Usage:
 * forge script script/Deploy.s.sol:DeploySDRScript --rpc-url $RPC_URL --broadcast
 */
contract DeploySDRScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address factoryAddress = vm.envAddress("FACTORY_ADDRESS");
        address adminAddress = vm.envAddress("ADMIN_ADDRESS");
        address complianceOracleAddress = vm.envAddress("COMPLIANCE_ORACLE_ADDRESS");
        
        MeridianFactory factory = MeridianFactory(factoryAddress);

        vm.startBroadcast(deployerPrivateKey);

        console.log("Deploying IMF SDR stablecoin...");
        address sdrStablecoin = factory.deployStablecoin(
            "SDR Meridian",
            "SDRM",
            "SDR_BASKET",
            MeridianStablecoin.BasketType.ImfSdr,
            adminAddress,
            complianceOracleAddress
        );

        console.log("SDR Stablecoin deployed at:", sdrStablecoin);

        vm.stopBroadcast();
    }
}

