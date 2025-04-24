# GitDem

GitDem, short for Git Democracy, is a tool that allows you to manage your Git repositories in a decentralized way.

**WIP** - The proof-of-concept is complete - a repository can be pushed to an on-chain contract, cloned and fetched from it, but the project is not ready for production use. To make this viable, we need to implement a better storage solution than storing the entire repository on-chain i.e. use IPFS or Arweave for the git objects and store the metadata on-chain.

## Overview

It consists of several parts:

- [Git remote helper](./git-remote-evm) that adds support for EVM-compatible blockchains to git
- [Solidity contracts](./on-chain) which handle the on-chain logic

## License

Dual-licensed under [MIT](./LICENSE-MIT) + [Apache 2.0](./LICENSE-APACHE)
