// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Script.sol";
import "../src/MeridianStablecoin.sol";
import "../src/MeridianFactory.sol";

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

