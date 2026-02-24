import { createSignal, createMemo, createEffect } from 'solid-js';
import { SearchCriterion, LogicalOperator, SearchGroup } from '../../../core/store/filterStore';
import { createId } from '../../../lib/primitives/createId';
import { SEARCH_FIELDS, OPERATORS_FOR_TYPE } from './searchConstants';
import { computeDisplayValue } from './searchHelpers';
import { fromISO } from '../../../utils/format';
import { supportedFormats } from '../../../core/store/systemStore';
import { criterionHandlerRegistry } from './fields';

export type SearchValue = string | number | null | Date;

export const useAdvancedSearch = (
    metadata: { locations: { id: number; name: string }[]; tags: { id: number; name: string }[] },
    options: {
        isOpen: () => boolean;
        initialQuery: () => SearchGroup | undefined;
    }
) => {
    // Top-level state
    const [criteria, setCriteria] = createSignal<SearchCriterion[]>([]);
    const [matchMode, setMatchMode] = createSignal<LogicalOperator>('and');

    // New criteria builder state
    const [currentKey, setCurrentKey] = createSignal('tags');
    const [currentOperator, setCurrentOperator] = createSignal('contains');
    const [currentValue, setCurrentValue] = createSignal<SearchValue>(null);
    const [currentValue2, setCurrentValue2] = createSignal<SearchValue>(null);
    const [currentUnit, setCurrentUnit] = createSignal('1048576');
    const [validationErrors, setValidationErrors] = createSignal<Record<string, string>>({});

    // Editing state
    const [editingId, setEditingId] = createSignal<string | null>(null);
    const [editingValue, setEditingValue] = createSignal<SearchValue>(null);
    const [editingValue2, setEditingValue2] = createSignal<SearchValue>(null);
    const [editingUnit, setEditingUnit] = createSignal('1048576');
    const [editingValidationErrors, setEditingValidationErrors] = createSignal<
        Record<string, string>
    >({});

    const selectedField = createMemo(() => SEARCH_FIELDS.find(f => f.value === currentKey()));
    const availableOperators = createMemo(() => {
        const field = selectedField();
        return field ? OPERATORS_FOR_TYPE[field.type] || [] : [];
    });

    // Initialize from props when opening
    createEffect(() => {
        if (options.isOpen()) {
            const query = options.initialQuery();
            if (query) {
                setMatchMode(query.logicalOperator);
                const initialCriteria = query.items
                    .filter(item => !('items' in item))
                    .map(item => ({
                        ...(item as SearchCriterion),
                        displayValue:
                            (item as SearchCriterion).displayValue ||
                            computeDisplayValue(item as SearchCriterion, metadata)
                    }));
                setCriteria(initialCriteria);
            } else {
                setCriteria([]);
                setMatchMode('and');
            }
            // Reset builder state
            setCurrentValue(null);
            setCurrentValue2(null);
            setValidationErrors({});
            setEditingValidationErrors({});
            setEditingId(null);
        }
    });

    // Reset operator when key changes
    createEffect(() => {
        const field = selectedField();
        if (field) {
            const defaultOp = OPERATORS_FOR_TYPE[field.type]?.[0]?.value;
            setCurrentOperator(defaultOp || '');

            // Sync initial state values with UI visual defaults, so validation matches representation.
            if (field.type === 'rating') {
                setCurrentValue(0);
            } else if (field.type === 'select') {
                const firstFormat = supportedFormats()[0]?.extensions[0];
                setCurrentValue(firstFormat || '');
            } else if (field.type === 'folder') {
                const firstFolder = metadata.locations[0]?.id;
                setCurrentValue(firstFolder ? Number(firstFolder) : null);
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

    const validateCriterion = (
        field: ReturnType<typeof selectedField>,
        op: string,
        val: SearchValue,
        val2: SearchValue,
        unit?: string
    ) => {
        if (!field) return { value: 'Invalid field' };
        const handlerName = field.value === 'size' ? 'size' : field.type || 'text';
        const handler = criterionHandlerRegistry[handlerName];
        if (!handler) return {};
        return handler.validate(val, val2, op, unit);
    };

    const validateCurrent = () => {
        const errors = validateCriterion(
            selectedField(),
            currentOperator(),
            currentValue(),
            currentValue2(),
            currentUnit()
        );
        setValidationErrors(errors);
        return Object.keys(errors).length === 0;
    };

    const handleStartEdit = (item: SearchCriterion) => {
        setEditingId(item.id);
        setEditingValidationErrors({});
        setEditingUnit(item.unitMultiplier || '1048576');

        if (Array.isArray(item.value)) {
            if (item.key === 'size') {
                const mult = Number(item.unitMultiplier || '1048576');
                setEditingValue(Number(item.value[0]) / mult);
                setEditingValue2(Number(item.value[1]) / mult);
            } else if (['added_at', 'created_at', 'modified_at'].includes(item.key)) {
                const d1 = fromISO(String(item.value[0]));
                const d2 = fromISO(String(item.value[1]));
                setEditingValue(d1 as unknown as SearchValue);
                setEditingValue2(d2 as unknown as SearchValue);
            } else {
                setEditingValue(item.value[0] as SearchValue);
                setEditingValue2(item.value[1] as SearchValue);
            }
        } else {
            if (item.key === 'size') {
                const mult = Number(item.unitMultiplier || '1048576');
                setEditingValue(Number(item.value) / mult);
            } else if (['added_at', 'created_at', 'modified_at'].includes(item.key)) {
                const d = fromISO(String(item.value));
                setEditingValue(d as unknown as SearchValue);
            } else {
                setEditingValue(item.value as SearchValue);
            }
            setEditingValue2(null);
        }
    };

    const handleConfirmEdit = () => {
        const id = editingId();
        if (!id) return;

        const currentItem = criteria().find(c => c.id === id);
        if (!currentItem) return;

        const field = SEARCH_FIELDS.find(f => f.value === currentItem.key);
        const handlerName = currentItem.key === 'size' ? 'size' : field?.type || 'text';
        const handler = criterionHandlerRegistry[handlerName];

        const errors = handler.validate(
            editingValue(),
            editingValue2(),
            currentItem.operator,
            editingUnit()
        );

        if (Object.keys(errors).length > 0) {
            setEditingValidationErrors(errors);
            return;
        }

        const { finalValue, unitMultiplier } = handler.process(
            editingValue(),
            editingValue2(),
            currentItem.operator,
            editingUnit()
        );

        setCriteria(prev =>
            prev.map(c => {
                if (c.id === id) {
                    const displayValue = handler.formatDisplay?.(
                        editingValue(),
                        currentItem.operator,
                        unitMultiplier,
                        {
                            locations: metadata.locations,
                            tags: metadata.tags,
                            supportedFormats: supportedFormats()
                        }
                    );

                    const updatedCriterion = {
                        ...c,
                        value: finalValue as SearchCriterion['value'],
                        unitMultiplier,
                        displayValue: undefined // force new calculation
                    };

                    return {
                        ...updatedCriterion,
                        displayValue:
                            displayValue ||
                            computeDisplayValue(updatedCriterion as SearchCriterion, metadata)
                    };
                }
                return c;
            })
        );
        setEditingId(null);
        setEditingValidationErrors({});
    };

    const handleAddCriteria = () => {
        if (!validateCurrent()) return;

        const field = selectedField();
        const handlerName = currentKey() === 'size' ? 'size' : field?.type || 'text';
        const handler = criterionHandlerRegistry[handlerName];

        const { finalValue, unitMultiplier } = handler.process(
            currentValue(),
            currentValue2(),
            currentOperator(),
            currentUnit()
        );

        const internalDisplayValue = handler.formatDisplay?.(
            currentValue(),
            currentOperator(),
            unitMultiplier,
            {
                locations: metadata.locations,
                tags: metadata.tags,
                supportedFormats: supportedFormats()
            }
        );

        const newCriterion: SearchCriterion = {
            id: createId('criterion'),
            key: currentKey(),
            operator: currentOperator(),
            value: finalValue as SearchCriterion['value'],
            unitMultiplier,
            displayValue:
                internalDisplayValue ||
                computeDisplayValue(
                    {
                        key: currentKey(),
                        value: finalValue as SearchCriterion['value'],
                        unitMultiplier
                    },
                    metadata
                )
        };

        setCriteria([...criteria(), newCriterion]);

        setCurrentValue(null);
        setCurrentValue2(null);
    };

    const handleRemoveCriteria = (id: string) => {
        setCriteria(criteria().filter(c => c.id !== id));
    };

    const handleReset = () => {
        setCriteria([]);
        setMatchMode('and');
    };

    return {
        // Data
        criteria,
        matchMode,
        currentKey,
        currentOperator,
        currentValue,
        currentValue2,
        currentUnit,
        validationErrors,
        editingId,
        editingValue,
        editingValue2,
        editingUnit,
        editingValidationErrors,
        selectedField,
        availableOperators,

        // Mutators
        setCriteria,
        setMatchMode,
        setCurrentKey,
        setCurrentOperator,
        setCurrentValue,
        setCurrentValue2,
        setCurrentUnit,
        setValidationErrors,
        setEditingId,
        setEditingValue,
        setEditingValue2,
        setEditingUnit,
        setEditingValidationErrors,

        // Actions
        handleStartEdit,
        handleConfirmEdit,
        handleAddCriteria,
        handleRemoveCriteria,
        handleReset
    };
};
