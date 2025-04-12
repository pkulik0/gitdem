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

  async function existingObjectFixtureBase(isSHA256: boolean) {
    const { gitRepository, owner, otherAccount } = await deployGitRepositoryFixtureBase(isSHA256);

    const data = crypto.randomBytes(100);
    const hash = generateHash(isSHA256, data);

    const otherData = crypto.randomBytes(100);
    const otherHash = generateHash(isSHA256, otherData);

    await gitRepository.pushObjectsAndRefs({
      objects: [{ hash, data }, { hash: otherHash, data: otherData }],
      refs: [],
    });

    return { gitRepository, owner, otherAccount, data, hash, otherData, otherHash };
  }

  async function existingObjectFixtureSHA1() {
    return existingObjectFixtureBase(false);
  }

  async function existingObjectFixture() {
    return existingObjectFixtureBase(true);
  }

  async function existingRefFixtureBase(isSHA256: boolean) {
    const { gitRepository, owner, otherAccount } = await deployGitRepositoryFixtureBase(isSHA256);

    const data = crypto.randomBytes(100);
    const hash = generateHash(isSHA256, data);

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
    describe("Getting", function () {
      it("can get", async function () {
        const { gitRepository, data, hash } = await loadFixture(existingObjectFixture);

        const object = await gitRepository.getObject(hash);
        expect(ethers.getBytes(object)).to.deep.equal(data);
      });

      it("can't get if no objects exist", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const hash = generateHash(true);
        await expect(gitRepository.getObject(hash)).to.be.revertedWith("Object not found");
      });

      it("can't get if the object doesn't exist", async function () {
        const { gitRepository } = await loadFixture(existingObjectFixture);

        const otherHash = generateHash(true);
        await expect(gitRepository.getObject(otherHash)).to.be.revertedWith("Object not found");
      });

      it("anyone can get an object", async function () {
        const { gitRepository, otherAccount, data, hash } = await loadFixture(existingObjectFixture);

        const object = await gitRepository.connect(otherAccount).getObject(hash);
        expect(ethers.getBytes(object)).to.deep.equal(data);
      });
    });
  });

  describe("Refs", function () {
    describe("Getting", function () {
      it("can get", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const data = crypto.randomBytes(100);
        const hash = generateHash(true, data);

        await gitRepository.pushObjectsAndRefs({
          objects: [{ hash, data }],
          refs: [{
            name: "refs/heads/main",
            hash: hash,
          }],
        })

        const hashes = await gitRepository.resolveRefs(["refs/heads/main"]);
        expect(ethers.getBytes(hashes[0])).to.deep.equal(hash);
      });

      it("non-existent returns empty hash", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const hashes = await gitRepository.resolveRefs(["refs/heads/main"]);
        expect(ethers.getBytes(hashes[0])).to.deep.equal(new Uint8Array(32));
      });
    });

    describe("Listing", function () {
      it("has HEAD symbolic ref", async function () {
        const { gitRepository } = await loadFixture(existingObjectFixture);

        const refs = await gitRepository.listRefs();

        expect(refs.symbolic.length).to.equal(1);
        expect(refs.symbolic[0].name).to.equal("HEAD");
        expect(refs.symbolic[0].target).to.equal("refs/heads/main");
      });

      it("has object-format kv ref", async function () {
        const { gitRepository } = await loadFixture(existingObjectFixture);
        const refs = await gitRepository.listRefs();
        expect(refs.kv.length).to.equal(1);
        expect(refs.kv[0].key).to.equal("object-format");
        expect(refs.kv[0].value).to.equal("sha256");

        const { gitRepository: gitRepositorySHA1 } = await loadFixture(existingObjectFixtureSHA1);
        const refsSHA1 = await gitRepositorySHA1.listRefs();
        expect(refsSHA1.kv.length).to.equal(1);
        expect(refsSHA1.kv[0].key).to.equal("object-format");
        expect(refsSHA1.kv[0].value).to.equal("sha1");
      });

      it("can list a single ref", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixture);

        await gitRepository.pushObjectsAndRefs({
          objects: [],
          refs: [{
            name: "refs/heads/main",
            hash: hash,
          }],
        });

        const refs = await gitRepository.listRefs();

        expect(refs.normal.length).to.equal(1);
        expect(refs.normal[0].name).to.equal("refs/heads/main");
        expect(ethers.getBytes(refs.normal[0].hash)).to.deep.equal(hash);

        expect(refs.symbolic.length).to.equal(1);
        expect(refs.symbolic[0].name).to.equal("HEAD");
        expect(refs.symbolic[0].target).to.equal("refs/heads/main");
      });

      it("can list multiple refs", async function () {
        const { gitRepository, hash, otherHash } = await loadFixture(existingObjectFixture);

        await gitRepository.pushObjectsAndRefs({
          objects: [],
          refs: [{
            name: "refs/heads/main",
            hash: hash,
          }, {
            name: "refs/heads/other",
            hash: otherHash,
          }],
        });

        const refs = await gitRepository.listRefs();

        expect(refs.normal.length).to.equal(2);
        expect(refs.normal[0].name).to.equal("refs/heads/main");
        expect(ethers.getBytes(refs.normal[0].hash)).to.deep.equal(hash);
        expect(refs.normal[1].name).to.equal("refs/heads/other");
        expect(ethers.getBytes(refs.normal[1].hash)).to.deep.equal(otherHash);
      });

      it("can list with no refs", async function () {
        const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

        const refs = await gitRepository.listRefs();

        expect(refs.normal.length).to.equal(0);
      });

      it("can list with SHA1 hash type", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixtureSHA1);

        await gitRepository.pushObjectsAndRefs({
          objects: [],
          refs: [{
            name: "refs/heads/main",
            hash: hash,
          }],
        });

        const refs = await gitRepository.listRefs();

        expect(refs.normal.length).to.equal(1);
        expect(refs.normal[0].name).to.equal("refs/heads/main");
        expect(ethers.getBytes(refs.normal[0].hash)).to.deep.equal(hash);
      });

      it("anyone can list refs", async function () {
        const { gitRepository, otherAccount, hash } = await loadFixture(
          existingObjectFixture
        );

        await gitRepository.pushObjectsAndRefs({
          objects: [],
          refs: [{
            name: "refs/heads/main",
            hash: hash,
          }],
        });

        const refs = await gitRepository.connect(otherAccount).listRefs();

        expect(refs.normal.length).to.equal(1);
        expect(refs.normal[0].name).to.equal("refs/heads/main");
        expect(ethers.getBytes(refs.normal[0].hash)).to.deep.equal(hash);
      });

      it("can handle a large number of refs", async function () {
        const { gitRepository, hash } = await loadFixture(existingObjectFixture);

        const count = 256;
        const refsToPush = [];
        for (let i = 0; i < count; i++) {
          refsToPush.push({
            name: `refs/heads/ref${i}`,
            hash: hash,
          });
        }
        await gitRepository.pushObjectsAndRefs({
          objects: [],
          refs: refsToPush,
        });

        const refs = await gitRepository.listRefs();

        expect(refs.normal.length).to.equal(count);
        for (let i = 0; i < count; i++) {
          expect(refs.normal[i].name).to.equal(`refs/heads/ref${i}`);
          expect(ethers.getBytes(refs.normal[i].hash)).to.deep.equal(hash);
        }
      });
    });
  });

  describe("Pushing", function () {
    it("can push with a SHA256 hash", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      const data = crypto.randomBytes(100);
      const hash = generateHash(true, data);

      await gitRepository.pushObjectsAndRefs({
        objects: [{ hash, data }],
        refs: [{
          name: "refs/heads/main",
          hash: hash,
        }],
      })

      const objectHashes = await gitRepository.getObjectHashes();
      expect(objectHashes.length).to.equal(1);
      expect(ethers.getBytes(objectHashes[0])).to.deep.equal(hash);
    });

    it("can push with a SHA1 hash", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixtureSHA1);

      const data = crypto.randomBytes(100);
      const hash = generateHash(false, data);

      await gitRepository.pushObjectsAndRefs({
        objects: [{ hash, data }],
        refs: [{
          name: "refs/heads/main",
          hash: hash,
        }],
      })

      const objectHashes = await gitRepository.getObjectHashes();
      expect(objectHashes.length).to.equal(1);
      expect(ethers.getBytes(objectHashes[0])).to.deep.equal(hash);
    });

    it("can't push with no data", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      await expect(gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [],
      })).to.be.revertedWith("No data to push");
    });

    it("can't push an empty object", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      const data = new Uint8Array(0);
      const hash = generateHash(true, data);

      await expect(gitRepository.pushObjectsAndRefs({
        objects: [{ hash, data }],
        refs: [{
          name: "refs/heads/main",
          hash: hash,
        }],
      })).to.be.revertedWith("Object is empty");
    });

    it("can't push with non-existent hash", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      const hash = generateHash(true);
      await expect(gitRepository.pushObjectsAndRefs({
        objects: [], // No objects
        refs: [{
          name: "refs/heads/main",
          hash: hash,
        }],
      })).to.be.revertedWith("Object not found");
    });

    it("can't push if not the owner", async function () {
      const { gitRepository, otherAccount } = await loadFixture(deployGitRepositoryFixture);

      const data = crypto.randomBytes(100);
      const hash = generateHash(true, data);
      await expect(gitRepository.connect(otherAccount).pushObjectsAndRefs({
        objects: [{ hash, data }],
        refs: [{
          name: "refs/heads/main",
          hash: hash,
        }],
      })).to.be.revertedWithCustomError(gitRepository, "OwnableUnauthorizedAccount")
        .withArgs(otherAccount.address);
    });

    it("can't push a ref with an invalid name", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      const data = crypto.randomBytes(100);
      const hash = generateHash(true, data);

      await expect(gitRepository.pushObjectsAndRefs({
        objects: [{ hash, data }],
        refs: [{
          name: "",
          hash: hash,
        }],
      })).to.be.revertedWith("Name is invalid");
    });

    it("can update a ref with a new hash", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      const data = crypto.randomBytes(100);
      const hash = generateHash(true, data);

      await gitRepository.pushObjectsAndRefs({
        objects: [{ hash, data }],
        refs: [{
          name: "refs/heads/main",
          hash: hash,
        }],
      });
      const hashes = await gitRepository.resolveRefs(["refs/heads/main"]);
      expect(ethers.getBytes(hashes[0])).to.deep.equal(hash);

      const otherData = crypto.randomBytes(100);
      const otherHash = generateHash(true, otherData);

      await gitRepository.pushObjectsAndRefs({
        objects: [{ hash: otherHash, data: otherData }],
        refs: [{
          name: "refs/heads/main",
          hash: otherHash,
        }],
      });
      const newHashes = await gitRepository.resolveRefs(["refs/heads/main"]);
      expect(ethers.getBytes(newHashes[0])).to.deep.equal(otherHash);
    });

    it("can delete a ref", async function () {
      const { gitRepository, hash } = await loadFixture(existingObjectFixture);

      const refName = "refs/heads/main";
      await gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: refName,
          hash: hash,
        }],
      });
      const refsBefore = await gitRepository.listRefs();

      expect(refsBefore.normal.length).to.equal(1);
      expect(refsBefore.normal[0].name).to.equal(refName);
      expect(ethers.getBytes(refsBefore.normal[0].hash)).to.deep.equal(hash);

      await gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: refName,
          hash: new Uint8Array(32),
        }],
      });
      const refsAfter = await gitRepository.listRefs();

      expect(refsAfter.normal.length).to.equal(0);
    });

    it("correctly deletes the first ref", async function () {
      const { gitRepository, hash, otherHash } = await loadFixture(existingObjectFixture);

      await gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: "refs/heads/one",
          hash: hash,
        }, {
          name: "refs/heads/two",
          hash: otherHash,
        }],
      });

      await gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: "refs/heads/one",
          hash: new Uint8Array(32),
        }],
      });
      const refs = await gitRepository.listRefs();

      expect(refs.normal.length).to.equal(1);
      expect(refs.normal[0].name).to.equal("refs/heads/two");
      expect(ethers.getBytes(refs.normal[0].hash)).to.deep.equal(otherHash);
    });

    it("correctly deletes the last ref", async function () {
      const { gitRepository, hash, otherHash } = await loadFixture(existingObjectFixture);

      await gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: "refs/heads/one",
          hash: hash,
        }, {
          name: "refs/heads/two",
          hash: otherHash,
        }],
      });

      await gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: "refs/heads/two",
          hash: new Uint8Array(32),
        }],
      });
      const refs = await gitRepository.listRefs();

      expect(refs.normal.length).to.equal(1);
      expect(refs.normal[0].name).to.equal("refs/heads/one");
      expect(ethers.getBytes(refs.normal[0].hash)).to.deep.equal(hash);
    });

    it("correctly deletes a ref from the middle", async function () {
      const { gitRepository, hash, otherHash } = await loadFixture(existingObjectFixture);

      await gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: "refs/heads/one",
          hash: hash,
        }, {
          name: "refs/heads/two",
          hash: otherHash,
        }, {
          name: "refs/heads/three",
          hash: otherHash,
        }],
      });

      await gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: "refs/heads/two",
          hash: new Uint8Array(32),
        }],
      });
      const refs = await gitRepository.listRefs();

      expect(refs.normal.length).to.equal(2);
      expect(refs.normal[0].name).to.equal("refs/heads/one");
      expect(ethers.getBytes(refs.normal[0].hash)).to.deep.equal(hash);
      expect(refs.normal[1].name).to.equal("refs/heads/three");
      expect(ethers.getBytes(refs.normal[1].hash)).to.deep.equal(otherHash);
    });

    it("can't delete a ref if there are none", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      await expect(gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: "refs/heads/some-ref",
          hash: new Uint8Array(32),
        }],
      })).to.be.revertedWith("No refs");
    });

    it("can't delete a ref that doesn't exist", async function () {
      const { gitRepository, hash } = await loadFixture(existingObjectFixture);

      await gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: "refs/heads/some-ref",
          hash: hash,
        }],
      });

      await expect(gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{
          name: "refs/heads/other",
          hash: new Uint8Array(32),
        }],
      })).to.be.revertedWith("Ref not found");
    });

    it("emits events", async function () {
      const { gitRepository } = await loadFixture(deployGitRepositoryFixture);

      const data = crypto.randomBytes(100);
      const hash = generateHash(true, data);

      await expect(gitRepository.pushObjectsAndRefs({
        objects: [{ hash, data }],
        refs: [{ name: "refs/heads/main", hash }],
      })).to.emit(gitRepository, "RefChanged").withArgs("refs/heads/main", hash, new Uint8Array(32))
        .and.emit(gitRepository, "ObjectAdded").withArgs(hash);

      await expect(gitRepository.pushObjectsAndRefs({
        objects: [],
        refs: [{ name: "refs/heads/main", hash: new Uint8Array(32) }],
      })).to.emit(gitRepository, "RefChanged").withArgs("refs/heads/main", new Uint8Array(32), hash);
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
