import { createSignal, createMemo, createEffect } from 'solid-js';
import { SearchCriterion, LogicalOperator, SearchGroup } from '../../../core/store/filterStore';
import { createId } from '../../../lib/primitives/createId';
import { SEARCH_FIELDS, OPERATORS_FOR_TYPE, SIZE_UNITS } from './searchConstants';
import { computeDisplayValue } from './searchHelpers';
import { formatToISO, fromISO, formatToDisplay } from '../../../utils/format';

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
            setCurrentValue(null);
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
        const errors: Record<string, string> = {};

        if (val === null || val === '') {
            errors.value = 'Value is required';
        }

        if (op === 'between' && (val2 === null || val2 === '')) {
            errors.value2 = 'End value is required';
        }

        if (field?.type === 'date') {
            if (val === null) errors.value = 'Date is required';
            if (op === 'between' && val2 === null) errors.value2 = 'End date is required';
        }

        if (field?.value === 'size') {
            if (!unit || isNaN(Number(unit)) || !SIZE_UNITS.some(u => u.value === unit)) {
                errors.unit = 'Unit is required';
            }
        }

        return errors;
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
        const errors = validateCriterion(
            field,
            currentItem.operator,
            editingValue(),
            editingValue2(),
            editingUnit()
        );

        if (Object.keys(errors).length > 0) {
            setEditingValidationErrors(errors);
            return;
        }

        setCriteria(prev =>
            prev.map(c => {
                if (c.id === id) {
                    let finalValue:
                        | string
                        | number
                        | boolean
                        | null
                        | (string | number | boolean | null)[] =
                        editingValue() instanceof Date
                            ? formatToISO(editingValue() as Date)
                            : (editingValue() as string | number | null);
                    let displayValue: string | undefined;

                    // Handle size conversion
                    if (c.key === 'size') {
                        const multiplier = Number(editingUnit());
                        const label =
                            SIZE_UNITS.find(u => u.value === editingUnit())?.label || 'MB';
                        if (c.operator === 'between') {
                            const v1 = Math.round(Number(editingValue()) * multiplier);
                            const v2 = Math.round(Number(editingValue2()) * multiplier);
                            finalValue = [v1, v2];
                            displayValue = `${editingValue()} ${label} to ${editingValue2()} ${label}`;
                        } else {
                            finalValue = Math.round(Number(editingValue()) * multiplier);
                            displayValue = `${editingValue()} ${label}`;
                        }
                    } else if (c.operator === 'between') {
                        if (['added_at', 'created_at', 'modified_at'].includes(c.key)) {
                            const v1 = formatToISO(editingValue() as Date | string);
                            const v2 = formatToISO(editingValue2() as Date | string);
                            finalValue = [v1, v2];
                            displayValue = `${formatToDisplay(v1)} to ${formatToDisplay(v2)}`;
                        } else {
                            finalValue = [
                                editingValue() as string | number | null,
                                editingValue2() as string | number | null
                            ];
                            displayValue = `${editingValue()} to ${editingValue2()}`;
                        }
                    } else if (['added_at', 'created_at', 'modified_at'].includes(c.key)) {
                        finalValue = formatToISO(editingValue() as Date | string);
                        displayValue = formatToDisplay(finalValue);
                    } else if (c.key === 'folder') {
                        displayValue =
                            metadata.locations.find(l => String(l.id) === String(editingValue()))
                                ?.name || String(editingValue());
                    } else if (c.key === 'tags') {
                        displayValue =
                            metadata.tags.find(t => String(t.id) === String(editingValue()))
                                ?.name || String(editingValue());
                    }

                    return {
                        ...c,
                        value: finalValue,
                        displayValue,
                        unitMultiplier: c.key === 'size' ? editingUnit() : undefined
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

        let finalValue: string | number | boolean | null | (string | number | boolean | null)[] =
            currentValue() instanceof Date
                ? formatToISO(currentValue() as Date)
                : (currentValue() as string | number | null);
        let displayValue: string | undefined;

        const label = SIZE_UNITS.find(u => u.value === currentUnit())?.label || 'MB';

        if (currentKey() === 'size' && finalValue !== null) {
            const multiplier = Number(currentUnit());
            if (currentOperator() === 'between') {
                const v1 = Math.round(Number(finalValue) * multiplier);
                const v2 = Math.round(Number(currentValue2()) * multiplier);
                finalValue = [v1, v2];
                displayValue = `${currentValue()} ${label} to ${currentValue2()} ${label}`;
            } else {
                finalValue = Math.round(Number(finalValue) * multiplier);
                displayValue = `${currentValue()} ${label}`;
            }
        } else if (currentOperator() === 'between') {
            if (selectedField()?.type === 'date') {
                const v1 = formatToISO(currentValue() as Date | string);
                const v2 = formatToISO(currentValue2() as Date | string);
                finalValue = [v1, v2];
                displayValue = `${formatToDisplay(v1)} to ${formatToDisplay(v2)}`;
            } else {
                finalValue = [
                    currentValue() as string | number | null,
                    currentValue2() as string | number | null
                ];
                displayValue = `${currentValue()} to ${currentValue2()}`;
            }
        } else if (selectedField()?.type === 'date') {
            finalValue = formatToISO(currentValue() as Date | string);
            displayValue = formatToDisplay(finalValue);
        } else if (currentKey() === 'folder') {
            displayValue =
                metadata.locations.find(l => String(l.id) === String(currentValue()))?.name ||
                String(currentValue());
        } else if (currentKey() === 'tags') {
            displayValue =
                metadata.tags.find(t => String(t.id) === String(currentValue()))?.name ||
                String(currentValue());
        }

        const newCriterion: SearchCriterion = {
            id: createId('criterion'),
            key: currentKey(),
            operator: currentOperator(),
            value: finalValue,
            displayValue:
                displayValue ||
                computeDisplayValue(
                    {
                        key: currentKey(),
                        value: finalValue,
                        unitMultiplier: currentKey() === 'size' ? currentUnit() : undefined
                    },
                    metadata
                ),
            unitMultiplier: currentKey() === 'size' ? currentUnit() : undefined
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
