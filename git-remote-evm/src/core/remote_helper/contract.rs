use alloy::sol;

use crate::core::remote_helper::error::RemoteHelperError;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Lock,
    "../on-chain/artifacts/contracts/Lock.sol/Lock.json"
);
