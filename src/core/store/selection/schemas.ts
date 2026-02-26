import { z } from 'zod';

/**
 * Schema for selection payload.
 * Used for validation when persisting or transferring selection data.
 */
export const SelectionPayloadSchema = z.object({
    selected_ids: z.array(z.number())
});

/**
 * Type derived from the SelectionPayloadSchema.
 */
export type SelectionPayload = z.infer<typeof SelectionPayloadSchema>;
