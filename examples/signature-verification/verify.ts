/**
 * ORBS TEE Nitro - Signature Verification (TypeScript)
 *
 * This example demonstrates how to verify signatures from ORBS TEE Nitro enclaves
 * using TypeScript/Node.js.
 *
 * Dependencies:
 * - secp256k1 (npm install secp256k1 @types/secp256k1)
 * - crypto (built-in Node.js module)
 *
 * Installation:
 * npm install secp256k1 @types/secp256k1
 *
 * Usage:
 * ts-node verify.ts
 * or compile first: tsc verify.ts && node verify.js
 */

import * as secp256k1 from 'secp256k1';
import * as crypto from 'crypto';

/**
 * Canonicalize JSON for deterministic serialization
 *
 * IMPORTANT: This must match the canonicalization used by the enclave!
 * The enclave sorts keys alphabetically and uses compact format.
 *
 * @param value - The JSON value to canonicalize
 * @returns Canonical JSON string
 */
export function canonicalizeJSON(value: any): string {
    function sortValue(val: any): any {
        if (val === null || typeof val !== 'object') {
            return val;
        }

        if (Array.isArray(val)) {
            return val.map(sortValue);
        }

        // Sort object keys alphabetically
        const sorted: Record<string, any> = {};
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
 * @param data - The original data that was signed
 * @param signatureHex - The signature as a hex string (with or without 0x prefix)
 * @param publicKeyHex - The public key as a hex string (with or without 0x prefix)
 * @returns true if signature is valid, false otherwise
 */
export function verifySignature(
    data: Buffer | Uint8Array,
    signatureHex: string,
    publicKeyHex: string
): boolean {
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
        console.error('Error verifying signature:', (error as Error).message);
        return false;
    }
}

/**
 * Verify a signed JSON response from the enclave
 *
 * This is the main function you'll use to verify enclave responses.
 *
 * @param jsonData - The JSON data from the response
 * @param signatureHex - The signature from the response
 * @param publicKeyHex - The enclave's public key
 * @returns true if signature is valid, false otherwise
 */
export function verifyJSONSignature(
    jsonData: any,
    signatureHex: string,
    publicKeyHex: string
): boolean {
    try {
        // Canonicalize JSON (same way the enclave does it)
        const canonicalJSON = canonicalizeJSON(jsonData);
        const dataBuffer = Buffer.from(canonicalJSON, 'utf8');

        // Verify the signature
        return verifySignature(dataBuffer, signatureHex, publicKeyHex);
    } catch (error) {
        console.error('Error verifying JSON signature:', (error as Error).message);
        return false;
    }
}

/**
 * Type definitions for TEE protocol responses
 */
export interface TeeResponse {
    id: string;
    success: boolean;
    data?: any;
    signature?: string;
    error?: string;
}

/**
 * Verify a complete TEE response object
 *
 * @param response - The full TEE response
 * @param publicKeyHex - The enclave's public key
 * @returns true if response has valid signature, false otherwise
 */
export function verifyTeeResponse(
    response: TeeResponse,
    publicKeyHex: string
): boolean {
    if (!response.success) {
        // Error responses don't have signatures
        return true; // Not a signature error, just an application error
    }

    if (!response.data || !response.signature) {
        console.warn('Response missing data or signature');
        return false;
    }

    return verifyJSONSignature(response.data, response.signature, publicKeyHex);
}

// ============================================================================
// Example Usage
// ============================================================================

function main() {
    console.log('ORBS TEE Nitro - Signature Verification Example (TypeScript)\n');

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

    // Example 3: Verify a complete TEE response
    console.log('\n\nExample 3: Verifying a complete TEE response');
    console.log('============================================');

    const teeResponse: TeeResponse = {
        id: 'req-123',
        success: true,
        data: {
            symbol: 'ETHUSDT',
            price: '2500.00'
        },
        signature: 'abcd1234...'
    };

    console.log('TEE Response:', JSON.stringify(teeResponse, null, 2));

    const valid3 = verifyTeeResponse(teeResponse, enclavePublicKey);
    console.log('\nTEE Response signature valid:', valid3, '(expected false - using example data)');

    console.log('\n\nHow to use this in your application:');
    console.log('====================================');
    console.log('1. Get the enclave\'s public key from the attestation document');
    console.log('2. Send requests to the enclave via vsocket');
    console.log('3. For each signed response:');
    console.log('   - Parse the response as TeeResponse');
    console.log('   - Call verifyTeeResponse(response, publicKey)');
    console.log('   - Only trust the response if verification succeeds');
    console.log('\nExample code:');
    console.log('-------------');
    console.log(`
    // Parse the response
    const response: TeeResponse = JSON.parse(responseBytes.toString());

    // Verify the signature
    const valid = verifyTeeResponse(response, enclavePublicKey);

    if (!valid) {
        throw new Error('Invalid signature - response may have been tampered with');
    }

    // Signature is valid - safe to use the data
    console.log('Data:', response.data);
    `);
}

// Run the example if executed directly
if (require.main === module) {
    main();
}
