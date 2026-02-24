import { Component } from 'solid-js';
import { Input } from '../../../ui/Input';
import { CriterionFieldRendererProps } from './types';

export const TextCriterionField: Component<CriterionFieldRendererProps> = props => {
    return (
        <Input
            size={props.size || 'md'}
            value={(props.value as string) || ''}
            onInput={e => props.setValue(e.currentTarget.value)}
            placeholder="Value..."
            error={!!props.errors.value}
            errorMessage={props.errors.value}
        />
    );
};
