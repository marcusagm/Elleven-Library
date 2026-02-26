import { describe, it, expect } from 'vitest';
import { validateCreateTag } from './test-schema';
import { ErrorCode } from './actions';

describe('Sprint 0: Zod Validation Infrastructure', () => {
    it('should validate a correct payload', () => {
        const payload = { name: 'Work', color: '#FF0000' };
        const result = validateCreateTag(payload);

        expect(result.success).toBe(true);
        if (result.success) {
            expect(result.data).toEqual(payload);
        }
    });

    it('should return a VALIDATION_ERROR for missing fields', () => {
        const payload = { name: 'Work' };
        const result = validateCreateTag(payload);

        expect(result.success).toBe(false);
        if (!result.success) {
            expect(result.error.code).toBe(ErrorCode.VALIDATION_ERROR);
            expect(result.error.message).toContain('color');
        }
    });

    it('should return a VALIDATION_ERROR for invalid field formats', () => {
        const payload = { name: 'Work', color: 'red' };
        const result = validateCreateTag(payload);

        expect(result.success).toBe(false);
        if (!result.success) {
            expect(result.error.code).toBe(ErrorCode.VALIDATION_ERROR);
            expect(result.error.message).toContain('Invalid color format');
        }
    });
});
