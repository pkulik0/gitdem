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
    function setDefaultBranch(string calldata newDefaultBranch) public onlyOwner {
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

    /// @dev Represents a git object.
    struct Object {
        bytes32 hash;
        bytes data;
    }

    /// @dev Emitted when an object is added.
    event ObjectAdded(bytes32 hash);

    /// @notice Adds an object to the project.
    /// @param object The object data.
    function addObject(Object calldata object) internal {
        require(object.data.length > 0, "Object is empty");
        require(object.hash != bytes32(0), "Hash is empty");

        bytes32 computedHash = _isSHA256 ? sha256(object.data) : SHA1.sha1(object.data);
        require(computedHash == object.hash, "Hash mismatch");

        require(_objects[object.hash].length == 0, "Object already exists");
        _objects[object.hash] = object.data;
        emit ObjectAdded(object.hash);
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
        string key;
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
            key: "object-format",
            value: _isSHA256 ? "sha256" : "sha1"
        });

        return Refs({
            normal: normal,
            symbolic: symbolic,
            kv: kv
        });
    }

    /// @notice Retrieves references by name.
    /// @param names The names of the references to retrieve.
    /// @return The hashes of the references.
    function resolveRefs(string[] calldata names) public view returns (bytes32[] memory) {
        bytes32[] memory hashes = new bytes32[](names.length);
        for (uint256 i = 0; i < names.length; i++) {
            bytes32 key = keccak256(bytes(names[i]));
            require(_references[key] != bytes32(0), "Ref not found");
            hashes[i] = _references[key];
        }
        return hashes;
    }

    /// @notice Validates a reference name.
    /// @param name The name of the reference to validate.
    function validateRefName(string calldata name) internal pure {
        // TODO: Follow Git ref name rules
        require(bytes(name).length > 0, "Name is invalid");
    }

    /// @dev Emitted when a reference is upserted.
    event RefChanged(string name, bytes32 hash, bytes32 oldHash);

    /// @notice Upserts a reference.
    /// @param ref The reference to upsert.
    function upsertRef(RefNormal calldata ref) internal {
        require(ref.hash != bytes32(0), "Hash is empty");
        require(_objects[ref.hash].length > 0, "Object not found");
        validateRefName(ref.name);

        bytes memory nameBytes = bytes(ref.name);
        bytes32 nameKeccak = keccak256(nameBytes);
        bytes32 oldHash = _references[nameKeccak];
        _references[nameKeccak] = ref.hash;
        emit RefChanged(ref.name, ref.hash, oldHash);

        if (_referenceNameToIndex[nameKeccak] == 0) {
            _referenceNames.push(ref.name);
            // offset by 1 to let 0 mean not found
            _referenceNameToIndex[nameKeccak] = _referenceNames.length;
        }
    }

    /// @notice Deletes a reference.
    /// @param name The name of the reference to delete.
    function deleteRef(string calldata name) internal {
        require(_referenceNames.length > 0, "No refs");

        bytes memory nameBytes = bytes(name);
        bytes32 nameKeccak = keccak256(nameBytes);

        uint256 refIndex = _referenceNameToIndex[nameKeccak];
        require(refIndex != 0, "Ref not found");
        refIndex--; // offset by 1 to let 0 mean not found

        bytes32 oldHash = _references[nameKeccak];
        delete _referenceNameToIndex[nameKeccak];
        delete _references[nameKeccak];
        _referenceNames[refIndex] = _referenceNames[_referenceNames.length - 1];
        _referenceNames.pop();

        emit RefChanged(name, bytes32(0), oldHash);
    }

    /// @dev A struct representing the data to push to the repository.
    struct PushData {
        Object[] objects;
        RefNormal[] refs;
    }

    /// @notice Pushes objects and references to the repository.
    /// @param data The data to push to the repository.
    function pushObjectsAndRefs(PushData calldata data) public onlyOwner {
        require(data.objects.length > 0 || data.refs.length > 0, "No data to push");

        for (uint256 i = 0; i < data.objects.length; i++) {
            addObject(data.objects[i]);
        }

        for (uint256 i = 0; i < data.refs.length; i++) {
            if (data.refs[i].hash != bytes32(0)) {
                upsertRef(data.refs[i]);
            } else {
                deleteRef(data.refs[i].name);
            }
        }
    }
}
