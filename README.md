# GitDem

GitDem, short for Git Democracy, is a tool that allows you to manage your Git repositories in a decentralized way.

## Overview

It consists of several parts:

- Git remote helper that adds Solana support to git
- Solana contracts to manage the on-chain side of things
- Web app to manage your repositories and their configurations

### Git remote helper

The git remote helper is a tool that adds Solana support to git. It allows you to push and pull your repositories from the blockchain. 

You can find more about git remote helpers [here](./git-remote-sol/gitremote-helpers.adoc)

### Solana contracts

The Solana contracts handles the on-chain metadata for your repositories, the logic of proposals and voting, minting of governance tokens, as well as the minting of NFTs used as achievements for the contributors.

The actual git objects and deltas are stored on Arweave/IPFS (**TBD**) and referenced in the on-chain metadata.

### Web app

The web app is similar to other platforms that you're probably familiar with, but with extra features suited for the decentralized and democratic nature of GitDem.

## License

This project is licensed under the GNU Affero General Public License v3.0. See the [LICENSE](LICENSE) file for details.
