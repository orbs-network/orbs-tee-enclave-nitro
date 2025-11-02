/**
 * ORBS TEE Nitro - Signature Verification (JavaScript/Node.js)
 *
 * This example demonstrates how to verify signatures from ORBS TEE Nitro enclaves
 * using Node.js or browser JavaScript.
 *
 * Dependencies:
 * - secp256k1 (npm install secp256k1)
 * - crypto (built-in Node.js module, or use WebCrypto in browsers)
 *
 * Installation:
 * npm install secp256k1
 *
 * Usage:
 * node verify.js
 */

const secp256k1 = require('secp256k1');
const crypto = require('crypto');

/**
 * Canonicalize JSON for deterministic serialization
 *
 * IMPORTANT: This must match the canonicalization used by the enclave!
 * The enclave sorts keys alphabetically and uses compact format.
 *
 * @param {*} value - The JSON value to canonicalize
 * @returns {string} - Canonical JSON string
 */
function canonicalizeJSON(value) {
    function sortValue(val) {
        if (val === null || typeof val !== 'object') {
            return val;
        }

        if (Array.isArray(val)) {
            return val.map(sortValue);
        }

        // Sort object keys alphabetically
        const sorted = {};
        Object.keys(val).sort().forEach(key => {
            sorted[key] = sortValue(val[key]);
        });
        return sorted;
    }

    const sorted = sortValue(value);
    // Serialize to compact JSON (no whitespace)
    return JSON.stringify(sorted);
}

/**
 * Verify an ECDSA signature
 *
 * @param {Buffer|Uint8Array} data - The original data that was signed
 * @param {string} signatureHex - The signature as a hex string (with or without 0x prefix)
 * @param {string} publicKeyHex - The public key as a hex string (with or without 0x prefix)
 * @returns {boolean} - true if signature is valid, false otherwise
 */
function verifySignature(data, signatureHex, publicKeyHex) {
    try {
        // Remove 0x prefix if present
        signatureHex = signatureHex.replace(/^0x/, '');
        publicKeyHex = publicKeyHex.replace(/^0x/, '');

        // Decode hex strings to buffers
        const signature = Buffer.from(signatureHex, 'hex');
        const publicKey = Buffer.from(publicKeyHex, 'hex');

        // Validate signature length (64 bytes: 32 bytes r + 32 bytes s)
        if (signature.length !== 64) {
            throw new Error(`Invalid signature length: expected 64 bytes, got ${signature.length}`);
        }

        // Hash the data with SHA-256
        const hash = crypto.createHash('sha256').update(data).digest();

        // Verify the signature
        return secp256k1.ecdsaVerify(signature, hash, publicKey);
    } catch (error) {
        console.error('Error verifying signature:', error.message);
        return false;
    }
}

/**
 * Verify a signed JSON response from the enclave
 *
 * This is the main function you'll use to verify enclave responses.
 *
 * @param {object} jsonData - The JSON data from the response
 * @param {string} signatureHex - The signature from the response
 * @param {string} publicKeyHex - The enclave's public key
 * @returns {boolean} - true if signature is valid, false otherwise
 */
function verifyJSONSignature(jsonData, signatureHex, publicKeyHex) {
    try {
        // Canonicalize JSON (same way the enclave does it)
        const canonicalJSON = canonicalizeJSON(jsonData);
        const dataBuffer = Buffer.from(canonicalJSON, 'utf8');

        // Verify the signature
        return verifySignature(dataBuffer, signatureHex, publicKeyHex);
    } catch (error) {
        console.error('Error verifying JSON signature:', error.message);
        return false;
    }
}

// ============================================================================
// Example Usage
// ============================================================================

function main() {
    console.log('ORBS TEE Nitro - Signature Verification Example (JavaScript)\n');

    // Example 1: Verify a simple message signature
    console.log('Example 1: Verifying a simple message');
    console.log('======================================');

    const message = Buffer.from('Hello, Enclave!', 'utf8');
    const signature = '3045022100abcd...'; // Example signature (replace with real one)
    const publicKey = '0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798';

    console.log('Message:', message.toString());
    console.log('Signature:', signature);
    console.log('Public Key:', publicKey);

    // Note: This will fail because it's an example signature
    const valid1 = verifySignature(message, signature, publicKey);
    console.log('Signature valid:', valid1, '(expected false - using example data)\n');

    // Example 2: Verify a JSON response from the enclave
    console.log('Example 2: Verifying a JSON response');
    console.log('====================================');

    // Simulate a response from the enclave
    const jsonResponse = {
        symbol: 'BTCUSDT',
        price: '42000.50',
        timestamp: 1234567890
    };

    const responseSignature = 'abcd1234...'; // Replace with real signature from enclave
    const enclavePublicKey = publicKey; // Same public key as above

    console.log('Response data:', JSON.stringify(jsonResponse, null, 2));
    console.log('Response signature:', responseSignature);
    console.log('Enclave public key:', enclavePublicKey);

    const valid2 = verifyJSONSignature(jsonResponse, responseSignature, enclavePublicKey);
    console.log('\nSignature valid:', valid2, '(expected false - using example data)');

    console.log('\n\nHow to use this in your application:');
    console.log('====================================');
    console.log('1. Get the enclave\'s public key from the attestation document');
    console.log('2. Send requests to the enclave via vsocket');
    console.log('3. For each signed response:');
    console.log('   - Extract the \'data\' and \'signature\' fields');
    console.log('   - Call verifyJSONSignature(data, signature, publicKey)');
    console.log('   - Only trust the response if verification succeeds');
    console.log('\nExample code:');
    console.log('-------------');
    console.log(`
    // Parse the response
    const response = JSON.parse(responseBytes);

    // Verify the signature if present
    if (response.data && response.signature) {
        const valid = verifyJSONSignature(
            response.data,
            response.signature,
            enclavePublicKey
        );

        if (!valid) {
            throw new Error('Invalid signature - response may have been tampered with');
        }

        // Signature is valid - safe to use the data
        console.log('Data:', response.data);
    }
    `);
}

// Run the example if executed directly
if (require.main === module) {
    main();
}

// Export functions for use as a library
module.exports = {
    canonicalizeJSON,
    verifySignature,
    verifyJSONSignature
};
