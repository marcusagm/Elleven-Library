import { Component, Show, createMemo } from 'solid-js';
import { Plus, CircleQuestionMark } from 'lucide-solid';
import { Select } from '../../ui/Select';
import { Input } from '../../ui/Input';
import { NumberInput } from '../../ui/NumberInput';
import { DateInput } from '../../ui/DateInput';
import { Button } from '../../ui/Button';
import { Tooltip } from '../../ui/Tooltip';
import { SEARCH_FIELDS, SIZE_UNITS } from './searchConstants';
import { useMetadata } from '../../../core/hooks';
import { supportedFormats } from '../../../core/store/systemStore';
import { getHierarchicalTags, getHierarchicalFolders } from './searchHelpers';

import { useAdvancedSearch } from './useAdvancedSearch';

export interface CriteriaBuilderProps {
    search: ReturnType<typeof useAdvancedSearch>;
}

export const CriteriaBuilder: Component<CriteriaBuilderProps> = props => {
    const metadata = useMetadata();

    const hierarchicalTags = createMemo(() => getHierarchicalTags(metadata.tags));
    const hierarchicalFolders = createMemo(() => getHierarchicalFolders(metadata.locations));

    return (
        <div class="criteria-builder-section">
            <Tooltip content="Choose a field, operator, and value to add new search criteria. Filter by name, tags, date, and more.">
                <div class="section-title">
                    Criteria Builder <CircleQuestionMark size={12} />
                </div>
            </Tooltip>
            <div class="builder-row">
                <Select
                    options={SEARCH_FIELDS}
                    value={props.search.currentKey()}
                    onValueChange={props.search.setCurrentKey}
                />
                <Select
                    options={props.search.availableOperators()}
                    value={props.search.currentOperator()}
                    onValueChange={props.search.setCurrentOperator}
                />

                <div class="builder-value-field">
                    <Show when={props.search.selectedField()?.type === 'text'}>
                        <Input
                            value={(props.search.currentValue() as string) || ''}
                            onInput={e => {
                                props.search.setCurrentValue(e.currentTarget.value);
                                if (props.search.validationErrors().value)
                                    props.search.setValidationErrors((prev: any) => ({
                                        ...prev,
                                        value: ''
                                    }));
                            }}
                            placeholder="Value..."
                            error={!!props.search.validationErrors().value}
                            errorMessage={props.search.validationErrors().value}
                        />
                    </Show>
                    <Show when={props.search.selectedField()?.type === 'number'}>
                        <div class="number-input-group">
                            <NumberInput
                                value={(props.search.currentValue() as number) ?? undefined}
                                onChange={val => {
                                    props.search.setCurrentValue(val ?? null);
                                    if (props.search.validationErrors().value)
                                        props.search.setValidationErrors((prev: any) => ({
                                            ...prev,
                                            value: ''
                                        }));
                                }}
                                placeholder={
                                    props.search.currentOperator() === 'between'
                                        ? 'From...'
                                        : 'Value...'
                                }
                                error={!!props.search.validationErrors().value}
                                errorMessage={props.search.validationErrors().value}
                            />
                            <Show when={props.search.currentOperator() === 'between'}>
                                <span class="range-separator">to</span>
                                <NumberInput
                                    value={(props.search.currentValue2() as number) ?? undefined}
                                    onChange={val => {
                                        props.search.setCurrentValue2(val ?? null);
                                        if (props.search.validationErrors().value2)
                                            props.search.setValidationErrors((prev: any) => ({
                                                ...prev,
                                                value2: ''
                                            }));
                                    }}
                                    placeholder="To..."
                                    error={!!props.search.validationErrors().value2}
                                    errorMessage={props.search.validationErrors().value2}
                                />
                            </Show>
                            <Show when={props.search.currentKey() === 'size'}>
                                <Select
                                    class="unit-select"
                                    options={SIZE_UNITS}
                                    value={props.search.currentUnit()}
                                    onValueChange={val => {
                                        props.search.setCurrentUnit(val);
                                        if (props.search.validationErrors().unit)
                                            props.search.setValidationErrors((prev: any) => ({
                                                ...prev,
                                                unit: ''
                                            }));
                                    }}
                                    error={!!props.search.validationErrors().unit}
                                    errorMessage={props.search.validationErrors().unit}
                                />
                            </Show>
                        </div>
                    </Show>
                    <Show when={props.search.selectedField()?.type === 'date'}>
                        <div class="date-input-group">
                            <DateInput
                                value={(props.search.currentValue() as Date) || null}
                                onChange={val => {
                                    props.search.setCurrentValue(val);
                                    if (props.search.validationErrors().value)
                                        props.search.setValidationErrors((prev: any) => ({
                                            ...prev,
                                            value: ''
                                        }));
                                }}
                                placeholder={
                                    props.search.currentOperator() === 'between'
                                        ? 'From Date'
                                        : 'Date'
                                }
                                error={!!props.search.validationErrors().value}
                                errorMessage={props.search.validationErrors().value}
                            />
                            <Show when={props.search.currentOperator() === 'between'}>
                                <span class="range-separator">to</span>
                                <DateInput
                                    value={(props.search.currentValue2() as Date) || null}
                                    onChange={val => {
                                        props.search.setCurrentValue2(val);
                                        if (props.search.validationErrors().value2)
                                            props.search.setValidationErrors((prev: any) => ({
                                                ...prev,
                                                value2: ''
                                            }));
                                    }}
                                    placeholder="To Date"
                                    error={!!props.search.validationErrors().value2}
                                    errorMessage={props.search.validationErrors().value2}
                                />
                            </Show>
                        </div>
                    </Show>
                    <Show when={props.search.selectedField()?.type === 'tags'}>
                        <div class="tag-select-placeholder">
                            <Select
                                options={hierarchicalTags()}
                                value={String(props.search.currentValue() || '')}
                                onValueChange={val => {
                                    props.search.setCurrentValue(val);
                                    if (props.search.validationErrors().value)
                                        props.search.setValidationErrors((prev: any) => ({
                                            ...prev,
                                            value: ''
                                        }));
                                }}
                                placeholder="Select Tag..."
                                searchable
                                error={!!props.search.validationErrors().value}
                                errorMessage={props.search.validationErrors().value}
                            />
                        </div>
                    </Show>
                    <Show when={props.search.selectedField()?.type === 'folder'}>
                        <div class="folder-select-placeholder">
                            <Select
                                options={hierarchicalFolders()}
                                value={String(props.search.currentValue() || '')}
                                onValueChange={val => {
                                    props.search.setCurrentValue(Number(val));
                                    if (props.search.validationErrors().value)
                                        props.search.setValidationErrors((prev: any) => ({
                                            ...prev,
                                            value: ''
                                        }));
                                }}
                                placeholder="Select Folder..."
                                searchable
                                error={!!props.search.validationErrors().value}
                                errorMessage={props.search.validationErrors().value}
                            />
                        </div>
                    </Show>
                    <Show when={props.search.selectedField()?.type === 'rating'}>
                        <Select
                            options={[0, 1, 2, 3, 4, 5].map(v => ({
                                value: String(v),
                                label: `${v} Stars`
                            }))}
                            value={String(props.search.currentValue() ?? '0')}
                            onValueChange={val => {
                                props.search.setCurrentValue(Number(val));
                                if (props.search.validationErrors().value)
                                    props.search.setValidationErrors((prev: any) => ({
                                        ...prev,
                                        value: ''
                                    }));
                            }}
                            error={!!props.search.validationErrors().value}
                            errorMessage={props.search.validationErrors().value}
                        />
                    </Show>
                    <Show when={props.search.selectedField()?.type === 'select'}>
                        <Select
                            options={supportedFormats().flatMap(f =>
                                f.extensions.map(ext => ({
                                    value: ext,
                                    label: `${ext.toUpperCase()} - ${f.name}`
                                }))
                            )}
                            value={(props.search.currentValue() as string) || ''}
                            onValueChange={val => {
                                props.search.setCurrentValue(val);
                                if (props.search.validationErrors().value)
                                    props.search.setValidationErrors((prev: any) => ({
                                        ...prev,
                                        value: ''
                                    }));
                            }}
                            searchable
                            error={!!props.search.validationErrors().value}
                            errorMessage={props.search.validationErrors().value}
                        />
                    </Show>
                </div>

                <div class="builder-actions">
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={props.search.handleAddCriteria}
                        class="add-button"
                    >
                        <Plus />
                    </Button>
                </div>
            </div>
        </div>
    );
};
