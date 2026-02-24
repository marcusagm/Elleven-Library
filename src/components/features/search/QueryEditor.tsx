import { Component, For, Show, createMemo } from 'solid-js';
import { Info, Pencil, Check, Trash2, CircleQuestionMark } from 'lucide-solid';
import { Input } from '../../ui/Input';
import { NumberInput } from '../../ui/NumberInput';
import { DateInput } from '../../ui/DateInput';
import { Select } from '../../ui/Select';
import { Button } from '../../ui/Button';
import { RadioGroup, RadioGroupItem } from '../../ui/RadioGroup';
import { Tooltip } from '../../ui/Tooltip';
import { SEARCH_FIELDS, OPERATORS_FOR_TYPE, SIZE_UNITS } from './searchConstants';
import { supportedFormats } from '../../../core/store/systemStore';
import { LogicalOperator, SearchCriterion } from '../../../core/store/filterStore';
import { useMetadata } from '../../../core/hooks';
import { cn } from '../../../lib/utils';
import { getHierarchicalFolders, getHierarchicalTags } from './searchHelpers';

import { useAdvancedSearch } from './useAdvancedSearch';

export interface QueryEditorProps {
    search: ReturnType<typeof useAdvancedSearch>;
}

const CriterionItem: Component<{
    item: SearchCriterion;
    index: number;
    search: ReturnType<typeof useAdvancedSearch>;
}> = props => {
    const metadata = useMetadata();
    const hierarchicalTags = createMemo(() => getHierarchicalTags(metadata.tags));
    const hierarchicalFolders = createMemo(() => getHierarchicalFolders(metadata.locations));

    const field = createMemo(() => SEARCH_FIELDS.find(f => f.value === props.item.key));
    const isEditing = () => props.search.editingId() === props.item.id;

    return (
        <div class={cn('criterion-item', isEditing() && 'editing')}>
            <span class="criterion-index">{props.index + 1}</span>
            <span class="criterion-field">{field()?.label || props.item.key}</span>
            <span class="criterion-operator">
                {OPERATORS_FOR_TYPE[field()?.type || '']?.find(o => o.value === props.item.operator)
                    ?.label || props.item.operator}
            </span>
            <span class="criterion-value">
                <Show
                    when={!isEditing()}
                    fallback={
                        <div class="edit-inputs">
                            <Show when={field()?.type === 'text'}>
                                <Input
                                    size="sm"
                                    value={(props.search.editingValue() as string) || ''}
                                    onInput={e => {
                                        props.search.setEditingValue(e.currentTarget.value);
                                        if (props.search.editingValidationErrors().value)
                                            props.search.setEditingValidationErrors(
                                                (prev: Record<string, string>) => ({
                                                    ...prev,
                                                    value: ''
                                                })
                                            );
                                    }}
                                    error={!!props.search.editingValidationErrors().value}
                                    errorMessage={props.search.editingValidationErrors().value}
                                />
                            </Show>
                            <Show when={field()?.type === 'number' || props.item.key === 'size'}>
                                <div class="horizontal-inputs">
                                    <NumberInput
                                        size="sm"
                                        value={(props.search.editingValue() as number) ?? undefined}
                                        onChange={val => {
                                            props.search.setEditingValue(val ?? null);
                                            if (props.search.editingValidationErrors().value)
                                                props.search.setEditingValidationErrors(
                                                    (prev: Record<string, string>) => ({
                                                        ...prev,
                                                        value: ''
                                                    })
                                                );
                                        }}
                                        placeholder={
                                            props.item.operator === 'between'
                                                ? 'From...'
                                                : 'Value...'
                                        }
                                        error={!!props.search.editingValidationErrors().value}
                                        errorMessage={props.search.editingValidationErrors().value}
                                    />
                                    <Show when={props.item.operator === 'between'}>
                                        <span>to</span>
                                        <NumberInput
                                            size="sm"
                                            value={
                                                (props.search.editingValue2() as number) ??
                                                undefined
                                            }
                                            onChange={val => {
                                                props.search.setEditingValue2(val ?? null);
                                                if (props.search.editingValidationErrors().value2)
                                                    props.search.setEditingValidationErrors(
                                                        (prev: Record<string, string>) => ({
                                                            ...prev,
                                                            value2: ''
                                                        })
                                                    );
                                            }}
                                            placeholder="To..."
                                            error={!!props.search.editingValidationErrors().value2}
                                            errorMessage={
                                                props.search.editingValidationErrors().value2
                                            }
                                        />
                                    </Show>
                                    <Show when={props.item.key === 'size'}>
                                        <Select
                                            size="sm"
                                            class="unit-select"
                                            options={SIZE_UNITS}
                                            value={props.search.editingUnit()}
                                            onValueChange={val => {
                                                props.search.setEditingUnit(val);
                                                if (props.search.editingValidationErrors().unit)
                                                    props.search.setEditingValidationErrors(
                                                        (prev: Record<string, string>) => ({
                                                            ...prev,
                                                            unit: ''
                                                        })
                                                    );
                                            }}
                                            error={!!props.search.editingValidationErrors().unit}
                                            errorMessage={
                                                props.search.editingValidationErrors().unit
                                            }
                                        />
                                    </Show>
                                </div>
                            </Show>
                            <Show when={field()?.type === 'date'}>
                                <div class="horizontal-inputs">
                                    <DateInput
                                        size="sm"
                                        value={(props.search.editingValue() as Date) || null}
                                        onChange={val => {
                                            props.search.setEditingValue(val);
                                            if (props.search.editingValidationErrors().value)
                                                props.search.setEditingValidationErrors(
                                                    (prev: Record<string, string>) => ({
                                                        ...prev,
                                                        value: ''
                                                    })
                                                );
                                        }}
                                        placeholder={
                                            props.item.operator === 'between' ? 'From Date' : 'Date'
                                        }
                                        error={!!props.search.editingValidationErrors().value}
                                        errorMessage={props.search.editingValidationErrors().value}
                                    />
                                    <Show when={props.item.operator === 'between'}>
                                        <span>to</span>
                                        <DateInput
                                            size="sm"
                                            value={(props.search.editingValue2() as Date) || null}
                                            onChange={val => {
                                                props.search.setEditingValue2(val);
                                                if (props.search.editingValidationErrors().value2)
                                                    props.search.setEditingValidationErrors(
                                                        (prev: Record<string, string>) => ({
                                                            ...prev,
                                                            value2: ''
                                                        })
                                                    );
                                            }}
                                            placeholder="To Date"
                                            error={!!props.search.editingValidationErrors().value2}
                                            errorMessage={
                                                props.search.editingValidationErrors().value2
                                            }
                                        />
                                    </Show>
                                </div>
                            </Show>
                            <Show when={field()?.type === 'tags'}>
                                <Select
                                    size="sm"
                                    options={hierarchicalTags()}
                                    value={String(props.search.editingValue() || '')}
                                    onValueChange={val => {
                                        props.search.setEditingValue(val);
                                        if (props.search.editingValidationErrors().value)
                                            props.search.setEditingValidationErrors(
                                                (prev: Record<string, string>) => ({
                                                    ...prev,
                                                    value: ''
                                                })
                                            );
                                    }}
                                    searchable
                                    error={!!props.search.editingValidationErrors().value}
                                    errorMessage={props.search.editingValidationErrors().value}
                                />
                            </Show>
                            <Show when={field()?.type === 'folder'}>
                                <Select
                                    size="sm"
                                    options={hierarchicalFolders()}
                                    value={String(props.search.editingValue() || '')}
                                    onValueChange={val => {
                                        props.search.setEditingValue(Number(val));
                                        if (props.search.editingValidationErrors().value)
                                            props.search.setEditingValidationErrors(
                                                (prev: Record<string, string>) => ({
                                                    ...prev,
                                                    value: ''
                                                })
                                            );
                                    }}
                                    searchable
                                    error={!!props.search.editingValidationErrors().value}
                                    errorMessage={props.search.editingValidationErrors().value}
                                />
                            </Show>
                            <Show when={field()?.type === 'rating'}>
                                <Select
                                    size="sm"
                                    options={[0, 1, 2, 3, 4, 5].map(v => ({
                                        value: String(v),
                                        label: `${v} Stars`
                                    }))}
                                    value={String(props.search.editingValue() ?? '0')}
                                    onValueChange={val => {
                                        props.search.setEditingValue(Number(val));
                                        if (props.search.editingValidationErrors().value)
                                            props.search.setEditingValidationErrors(
                                                (prev: Record<string, string>) => ({
                                                    ...prev,
                                                    value: ''
                                                })
                                            );
                                    }}
                                    error={!!props.search.editingValidationErrors().value}
                                    errorMessage={props.search.editingValidationErrors().value}
                                />
                            </Show>
                            <Show when={field()?.type === 'select'}>
                                <Select
                                    size="sm"
                                    options={supportedFormats().flatMap(f =>
                                        f.extensions.map(ext => ({
                                            value: ext,
                                            label: ext.toUpperCase()
                                        }))
                                    )}
                                    value={(props.search.editingValue() as string) || ''}
                                    onValueChange={val => {
                                        props.search.setEditingValue(val);
                                        if (props.search.editingValidationErrors().value)
                                            props.search.setEditingValidationErrors(
                                                (prev: Record<string, string>) => ({
                                                    ...prev,
                                                    value: ''
                                                })
                                            );
                                    }}
                                    searchable
                                    error={!!props.search.editingValidationErrors().value}
                                    errorMessage={props.search.editingValidationErrors().value}
                                />
                            </Show>
                        </div>
                    }
                >
                    {props.item.displayValue}
                </Show>
            </span>
            <Show
                when={!isEditing()}
                fallback={
                    <Button variant="ghost" size="icon" onClick={props.search.handleConfirmEdit}>
                        <Check size={16} />
                    </Button>
                }
            >
                <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => props.search.handleStartEdit(props.item)}
                >
                    <Pencil size={14} />
                </Button>
            </Show>
            <Button
                variant="ghost-destructive"
                size="icon"
                onClick={() => props.search.handleRemoveCriteria(props.item.id)}
            >
                <Trash2 size={14} />
            </Button>
        </div>
    );
};

