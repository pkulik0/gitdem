// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "./external/SHA1.sol";

/// @title Git Repository
/// @author pkulik0
/// @notice This contract is used to store data of a single Git repository.
contract GitRepository is Ownable2Step {
    /// @dev The reference to the default branch of the repository.
    string public defaultBranchRef = "refs/heads/main";
    /// @dev SHA256 is highly recommended due to lower gas costs and better collision resistance.
    bool _isSHA256;

    /// @dev keccak256(name) -> padded SHA1 or SHA256 hash
    mapping(bytes32 => bytes32) _references;
    /// @dev The names are returned as strings so we can't use bytes32
    string[] _referenceNames;
    /// @dev keccak256(name) -> index in _referenceNames plus 1
    mapping(bytes32 => uint256) _referenceNameToIndex;

    /// @dev Padded SHA1 or SHA256 hash -> object data
    mapping(bytes32 => bytes) _objects;

    /// @param isSHA256 Whether to use SHA256 hashes. Once set, it cannot be changed.
    constructor(bool isSHA256) Ownable(msg.sender) {
        _isSHA256 = isSHA256;
    }

    /// @notice Sets the default branch of the repository.
    /// @param newDefaultBranch The name of the new default branch.
    function setDefaultBranch(string memory newDefaultBranch) public onlyOwner {
        require(bytes(newDefaultBranch).length > 0, "Default branch is empty");
        validateRefName(newDefaultBranch);
        defaultBranchRef = string.concat("refs/heads/", newDefaultBranch);
    }

    /// @notice Retrieves an object by its hash.
    /// @param hash The hash of the object to retrieve.
    /// @return The object data.
    function getObject(bytes32 hash) public view returns (bytes memory) {
        require(_objects[hash].length > 0, "Object not found");
        return _objects[hash];
    }

    /// @notice Adds an object to the project.
    /// @param object The object data.
    function addObject(bytes32 hash, bytes memory object) public onlyOwner {
        require(object.length > 0, "Object is empty");
        require(hash != bytes32(0), "Hash is empty");

        bytes32 computedHash = _isSHA256 ? sha256(object) : SHA1.sha1(object);
        require(computedHash == hash, "Hash mismatch");

        require(_objects[hash].length == 0, "Object already exists");
        _objects[hash] = object;
    }

    /// @dev A struct representing a normal reference.
    struct RefNormal {
        string name;
        bytes32 hash;
    }

    /// @dev A struct representing a symbolic reference.
    struct RefSymbolic {
        string name;
        string target;
    }

    /// @dev A struct representing a key-value reference.
    struct RefKV {
        string name;
        string value;
    }

    /// @dev A struct containing arrays of all reference types.
    struct Refs {
        RefNormal[] normal;
        RefSymbolic[] symbolic;
        RefKV[] kv;
    }

    /// @notice Lists all references in the project.
    /// @return A struct containing all direct and symbolic references.
    function listRefs() public view returns (Refs memory) {
        uint256 count = _referenceNames.length;
        RefNormal[] memory normal = new RefNormal[](count);
        for (uint256 i = 0; i < count; i++) {
            normal[i] = RefNormal({
                name: _referenceNames[i],
                hash: _references[keccak256(bytes(_referenceNames[i]))]
            });
        }

        RefSymbolic[] memory symbolic = new RefSymbolic[](1);
        symbolic[0] = RefSymbolic({
            name: "HEAD",
            target: defaultBranchRef
        });

        RefKV[] memory kv = new RefKV[](1);
        kv[0] = RefKV({
            name: "object-format",
            value: _isSHA256 ? "sha256" : "sha1"
        });

        return Refs({
            normal: normal,
            symbolic: symbolic,
            kv: kv
        });
    }

    /// @notice Retrieves a reference by name.
    /// @param name The name of the reference to retrieve.
    /// @return The hash of the reference.
    function getRef(string memory name) public view returns (bytes32) {
        bytes32 key = keccak256(bytes(name));
        require(_references[key] != bytes32(0), "Ref not found");
        return _references[key];
    }

    /// @notice Validates a reference name.
    /// @param name The name of the reference to validate.
    function validateRefName(string memory name) internal pure {
        // TODO: Follow Git ref name rules
        require(bytes(name).length > 0, "Name is invalid");
    }

    /// @notice Upserts a reference.
    /// @param name The name of the reference to upsert.
    /// @param hash The hash of the reference to upsert.
    function upsertRef(string memory name, bytes32 hash) public onlyOwner {
        require(hash != bytes32(0), "Hash is empty");
        require(_objects[hash].length > 0, "Object not found");
        validateRefName(name);

        bytes memory nameBytes = bytes(name);
        bytes32 nameKeccak = keccak256(nameBytes);
        _references[nameKeccak] = hash;

        if (_referenceNameToIndex[nameKeccak] == 0) {
            _referenceNames.push(name);
            // offset by 1 to let 0 mean not found
            _referenceNameToIndex[nameKeccak] = _referenceNames.length;
        }
    }

    /// @notice Deletes a reference.
    /// @param name The name of the reference to delete.
    function deleteRef(string memory name) public onlyOwner {
        require(_referenceNames.length > 0, "No refs");

        bytes memory nameBytes = bytes(name);
        bytes32 nameKeccak = keccak256(nameBytes);

        uint256 refIndex = _referenceNameToIndex[nameKeccak];
        require(refIndex != 0, "Ref not found");
        refIndex--; // offset by 1 to let 0 mean not found

        delete _referenceNameToIndex[nameKeccak];
        delete _references[nameKeccak];
        _referenceNames[refIndex] = _referenceNames[_referenceNames.length - 1];
        _referenceNames.pop();
    }
}
