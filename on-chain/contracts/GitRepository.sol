// SPDX-License-Identifier: Apache-2.0 OR MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/access/Ownable2Step.sol";
import "./external/SHA1.sol";

/// @title Git Repository
/// @author pkulik0
/// @notice This contract is used to store data of a single Git repository.
contract GitRepository is Ownable2Step {
    /// @dev SHA256 are highly recommended due to lower gas costs and better collision resistance.
    bool isSHA256;

    /// @dev The reference to the default branch of the repository.
    string public defaultBranchRef = "refs/heads/main";

    /// @dev keccak256(name) -> padded SHA1 or SHA256 hash
    mapping(bytes32 => bytes32) references;
    /// @dev The names are returned as strings so we can't use bytes32
    string[] referenceNames;
    /// @dev keccak256(name) -> index in referenceNames plus 1
    mapping(bytes32 => uint256) referenceNameToIndex;

    /// @dev Padded SHA1 or SHA256 hash -> object data
    mapping(bytes32 => bytes) objects;

    /// @param _isSHA256 Whether to use SHA256 hashes. Once set, it cannot be changed.
    constructor(bool _isSHA256) Ownable(msg.sender) {
        isSHA256 = _isSHA256;
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
        require(objects[hash].length > 0, "Object not found");
        return objects[hash];
    }

    /// @notice Adds an object to the project.
    /// @param object The object data.
    function addObject(bytes32 hash, bytes memory object) public onlyOwner {
        require(object.length > 0, "Object is empty");
        require(hash != bytes32(0), "Hash is empty");

        bytes32 computedHash = isSHA256 ? sha256(object) : SHA1.sha1(object);
        require(computedHash == hash, "Hash mismatch");

        require(objects[hash].length == 0, "Object already exists");
        objects[hash] = object;
    }

    struct Ref {
        string name;
        bytes32 hash;
    }

    /// @notice Lists all references in the project.
    /// @return An array of Ref structs containing the reference name and hash.
    function listRefs() public view returns (Ref[] memory) {
        uint256 count = referenceNames.length;
        Ref[] memory response = new Ref[](count);
        for (uint256 i = 0; i < count; i++) {
            response[i] = Ref({
                name: referenceNames[i],
                hash: references[keccak256(bytes(referenceNames[i]))]
            });
        }
        return response;
    }

    /// @notice Retrieves a reference by name.
    /// @param name The name of the reference to retrieve.
    /// @return The hash of the reference.
    function getRef(string memory name) public view returns (bytes32) {
        bytes32 key = keccak256(bytes(name));
        require(references[key] != bytes32(0), "Ref not found");
        return references[key];
    }

    /// @notice Validates a reference name.
    /// @param name The name of the reference to validate.
    function validateRefName(string memory name) private pure {
        // TODO: Follow Git ref name rules
        require(bytes(name).length > 0, "Name is invalid");
    }

    /// @notice Upserts a reference.
    /// @param name The name of the reference to upsert.
    /// @param hash The hash of the reference to upsert.
    function upsertRef(string memory name, bytes32 hash) public onlyOwner {
        require(hash != bytes32(0), "Hash is empty");
        require(objects[hash].length > 0, "Object not found");
        validateRefName(name);

        bytes memory nameBytes = bytes(name);
        bytes32 nameKeccak = keccak256(nameBytes);
        references[nameKeccak] = hash;

        if (referenceNameToIndex[nameKeccak] == 0) {
            referenceNames.push(name);
            // offset by 1 to let 0 mean not found
            referenceNameToIndex[nameKeccak] = referenceNames.length;
        }
    }

    /// @notice Deletes a reference.
    /// @param name The name of the reference to delete.
    function deleteRef(string memory name) public onlyOwner {
        require(referenceNames.length > 0, "No refs");

        bytes memory nameBytes = bytes(name);
        bytes32 nameKeccak = keccak256(nameBytes);

        uint256 refIndex = referenceNameToIndex[nameKeccak];
        require(refIndex != 0, "Ref not found");
        refIndex--; // offset by 1 to let 0 mean not found

        delete referenceNameToIndex[nameKeccak];
        delete references[nameKeccak];
        referenceNames[refIndex] = referenceNames[referenceNames.length - 1];
        referenceNames.pop();
    }
}