export const QueryEditor: Component<QueryEditorProps> = props => {
    return (
        <div class="query-editor-section">
            <Tooltip content="Review and manage your active criteria. You can edit values in-line or remove them. All criteria work together based on the 'Any' or 'All' match mode below.">
                <div class="section-title">
                    Query Editor <CircleQuestionMark size={12} />
                </div>
            </Tooltip>
            <div class="criteria-list">
                <Show when={props.search.criteria().length === 0}>
                    <div class="empty-query-info">
                        <Info size={24} />
                        <div>
                            <strong>Empty Query</strong>
                            <br />
                            Your query is currently empty. Create a criteria above to enable search.
                        </div>
                    </div>
                </Show>
                <For each={props.search.criteria()}>
                    {(item, index) => (
                        <CriterionItem item={item} index={index()} search={props.search} />
                    )}
                </For>
            </div>

            <div class="match-mode-section">
                <Tooltip content="Choose how to combine your criteria. 'All' requires every condition to be met, while 'Any' matches if at least one condition is met.">
                    <div class="section-title">
                        Match Mode <CircleQuestionMark size={12} />
                    </div>
                </Tooltip>
                <RadioGroup
                    value={props.search.matchMode()}
                    onValueChange={val => props.search.setMatchMode(val as LogicalOperator)}
                    orientation="horizontal"
                    class="match-radio-horizontal"
                >
                    <RadioGroupItem value="or" label="Any (OR)" />
                    <RadioGroupItem value="and" label="All (AND)" />
                </RadioGroup>
            </div>
        </div>
    );
};
