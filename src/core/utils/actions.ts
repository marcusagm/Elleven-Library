import { z } from 'zod';
import { fromError } from 'zod-validation-error';
import { ActionResult, ErrorCode } from '../types/actions';

/**
 * Higher-order function to create a validated and secure action.
 * This automates payload validation and error normalization.
 *
 * @param {z.ZodSchema<DataType>} schema - The Zod schema to validate the input.
 * @param {(payload: DataType) => Promise<ActionResult<ResultType>> | ActionResult<ResultType>} actionLogic - The business logic of the action.
 * @returns {Function} A new function that validates the payload before execution.
 */
export function createSecureAction<DataType, ResultType = void>(
    schema: z.ZodSchema<DataType>,
    actionLogic: (payload: DataType) => Promise<ActionResult<ResultType>> | ActionResult<ResultType>
) {
    return async (rawPayload: unknown): Promise<ActionResult<ResultType>> => {
        const validationResult = schema.safeParse(rawPayload);

        if (!validationResult.success) {
            return {
                success: false,
                error: {
                    code: ErrorCode.VALIDATION_ERROR,
                    message: fromError(validationResult.error).message,
                    details: { zodError: validationResult.error.format() }
                }
            };
        }

        try {
            return await actionLogic(validationResult.data);
        } catch (error) {
            return {
                success: false,
                error: {
                    code: ErrorCode.INTERNAL_ERROR,
                    message:
                        error instanceof Error ? error.message : 'An unexpected error occurred',
                    details: { originalError: error }
                }
            };
        }
    };
}
