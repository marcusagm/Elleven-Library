import { Component } from 'solid-js';
import { Select } from '../../../ui/Select';
import { CriterionFieldRendererProperties } from './types';
import { supportedFormats } from '../../../../core/store/systemStore';

/**
 * Renders a searchable dropdown selector for predefined set of options (currently used for file extensions).
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the selection field renderer.
 * @returns {JSX.Element} The rendered select component.
 */
export const SelectCriterionField: Component<CriterionFieldRendererProperties> = properties => {
    /** Maps available file formats and their extensions into a list of selectable options. */
    const extensions = () =>
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

/**
 * Handler implementation for selection-based search criteria (e.g., File Format).
 * Formats extensions into "EXT (Format Name)" strings for display.
 */
export const selectHandler: import('./types').SearchFieldHandler = {
    /** The visual component representing the format selector. */
    component: SelectCriterionField,

    /**
     * Validates that a selection has been made.
     *
     * @param value - The primary extension value selected.
     * @returns A record of validation error messages.
     */
    validate: value => {
        const validationErrors: Record<string, string> = {};
        if (value === null || value === '') {
            validationErrors.value = 'Value is required';
        }
        return validationErrors;
    },

    /**
     * Processes the raw extension string for query usage.
     *
     * @param value - Selected extension string.
     * @returns The unchanged value.
     */
    process: value => ({ finalValue: value }),

    /**
     * Resolves the selected extension back to its human-readable format name using metadata.
     *
     * @param extensionValue - The file extension to format.
     * @param _value2 - Unused secondary value.
     * @param _operator - Unused operator.
     * @param _unit - Unused unit.
     * @param metadata - Store metadata containing supported formats information.
     * @returns A friendly formatted string like ".PNG (Portable Network Graphics)".
     */
    formatDisplay: (extensionValue, _value2, _operator, _unit, metadata) => {
        const foundFormat = metadata?.supportedFormats?.find(supportedFormat =>
            supportedFormat.extensions.includes(String(extensionValue))
        );
        return foundFormat
            ? `.${String(extensionValue).toUpperCase()} (${foundFormat.name})`
            : String(extensionValue);
    }
};
