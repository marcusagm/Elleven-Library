/* eslint-disable */
import { validateCreateTag } from '../src/core/types/test-schema';

console.log('--- Testing Zod Validation Infrastructure ---');

// Test Case 1: Valid Payload
const validPayload = { name: 'Work', color: '#FF0000' };
const validResult = validateCreateTag(validPayload);
console.log('\nValid Payload Result:', JSON.stringify(validResult, null, 2));

// Test Case 2: Invalid Payload (Missing color)
const missingColorPayload = { name: 'Work' };
const missingColorResult = validateCreateTag(missingColorPayload);
console.log(
    '\nInvalid Payload (missing color) Result:',
    JSON.stringify(missingColorResult, null, 2)
);

// Test Case 3: Invalid Payload (Name too short)
const emptyNamePayload = { name: '', color: '#123456' };
const emptyNameResult = validateCreateTag(emptyNamePayload);
console.log(
    '\nInvalid Payload (empty name) Result:',
    JSON.stringify(emptyNameResult, null, 2)
);

console.log('\n--- Test Finished ---');
