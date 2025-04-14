# GitDem

GitDem, short for Git Democracy, is a tool that allows you to manage your Git repositories in a decentralized way.

**WIP** - The proof-of-concept is complete - a repository can be pushed to an on-chain contract, cloned and fetched from it, but the project is not ready for production use.

## Overview

It consists of several parts:

- [Git remote helper](./git-remote-evm) that adds support for EVM-compatible blockchains to git
- [Solidity contracts](./on-chain) which handle the on-chain logic
- [CLI](./cli) to deploy and manage repositories via the command line (**Not available yet**)

## License

Dual-licensed under [MIT](./LICENSE-MIT) + [Apache 2.0](./LICENSE-APACHE)
