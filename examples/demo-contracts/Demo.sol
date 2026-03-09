// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title DemoVault
/// @notice Simple vault used to pair with the Stylus `vault.rs` example.
contract DemoVault {
    uint256 public balance;

    event Deposited(address indexed from, uint256 amount);
    event Withdrawn(address indexed to, uint256 amount);

    function deposit() external payable {
        require(msg.value > 0, "no value");
        balance += msg.value;
        emit Deposited(msg.sender, msg.value);
    }

    function withdraw(uint256 amount) external {
        require(amount <= balance, "insufficient");
        balance -= amount;
        (bool ok, ) = msg.sender.call{value: amount}("");
        require(ok, "transfer failed");
        emit Withdrawn(msg.sender, amount);
    }

    function depositAndWithdrawHalf() external payable {
        deposit();
        uint256 half = msg.value / 2;
        withdraw(half);
    }
}


