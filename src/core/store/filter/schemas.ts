import { z } from 'zod';

/**
 * Schema for a single search criterion.
 * Validates the field key, comparison operator, and the associated value.
 */
export const SearchCriterionSchema = z.object({
    /** Unique identifier for the criterion instance */
    id: z.string().min(1),
    /** The metadata field key (e.g., 'tags', 'rating', 'size', 'color') */
    key: z.string().min(1),
    /** The comparison operator (e.g., 'contains', 'greaterThan', 'similar') */
    operator: z.string().min(1),
    /** The search value: primitive, array (ranges), or object (color criteria) */
    value: z.union([
        z.string(),
        z.number(),
        z.boolean(),
        z.null(),
        z.array(z.union([z.string(), z.number(), z.boolean(), z.null()])),
        z.record(z.string(), z.unknown())
    ]),
    /** Optional scale factor for numeric fields (e.g., byte multipliers) */
    unitMultiplier: z.string().optional(),
    /** UI-friendly description of the criterion */
    displayValue: z.string().optional()
});

/** Interface for a single search criterion */
export interface SearchCriterion {
    id: string;
    key: string;
    operator: string;
    value:
        | string
        | number
        | boolean
        | null
        | (string | number | boolean | null)[]
        | Record<string, unknown>;
    unitMultiplier?: string;
    displayValue?: string;
}

/** Logical operators for search grouping */
export type LogicalOperator = 'and' | 'or';

/** Interface for a search group */
export interface SearchGroup {
    id: string;
    logicalOperator: LogicalOperator;
    items: (SearchCriterion | SearchGroup)[];
}

/**
 * Recursive schema for search groups.
 * Allows logical grouping (AND/OR) of criteria and subgroups.
 */
export const SearchGroupSchema: z.ZodType<SearchGroup> = z.lazy(() =>
    z.object({
        /** Unique identifier for the group */
        id: z.string().min(1),
        /** Logical connector for items within this group */
        logicalOperator: z.enum(['and', 'or']),
        /** List of criteria or nested groups */
        items: z.array(z.union([SearchCriterionSchema, SearchGroupSchema]))
    })
) as z.ZodType<SearchGroup>;

/** Payload type for a search group, derived from the schema */
export type SearchGroupPayload = z.infer<typeof SearchGroupSchema>;
/** Payload type for a search criterion, derived from the schema */
export type SearchCriterionPayload = z.infer<typeof SearchCriterionSchema>;
