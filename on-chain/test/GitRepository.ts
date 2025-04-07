import {
  time,
  loadFixture,
} from "@nomicfoundation/hardhat-toolbox/network-helpers";
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import { expect } from "chai";
import hre from "hardhat";
import { ethers } from "hardhat";
import crypto, { hash } from "crypto";

function generateHash(isSHA256: boolean, data?: Uint8Array): Uint8Array {
  const hash = crypto.createHash(isSHA256 ? "sha256" : "sha1")
    .update(data ?? crypto.randomBytes(100))
    .digest();

  const bytes = new Uint8Array(32);
  bytes.set(hash);
  return bytes;
}

describe("GitRepository", function () {
  async function deployGitRepositoryFixtureBase(isSHA256: boolean) {
    const [owner, otherAccount] = await hre.ethers.getSigners();

    const GitRepository = await hre.ethers.getContractFactory("GitRepository");
    const gitRepository = await GitRepository.deploy(isSHA256);

    return { gitRepository, owner, otherAccount };
  }

  async function deployGitRepositoryFixtureSHA1() {
    return deployGitRepositoryFixtureBase(false);
  }

  async function deployGitRepositoryFixture() {
    return deployGitRepositoryFixtureBase(true);
  }

  describe("Deployment", function () {
    it("should set the owner to the deployer", async function () {
      const { gitRepository, owner } = await loadFixture(deployGitRepositoryFixture);

      expect(await gitRepository.owner()).to.equal(owner.address);
    });

    it("can transfer ownership with confirmation", async function () {
      const { gitRepository, otherAccount } = await loadFixture(deployGitRepositoryFixture);

      await gitRepository.transferOwnership(otherAccount.address);
      expect(await gitRepository.pendingOwner()).to.equal(otherAccount.address);

      await gitRepository.connect(otherAccount).acceptOwnership();
      expect(await gitRepository.owner()).to.equal(otherAccount.address);
    });
  });

  describe("Objects", function () {
    describe("Adding", function () {
      it("can add SHA256", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const data = crypto.randomBytes(100);
        const hash = generateHash(true, data);
        await gitRepository.addObject(hash, data);
      });

      it("can add SHA1", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixtureSHA1);

        const data = crypto.randomBytes(100);
        const hash = generateHash(false, data);
        await gitRepository.addObject(hash, data);
      });

      it("can't add with a SHA256 hash to a SHA1 repository", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixtureSHA1);

        const data = crypto.randomBytes(100);
        const hash = generateHash(true, data);
        await expect(gitRepository.addObject(hash, data)).to.be.revertedWith("Hash mismatch");
      });

      it("can't add with a SHA1 hash to a SHA256 repository", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const data = crypto.randomBytes(100);
        const hash = generateHash(false, data);
        await expect(gitRepository.addObject(hash, data)).to.be.revertedWith("Hash mismatch");
      });

      it("can't add with an empty hash", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const data = crypto.randomBytes(100);
        const hash = new Uint8Array(32);
        await expect(gitRepository.addObject(hash, data)).to.be.revertedWith("Hash is empty");
      });

      it("can't add an empty object", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const data = new Uint8Array(0);
        const hash = generateHash(true, data);
        await expect(gitRepository.addObject(hash, data)).to.be.revertedWith("Object is empty");
      });

      it("can't add if not the owner", async function () {
        const { gitRepository, otherAccount } = await loadFixture(
          deployGitRepositoryFixture
        );

        const hash = generateHash(true);
        await expect(gitRepository.connect(otherAccount).addObject(hash, new Uint8Array(32)))
          .to.be.revertedWithCustomError(gitRepository, "OwnableUnauthorizedAccount")
          .withArgs(otherAccount.address);
      });

      it("can't add if the object already exists", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const data = crypto.randomBytes(100);
        const hash = generateHash(true, data);
        await gitRepository.addObject(hash, data);
        await expect(gitRepository.addObject(hash, data)).to.be.revertedWith("Object already exists");
      });
    });

    describe("Getting", function () {
      it("can get", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const data = crypto.randomBytes(100);
        const hash = generateHash(true, data);
        await gitRepository.addObject(hash, data);

        const object = await gitRepository.getObject(hash);
        expect(ethers.getBytes(object)).to.deep.equal(data);
      });

      it("can't get if no objects exist", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const hash = generateHash(true);
        await expect(gitRepository.getObject(hash)).to.be.revertedWith("Object not found");
      });

      it("can't get if the object doesn't exist", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const data = crypto.randomBytes(100);
        const hash = generateHash(true, data);
        await gitRepository.addObject(hash, data);

        const hash2 = generateHash(true);
        await expect(gitRepository.getObject(hash2)).to.be.revertedWith("Object not found");
      });

      it("anyone can get an object", async function () {
        const { gitRepository, otherAccount } = await loadFixture(deployGitRepositoryFixture);

        const data = crypto.randomBytes(100);
        const hash = generateHash(true, data);
        await gitRepository.addObject(hash, data);

        const object = await gitRepository.connect(otherAccount).getObject(hash);
        expect(ethers.getBytes(object)).to.deep.equal(data);
      });
    });
  });

  describe("Refs", function () {
    async function existingObjectFixtureBase(isSHA256: boolean) {
      const { gitRepository, owner, otherAccount } = await deployGitRepositoryFixtureBase(isSHA256);

      const data = crypto.randomBytes(100);
      const hash = generateHash(isSHA256, data);
      await gitRepository.addObject(hash, data);

      const otherData = crypto.randomBytes(100);
      const otherHash = generateHash(isSHA256, otherData);
      await gitRepository.addObject(otherHash, otherData);

      return { gitRepository, owner, otherAccount, hash, otherHash };
    }

    async function existingObjectFixtureSHA1() {
      return existingObjectFixtureBase(false);
    }

    async function existingObjectFixture() {
      return existingObjectFixtureBase(true);
    }

    describe("Getting", function () {
      it("can get", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixture);

        await gitRepository.upsertRef("refs/heads/main", hash);

        const value = await gitRepository.getRef("refs/heads/main");
        expect(ethers.getBytes(value)).to.deep.equal(hash);
      });

      it("can't get non-existent", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        await expect(gitRepository.getRef("refs/heads/main")).to.be.revertedWith("Ref not found");
      });
    });

    describe("Upserting", function () {
      it("can add with a SHA256 hash", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixture);

        await gitRepository.upsertRef("refs/heads/main", hash);

        const value = await gitRepository.getRef("refs/heads/main");
        expect(ethers.getBytes(value)).to.deep.equal(hash);
      });

      it("can add with a SHA1 hash", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixtureSHA1);
        await gitRepository.upsertRef("refs/heads/main", hash);
      });

      it("can't add with non-existent hash", async function () {
        const { gitRepository } = await loadFixture(existingObjectFixture);

        const hash = generateHash(true);
        await expect(gitRepository.upsertRef("refs/heads/main", hash)).to.be.revertedWith(
          "Object not found"
        );
      });

      it("can't add with an empty hash", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const hash = new Uint8Array(32);
        await expect(gitRepository.upsertRef("refs/heads/main", hash)).to.be.revertedWith(
          "Hash is empty"
        );
      });

      it("can't add if not the owner", async function () {
        const { gitRepository, otherAccount, hash } = await loadFixture(existingObjectFixture);

        await expect(gitRepository.connect(otherAccount).upsertRef("refs/heads/main", hash))
          .to.be.revertedWithCustomError(gitRepository, "OwnableUnauthorizedAccount")
          .withArgs(otherAccount.address);
      });

      it("can't add with an invalid name", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixture);

        await expect(gitRepository.upsertRef("", hash)).to.be.revertedWith(
          "Name is invalid"
        );
      });

      it("can update with a new hash", async function () {
        const { gitRepository, hash, otherHash } = await loadFixture(existingObjectFixture);

        await gitRepository.upsertRef("refs/heads/main", hash);
        const value = await gitRepository.getRef("refs/heads/main");
        expect(ethers.getBytes(value)).to.deep.equal(hash);

        await gitRepository.upsertRef("refs/heads/main", otherHash);
        const newValue = await gitRepository.getRef("refs/heads/main");
        expect(ethers.getBytes(newValue)).to.deep.equal(otherHash);
      });
    });

    describe("Listing", function () {
      it("can list a single ref", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixture);

        await gitRepository.upsertRef("refs/heads/main", hash);

        const refs = await gitRepository.listRefs();
        expect(refs.length).to.equal(1);
        expect(refs[0].name).to.equal("refs/heads/main");
        expect(ethers.getBytes(refs[0].hash)).to.deep.equal(hash);
      });

      it("can list multiple refs", async function () {
        const { gitRepository, hash, otherHash } = await loadFixture(existingObjectFixture);

        await gitRepository.upsertRef("refs/heads/main", hash);
        await gitRepository.upsertRef("refs/heads/other", otherHash);

        const refs = await gitRepository.listRefs();
        expect(refs.length).to.equal(2);
        expect(refs[0].name).to.equal("refs/heads/main");
        expect(ethers.getBytes(refs[0].hash)).to.deep.equal(hash);
        expect(refs[1].name).to.equal("refs/heads/other");
        expect(ethers.getBytes(refs[1].hash)).to.deep.equal(otherHash);
      });

      it("can list with no refs", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const refs = await gitRepository.listRefs();
        expect(refs.length).to.equal(0);
      });

      it("can list with SHA1 hash type", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixtureSHA1);

        await gitRepository.upsertRef("refs/heads/main", hash);

        const refs = await gitRepository.listRefs();
        expect(refs.length).to.equal(1);
        expect(refs[0].name).to.equal("refs/heads/main");
        expect(ethers.getBytes(refs[0].hash)).to.deep.equal(hash);
      });

      it("anyone can list refs", async function () {
        const { gitRepository, otherAccount, hash } = await loadFixture(
          existingObjectFixture
        );

        await gitRepository.upsertRef("refs/heads/main", hash);

        const refs = await gitRepository.connect(otherAccount).listRefs();
        expect(refs.length).to.equal(1);
        expect(refs[0].name).to.equal("refs/heads/main");
        expect(ethers.getBytes(refs[0].hash)).to.deep.equal(hash);
      });

      it("can handle a large number of refs", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixture);

        const count = 512;
        for (let i = 0; i < count; i++) {
          await gitRepository.upsertRef(`refs/heads/ref${i}`, hash);
        }

        const refs = await gitRepository.listRefs();
        expect(refs.length).to.equal(count);
        for (let i = 0; i < count; i++) {
          expect(refs[i].name).to.equal(`refs/heads/ref${i}`);
          expect(ethers.getBytes(refs[i].hash)).to.deep.equal(hash);
        }
      });
    });

    describe("Deleting", function () {
      it("can delete", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixture);

        const refName = "refs/heads/main";
        await gitRepository.upsertRef(refName, hash);
        const refsBefore = await gitRepository.listRefs();
        expect(refsBefore.length).to.equal(1);

        await gitRepository.deleteRef(refName);
        const refsAfter = await gitRepository.listRefs();
        expect(refsAfter.length).to.equal(0);
      });

      it("correctly deletes the first one", async function () {
        const { gitRepository, hash, otherHash } = await loadFixture(existingObjectFixture);

        await gitRepository.upsertRef("refs/heads/one", hash);
        await gitRepository.upsertRef("refs/heads/two", otherHash);

        await gitRepository.deleteRef("refs/heads/one");
        const refs = await gitRepository.listRefs();
        expect(refs.length).to.equal(1);
        expect(refs[0].name).to.equal("refs/heads/two");
        expect(ethers.getBytes(refs[0].hash)).to.deep.equal(otherHash);
      });

      it("correctly deletes the last one", async function () {
        const { gitRepository, hash, otherHash } = await loadFixture(existingObjectFixture);

        await gitRepository.upsertRef("refs/heads/one", hash);
        await gitRepository.upsertRef("refs/heads/two", otherHash);

        await gitRepository.deleteRef("refs/heads/two");
        const refs = await gitRepository.listRefs();
        expect(refs.length).to.equal(1);
        expect(refs[0].name).to.equal("refs/heads/one");
        expect(ethers.getBytes(refs[0].hash)).to.deep.equal(hash);
      });

      it("correctly deletes the middle one", async function () {
        const { gitRepository, hash, otherHash } = await loadFixture(existingObjectFixture);

        await gitRepository.upsertRef("refs/heads/one", hash);
        await gitRepository.upsertRef("refs/heads/two", otherHash);
        await gitRepository.upsertRef("refs/heads/three", otherHash);

        await gitRepository.deleteRef("refs/heads/two");
        const refs = await gitRepository.listRefs();
        expect(refs.length).to.equal(2);
        expect(refs[0].name).to.equal("refs/heads/one");
        expect(ethers.getBytes(refs[0].hash)).to.deep.equal(hash);
        expect(refs[1].name).to.equal("refs/heads/three");
        expect(ethers.getBytes(refs[1].hash)).to.deep.equal(otherHash);
      });

      it("can't delete if there are none", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        await expect(gitRepository.deleteRef("refs/heads/main")).to.be.revertedWith("No refs");
      });

      it("can't delete a ref that doesn't exist", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixture);

        await gitRepository.upsertRef("refs/heads/main", hash);

        await expect(gitRepository.deleteRef("refs/heads/other")).to.be.revertedWith("Ref not found");
      });

      it("can't delete if not the owner", async function () {
        const { gitRepository, otherAccount, hash } = await loadFixture(existingObjectFixture);

        const refName = "refs/heads/main";
        await gitRepository.upsertRef(refName, hash);

        await expect(gitRepository.connect(otherAccount).deleteRef(refName))
          .to.be.revertedWithCustomError(gitRepository, "OwnableUnauthorizedAccount")
          .withArgs(otherAccount.address);
      });
    });
  });

  describe("Default branch", function () {
    it("can get", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      const refName = "refs/heads/main";
      expect(await gitRepository.defaultBranchRef()).to.equal(refName);
    });

    it("can change", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      const branchName = "some-branch";
      await gitRepository.setDefaultBranch(branchName);
      const refName = "refs/heads/" + branchName;
      expect(await gitRepository.defaultBranchRef()).to.equal(refName);
    });

    it("can't change to an empty string", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      await expect(gitRepository.setDefaultBranch("")).to.be.revertedWith("Default branch is empty");
    });
  });
});
