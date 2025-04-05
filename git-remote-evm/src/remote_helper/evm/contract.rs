use alloy::sol;

sol!(
  #[allow(missing_docs)]
  #[sol(rpc)]
  Lock,
  "../on-chain/artifacts/contracts/Lock.sol/Lock.json"
);
