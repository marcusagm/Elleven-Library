import { Button } from '../../ui';
import { Component, createMemo } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { Plus, CircleQuestionMark } from 'lucide-solid';
import { Select } from '../../ui/Select';
import { Tooltip } from '../../ui/Tooltip';
import { SEARCH_FIELDS } from '../../../core/store/filter/constants';
import { criterionHandlerRegistry } from './fields';
import { useAdvancedSearch, SearchValue } from './useAdvancedSearch';

export interface CriteriaBuilderProps {
    search: ReturnType<typeof useAdvancedSearch>;
}

export const CriteriaBuilder: Component<CriteriaBuilderProps> = props => {
    const DynamicFieldComponent = createMemo(() => {
        const key = props.search.currentKey();
        const type = props.search.selectedField()?.type;
        const resolvedType = key === 'size' ? 'size' : type;
        const handler =
            criterionHandlerRegistry[resolvedType || 'text'] || criterionHandlerRegistry.text;
        return handler.component;
    });

    return (
        <div class="criteria-builder-section">
            <div class="section-title">
                Criteria Builder
                <Tooltip content="Choose a field, operator, and value to add new search criteria. Filter by name, tags, date, and more.">
                    <CircleQuestionMark size={12} />
                </Tooltip>
            </div>
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
                    <Dynamic
                        component={DynamicFieldComponent()}
                        fieldKey={props.search.currentKey()}
                        comparisonOperator={props.search.currentOperator()}
                        value={props.search.currentValue()}
                        setValue={(val: SearchValue) => {
                            props.search.setCurrentValue(() => val);
                            if (props.search.validationErrors().value) {
                                props.search.setValidationErrors(
                                    (prev: Record<string, string>) => ({
                                        ...prev,
                                        value: ''
                                    })
                                );
                            }
                        }}
                        value2={props.search.currentValue2()}
                        setValue2={(val: SearchValue) => {
                            props.search.setCurrentValue2(() => val);
                            if (props.search.validationErrors().value2) {
                                props.search.setValidationErrors(
                                    (prev: Record<string, string>) => ({
                                        ...prev,
                                        value2: ''
                                    })
                                );
                            }
                        }}
                        unitMultiplier={props.search.currentUnitMultiplier()}
                        setUnitMultiplier={(unit: string) => {
                            props.search.setCurrentUnitMultiplier(unit);
                            if (props.search.validationErrors().unit) {
                                props.search.setValidationErrors(
                                    (prev: Record<string, string>) => ({
                                        ...prev,
                                        unit: ''
                                    })
                                );
                            }
                        }}
                        errors={props.search.validationErrors()}
                    />
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
