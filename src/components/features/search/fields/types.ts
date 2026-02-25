import { Component } from 'solid-js';
import { SearchValue } from '../useAdvancedSearch';

export interface CriterionFieldRendererProps {
    /** The actual search key/field being evaluated (e.g. 'size', 'added_at', 'filename') */
    fieldKey: string;

    /** The configured comparison logic (e.g. 'eq', 'between', 'contains') */
    operator: string;

    /** Primary field value */
    value: SearchValue;
    /** Mutator for primary field value */
    setValue: (val: SearchValue) => void;

    /** Secondary field value (Only provided/visible when operator relies on a range, like 'between') */
    value2?: SearchValue;
    /** Mutator for secondary field value */
    setValue2?: (val: SearchValue) => void;

    /** Additional metric/unit configuration (Used for Size fields) */
    unit?: string;
    /** Mutator for unit configuration */
    setUnit?: (unit: string) => void;

    /** Active validation errors corresponding to this field payload */
    errors: Record<string, string>;

    /** Optional explicit visual override (Defaults to basic md inputs, allows sm for Editor iteration) */
    size?: 'sm' | 'md';
}

export type CriterionFieldRendererComponent = Component<CriterionFieldRendererProps>;

export type StoreMetadata = {
    locations: { id: number; name: string }[];
    tags: { id: number; name: string }[];
    supportedFormats?: { name: string; extensions: string[] }[];
};

export interface SearchFieldHandler {
    component: CriterionFieldRendererComponent;

    validate: (
        value: SearchValue,
        value2: SearchValue,
        operator: string,
        unit?: string
    ) => Record<string, string>;

    process: (
        value: SearchValue,
        value2: SearchValue,
        operator: string,
        unit?: string
    ) => {
        finalValue: unknown;
        unitMultiplier?: string;
    };

    formatDisplay?: (
        value: unknown,
        value2: unknown,
        operator: string,
        unitMultiplier?: string,
        metadata?: StoreMetadata
    ) => string;
}
