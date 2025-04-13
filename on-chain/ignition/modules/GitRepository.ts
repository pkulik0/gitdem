// This setup uses Hardhat Ignition to manage smart contract deployments.
// Learn more about it at https://hardhat.org/ignition

import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";

const GitRepositoryModule = buildModule("GitRepositoryModule", (m) => {
  const isSHA256 = m.getParameter("isSHA256", false);

  const gitRepository = m.contract("GitRepository", [isSHA256]);

  return { gitRepository };
});

export default GitRepositoryModule;
