import { z } from 'zod';
import { fromError } from 'zod-validation-error';
import { ErrorCode, ActionResult } from './actions';

/**
 * Example schema for testing Sprint 0 infrastructure.
 * This represents a hypothetical payload for creating a tag.
 */
export const CreateTagSchema = z.object({
    name: z.string().min(1, 'Tag name is required').max(50, 'Tag name is too long'),
    color: z.string().regex(/^#[0-9A-Fa-f]{6}$/, 'Invalid color format (hex required)')
});

/**
 * Payload structure for creating a new tag.
 */
export type CreateTagPayload = z.infer<typeof CreateTagSchema>;

/**
 * Validates a payload against the CreateTagSchema and returns a standardized ActionResult.
 *
 * @param {unknown} payload - The raw data to validate.
 * @returns {ActionResult<CreateTagPayload>} A success result with the typed payload or a validation error.
 */
export function validateCreateTag(payload: unknown): ActionResult<CreateTagPayload> {
    const result = CreateTagSchema.safeParse(payload);

    if (!result.success) {
        return {
            success: false,
            error: {
                code: ErrorCode.VALIDATION_ERROR,
                message: fromError(result.error).message,
                details: { zodError: result.error.format() }
            }
        };
    }

    return { success: true, data: result.data };
}
