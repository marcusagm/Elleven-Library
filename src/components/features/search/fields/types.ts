import { Component } from 'solid-js';
import { SearchValue } from '../useAdvancedSearch';

/**
 * Properties defining the state and mutators for an individual search criterion field renderer.
 * These properties are passed to specialized components that handle specific data types (e.g., date, size, tags).
 */
export interface CriterionFieldRendererProperties {
    /** The actual search key or database field being evaluated (e.g., 'size', 'added_at', 'filename'). */
    fieldKey: string;

    /** The logic used to compare values, such as 'eq' (equals), 'between', or 'contains'. */
    comparisonOperator: string;

    /** The primary search value currently assigned to this criterion. */
    value: SearchValue;

    /**
     * Callback function used to update the primary search value in the underlying state.
     * @param value - The new search value to be applied.
     */
    setValue: (value: SearchValue) => void;

    /** The secondary search value, typically used for range-based logic like the 'between' operator. */
    value2?: SearchValue;

    /**
     * Callback function used to update the secondary search value.
     * @param value - The new secondary search value to be applied.
     */
    setValue2?: (value: SearchValue) => void;

    /** Additional numeric metric or unit configuration, most common in file size calculations. */
    unitMultiplier?: string;

    /**
     * Callback function used to update the selected unit or multiplier.
     * @param unit - The unit string identifier (e.g., 'MB', 'GB').
     */
    setUnitMultiplier?: (unit: string) => void;

    /** Map of active validation error messages, keyed by field part (e.g., 'value', 'value2'). */
    errors: Record<string, string>;

    /** The preferred visual scale of the input controls. Defaults to 'md'. */
    size?: 'sm' | 'md';
}

/**
 * Type representing a Solid.js component used for rendering a search criterion input.
 */
export type CriterionFieldRendererComponent = Component<CriterionFieldRendererProperties>;

/**
 * Metadata provided by the core stores to the search field handlers for rendering options or performing lookups.
 */
export type StoreMetadata = {
    /** List of available storage locations or folders. */
    locations: { id: number; name: string }[];

    /** List of globally defined tags. */
    tags: { id: number; name: string }[];

    /** Optional list of file formats supported by the current system environment. */
    supportedFormats?: { name: string; extensions: string[] }[];
};

/**
 * interface defining the logic for validating, processing, and formatting a specific search field type.
 */
export interface SearchFieldHandler {
    /** The visual component used to render the inputs for this field type. */
    component: CriterionFieldRendererComponent;

    /**
     * Validates the input values for a given operator and unit.
     *
     * @param value - The primary value to validate.
     * @param value2 - The secondary value to validate (for ranges).
     * @param operator - The comparison operator being used.
     * @param unitMultiplier - The optional unit multiplier (e.g., for file sizes).
     * @returns A record of error messages where keys indicate the field part with the error.
     */
    validate: (
        value: SearchValue,
        value2: SearchValue,
        operator: string,
        unitMultiplier?: string
    ) => Record<string, string>;

    /**
     * Transforms the raw UI values into a format suitable for the final search query.
     *
     * @param value - The primary raw value.
     * @param value2 - The secondary raw value.
     * @param operator - The comparison operator.
     * @param unitMultiplier - The unit multiplier.
     * @returns The processed value (often an array or string) and any final unit multiplier.
     */
    process: (
        value: SearchValue,
        value2: SearchValue,
        operator: string,
        unitMultiplier?: string
    ) => {
        finalValue: unknown;
        unitMultiplier?: string;
    };

    /**
     * Optional method to create a human-readable string representation of the criterion for display.
     *
     * @param value - The processed primary value.
     * @param value2 - The processed secondary value.
     * @param operator - The comparison operator.
     * @param unitMultiplier - The unit multiplier used.
     * @param metadata - Contextual data (tags, locations) for resolving IDs to names.
     * @returns A friendly display string.
     */
    formatDisplay?: (
        value: unknown,
        value2: unknown,
        operator: string,
        unitMultiplier?: string,
        metadata?: StoreMetadata
    ) => string;
}
