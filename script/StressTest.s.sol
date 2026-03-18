// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";

contract StateBloater {
    mapping(uint256 => bytes32) public data;

    function bloat(uint256 reps) public {
        for (uint256 i = 0; i < reps; i++) {
            data[uint256(keccak256(abi.encode(block.timestamp, i)))] = keccak256(abi.encode(msg.sender, i));
        }
    }
}

contract StressTest is Script {
    function run() public {
        // Default Anvil Account 0 (0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80)
        uint256 deployerPrivateKey = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;

        // We run 1000 independent transactions to strictly assert the mempool and RPC state trie persistence boundaries
        // Exceeding 1000 on localhost without heavy batching may trigger naive foundry RPC timeout rate limits.
        for (uint256 i = 0; i < 1000; i++) {
            vm.startBroadcast(deployerPrivateKey);
            StateBloater bloater = new StateBloater();
            bloater.bloat(10); // Internal storage expansion per transaction
            vm.stopBroadcast();
        }
    }
}
