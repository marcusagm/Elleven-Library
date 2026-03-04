import { Component, JSX } from 'solid-js';
import { Select } from '../../../ui/Select';
import { CriterionFieldRendererProperties } from './types';
import { supportedFormats } from '../../../../core/store/systemStore';

/**
 * Renders a searchable dropdown selector for predefined set of options (currently used for file extensions).
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the selection field renderer.
 * @returns {JSX.Element} The rendered select component.
 */
export const SelectCriterionField: Component<CriterionFieldRendererProperties> = (
    properties: CriterionFieldRendererProperties
): JSX.Element => {
    /**
     * Maps available file formats and their extensions into a list of selectable options.
     *
     * @returns {Array<{ value: string; label: string }>} The list of selectable options.
     */
    const extensions = (): Array<{ value: string; label: string }> =>
        supportedFormats().flatMap(format =>
            format.extensions.map(extension => ({
                value: extension,
                label: `.${extension.toUpperCase()} (${format.name})`
            }))
        );

    return (
        <Select
            size={properties.size || 'md'}
            options={extensions()}
            value={String(properties.value || '')}
            onValueChange={value => properties.setValue(value)}
            searchable
            error={!!properties.errors.value}
            errorMessage={properties.errors.value}
        />
    );
};
