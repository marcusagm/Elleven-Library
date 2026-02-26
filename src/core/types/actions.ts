/**
 * Standard codes for errors across the application domain.
 * This ensures that the UI can react to specific error types consistently.
 */
export enum ErrorCode {
    VALIDATION_ERROR = 'VALIDATION_ERROR',
    IO_ERROR = 'IO_ERROR',
    UNAUTHORIZED = 'UNAUTHORIZED',
    NOT_FOUND = 'NOT_FOUND',
    CONFLICT = 'CONFLICT',
    INTERNAL_ERROR = 'INTERNAL_ERROR',
    UNKNOWN_ERROR = 'UNKNOWN_ERROR'
}

/**
 * Base error structure for all actions.
 */
export interface BaseError {
    /** The error code identifying the type of error. */
    code: ErrorCode | string;
    /** Human-readable message describing the error. */
    message: string;
    /** Additional context or machine-readable error data. */
    details?: Record<string, unknown>;
}

/**
 * Standardized result for all actions in the core layer.
 * This pattern avoids throwing exceptions for expected business errors
 * and forces the consumer to handle both success and failure states.
 */
export type ActionResult<DataType = void, ErrorType = BaseError> =
    | { success: true; data: DataType }
    | { success: false; error: ErrorType };
