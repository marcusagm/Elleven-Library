import { createSignal, createMemo, createEffect } from 'solid-js';
import { SearchCriterion, LogicalOperator, SearchGroup } from '../../../core/store/filterStore';
import { createId } from '../../../lib/primitives/createId';
import { SEARCH_FIELDS, OPERATORS_FOR_TYPE } from './searchConstants';
import { computeDisplayValue } from './searchHelpers';
import { fromISO } from '../../../utils/format';
import { supportedFormats } from '../../../core/store/systemStore';
import { criterionHandlerRegistry } from './fields';

export type SearchValue = string | number | null | Date;

/**
 * Custom hook that manages the complex state and logic for the Advanced Search system.
 * Handles the construction, validation, and editing of multiple search criteria,
 * including integration with specialized field handlers and metadata resolution.
 *
 * @param metadata - Contextual information from stores (e.g., tags, locations).
 * @param queryOptions - Configuration for visibility and initial query state.
 * @returns An object containing reactive state accessors and action handlers.
 */
export const useAdvancedSearch = (
    metadata: {
        locations: { id: number; name: string }[];
        tags: { id: number; name: string }[];
    },
    queryOptions: {
        /** Function that returns whether the advanced search panel is currently open. */
        isOpen: () => boolean;
        /** Function that returns the initial search group to populate the editor. */
        initialQuery: () => SearchGroup | undefined;
    }
) => {
    // Top-level state management
    const [criteria, setCriteria] = createSignal<SearchCriterion[]>([]);
    const [matchMode, setMatchMode] = createSignal<LogicalOperator>('and');

    // New criteria builder (Add mode) state
    const [currentKey, setCurrentKey] = createSignal('tags');
    const [currentOperator, setCurrentOperator] = createSignal('contains');
    const [currentValue, setCurrentValue] = createSignal<SearchValue>(null);
    const [currentValue2, setCurrentValue2] = createSignal<SearchValue>(null);
    const [currentUnitMultiplier, setCurrentUnitMultiplier] = createSignal('1048576');
    const [validationErrors, setValidationErrors] = createSignal<Record<string, string>>({});

    // Existing criteria editing state
    const [editingId, setEditingId] = createSignal<string | null>(null);
    const [editingValue, setEditingValue] = createSignal<SearchValue>(null);
    const [editingValue2, setEditingValue2] = createSignal<SearchValue>(null);
    const [editingUnitMultiplier, setEditingUnitMultiplier] = createSignal('1048576');
    const [editingValidationErrors, setEditingValidationErrors] = createSignal<
        Record<string, string>
    >({});

    /** Derived signal identifying the currently selected field configuration. */
    const selectedField = createMemo(() =>
        SEARCH_FIELDS.find(field => field.value === currentKey())
    );

    /** Derived list of operators valid for the currently selected field's data type. */
    const availableOperators = createMemo(() => {
        const field = selectedField();
        return field ? OPERATORS_FOR_TYPE[field.type] || [] : [];
    });

    /** Synchronizes internal state with the provided initial query whenever the panel opens. */
    createEffect(() => {
        if (queryOptions.isOpen()) {
            const initialQuery = queryOptions.initialQuery();
            if (initialQuery) {
                setMatchMode(initialQuery.logicalOperator);
                const initialCriteriaList = initialQuery.items
                    .filter(item => !('items' in item))
                    .map(item => ({
                        ...(item as SearchCriterion),
                        displayValue:
                            (item as SearchCriterion).displayValue ||
                            computeDisplayValue(item as SearchCriterion, metadata)
                    }));
                setCriteria(initialCriteriaList);
            } else {
                setCriteria([]);
                setMatchMode('and');
            }
            // Reset temporary builder and editing states
            setCurrentValue(null);
            setCurrentValue2(null);
            setValidationErrors({});
            setEditingValidationErrors({});
            setEditingId(null);
        }
    });

    /** Automatically resets the operator and values when the search key (field) changes. */
    createEffect(() => {
        const field = selectedField();
        if (field) {
            const defaultComparisonOperator = OPERATORS_FOR_TYPE[field.type]?.[0]?.value;
            setCurrentOperator(defaultComparisonOperator || '');

            // Synchronize initial state values with UI visual expectations.
            if (field.type === 'rating') {
                setCurrentValue(0);
            } else if (field.type === 'select') {
                const firstAvailableFormat = supportedFormats()[0]?.extensions[0];
                setCurrentValue(firstAvailableFormat || '');
            } else if (field.type === 'folder') {
                const firstLocationId = metadata.locations[0]?.id;
                setCurrentValue(firstLocationId ? Number(firstLocationId) : null);
            } else if (field.type === 'number' || field.value === 'size') {
                setCurrentValue(1);
            } else {
                setCurrentValue(null);
            }

            setCurrentValue2(null);
            setValidationErrors({});
            setEditingValidationErrors({});
        }
    });

    /**
     * Internal utility to validate a set of criterion values against their corresponding field handler.
     *
     * @param field - The selected field definition.
     * @param operator - The comparison operator.
     * @param value - Primary value.
     * @param value2 - Secondary value (for ranges).
     * @param unitMultiplier - Optional unit multiplier.
     * @returns A map of validation errors.
     */
    const validateCriterionValues = (
        field: ReturnType<typeof selectedField>,
        operator: string,
        value: SearchValue,
        value2: SearchValue,
        unitMultiplier?: string
    ) => {
        if (!field) return { value: 'Invalid field' };
        const handlerName = field.value === 'size' ? 'size' : field.type || 'text';
        const fieldHandler = criterionHandlerRegistry[handlerName];
        if (!fieldHandler) return {};
        return fieldHandler.validate(value, value2, operator, unitMultiplier);
    };

    /** Validates the current 'Add mode' builder state. */
    const validateCurrentBuilderState = () => {
        const activeErrors = validateCriterionValues(
            selectedField(),
            currentOperator(),
            currentValue(),
            currentValue2(),
            currentUnitMultiplier()
        );
        setValidationErrors(activeErrors);
        return Object.keys(activeErrors).length === 0;
    };

    /**
     * Activates the editing mode for a specific existing criterion.
     * @param criterionItem - The specific search criterion to be edited.
     */
    const handleStartEdit = (criterionItem: SearchCriterion) => {
        setEditingId(criterionItem.id);
        setEditingValidationErrors({});
        setEditingUnitMultiplier(criterionItem.unitMultiplier || '1048576');

        if (Array.isArray(criterionItem.value)) {
            if (criterionItem.key === 'size') {
                const multiplier = Number(criterionItem.unitMultiplier || '1048576');
                setEditingValue(Number(criterionItem.value[0]) / multiplier);
                setEditingValue2(Number(criterionItem.value[1]) / multiplier);
            } else if (['added_at', 'created_at', 'modified_at'].includes(criterionItem.key)) {
                const startDateObject = fromISO(String(criterionItem.value[0]));
                const endDateObject = fromISO(String(criterionItem.value[1]));
                setEditingValue(startDateObject as unknown as SearchValue);
                setEditingValue2(endDateObject as unknown as SearchValue);
            } else {
                setEditingValue(criterionItem.value[0] as SearchValue);
                setEditingValue2(criterionItem.value[1] as SearchValue);
            }
        } else {
            if (criterionItem.key === 'size') {
                const multiplier = Number(criterionItem.unitMultiplier || '1048576');
                setEditingValue(Number(criterionItem.value) / multiplier);
            } else if (['added_at', 'created_at', 'modified_at'].includes(criterionItem.key)) {
                const dateValueObject = fromISO(String(criterionItem.value));
                setEditingValue(dateValueObject as unknown as SearchValue);
            } else {
                setEditingValue(criterionItem.value as SearchValue);
            }
            setEditingValue2(null);
        }
    };

    /** Confirms changes made in the editing session and updates the main criteria list. */
    const handleConfirmEdit = () => {
        const activeEditingId = editingId();
        if (!activeEditingId) return;

        const originalCriterion = criteria().find(criterion => criterion.id === activeEditingId);
        if (!originalCriterion) return;

        const fieldDefinition = SEARCH_FIELDS.find(field => field.value === originalCriterion.key);
        const handlerName =
            originalCriterion.key === 'size' ? 'size' : fieldDefinition?.type || 'text';
        const fieldHandler = criterionHandlerRegistry[handlerName];

        const validationErrorsMap = fieldHandler.validate(
            editingValue(),
            editingValue2(),
            originalCriterion.operator,
            editingUnitMultiplier()
        );

        if (Object.keys(validationErrorsMap).length > 0) {
            setEditingValidationErrors(validationErrorsMap);
            return;
        }

        const { finalValue, unitMultiplier } = fieldHandler.process(
            editingValue(),
            editingValue2(),
            originalCriterion.operator,
            editingUnitMultiplier()
        );

        setCriteria(previousCriteriaList =>
            previousCriteriaList.map(criterion => {
                if (criterion.id === activeEditingId) {
                    const humanReadableString = fieldHandler.formatDisplay?.(
                        editingValue(),
                        editingValue2(),
                        originalCriterion.operator,
                        unitMultiplier,
                        {
                            locations: metadata.locations,
                            tags: metadata.tags,
                            supportedFormats: supportedFormats()
                        }
                    );

                    const partiallyUpdatedCriterion = {
                        ...criterion,
                        value: finalValue as SearchCriterion['value'],
                        unitMultiplier,
                        displayValue: undefined // Forced invalidation for recalculated display.
                    };

                    return {
                        ...partiallyUpdatedCriterion,
                        displayValue:
                            humanReadableString ||
                            computeDisplayValue(
                                partiallyUpdatedCriterion as SearchCriterion,
                                metadata
                            )
                    };
                }
                return criterion;
            })
        );
        setEditingId(null);
        setEditingValidationErrors({});
    };

    /** Adds the currently constructed criterion from the builder into the active list. */
    const handleAddCriteria = () => {
        if (!validateCurrentBuilderState()) return;

        const fieldDefinition = selectedField();
        const handlerName = currentKey() === 'size' ? 'size' : fieldDefinition?.type || 'text';
        const fieldHandler = criterionHandlerRegistry[handlerName];

        const { finalValue, unitMultiplier } = fieldHandler.process(
            currentValue(),
            currentValue2(),
            currentOperator(),
            currentUnitMultiplier()
        );

        const internalDisplayDescription = fieldHandler.formatDisplay?.(
            currentValue(),
            currentValue2(),
            currentOperator(),
            unitMultiplier,
            {
                locations: metadata.locations,
                tags: metadata.tags,
                supportedFormats: supportedFormats()
            }
        );

        const newCriterionObject: SearchCriterion = {
            id: createId('criterion'),
            key: currentKey(),
            operator: currentOperator(),
            value: finalValue as SearchCriterion['value'],
            unitMultiplier,
            displayValue:
                internalDisplayDescription ||
                computeDisplayValue(
                    {
                        key: currentKey(),
                        value: finalValue as SearchCriterion['value'],
                        unitMultiplier
                    },
                    metadata
                )
        };

        setCriteria([...criteria(), newCriterionObject]);

        // Clean up builder values after successful addition
        setCurrentValue(null);
        setCurrentValue2(null);
    };

    /**
     * Removes a specific criterion from the active search list.
     * @param criterionId - The unique identifier of the criterion to be deleted.
     */
    const handleRemoveCriteria = (criterionId: string) => {
        setCriteria(criteria().filter(criterion => criterion.id !== criterionId));
    };

    /** Wipes all defined criteria and resets match mode to default. */
    const handleResetAllCriteria = () => {
        setCriteria([]);
        setMatchMode('and');
    };

    return {
        // Reactive State Accessors
        criteria,
        matchMode,
        currentKey,
        currentOperator,
        currentValue,
        currentValue2,
        currentUnitMultiplier,
        validationErrors,
        editingId,
        editingValue,
        editingValue2,
        editingUnitMultiplier,
        editingValidationErrors,
        selectedField,
        availableOperators,

        // Direct State Mutators
        setCriteria,
        setMatchMode,
        setCurrentKey,
        setCurrentOperator,
        setCurrentValue,
        setCurrentValue2,
        setCurrentUnitMultiplier,
        setValidationErrors,
        setEditingId,
        setEditingValue,
        setEditingValue2,
        setEditingUnitMultiplier,
        setEditingValidationErrors,

        // Business Logic Actions
        handleStartEdit,
        handleConfirmEdit,
        handleAddCriteria,
        handleRemoveCriteria,
        handleReset: handleResetAllCriteria
    };
};
