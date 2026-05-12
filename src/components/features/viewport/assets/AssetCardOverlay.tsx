import { Component, For, Show } from 'solid-js';
import { MetadataField } from '../../../../core/store/viewportPreferencesStore';
import { AssetItem } from '../../../../types';
import { formatFileSize, formatToDisplay } from '../../../../utils/format';

/**
 * Properties for rendering asset metadata.
 */
export interface AssetMetadataViewProperties {
    /**
     * The full asset data object
     * @type {AssetItem}
     */
    item: AssetItem;

    /**
     * Fields configured by the user to be visible
     * @type {MetadataField[]}
     */
    visibleFields: MetadataField[];
}

/**
 * Generic mapper to render each field
 *
 * @param {MetadataField} field - The string identifier of the field
 * @param {AssetItem} item - Data object
 * @returns {string} String representation of that data
 */
const fieldFormatters: Record<MetadataField, (item: AssetItem) => string> = {
    filename: item => item.filename || '---',
    extension: item => (item.format ? `.${item.format.toLowerCase()}` : '---'),
    dimensions: item => (item.width && item.height ? `${item.width} x ${item.height}` : '---'),
    size: item => formatFileSize(item.size),
    rating: item => (item.rating > 0 ? `★ ${item.rating}` : 'Unrated'),
    modified_at: item => formatToDisplay(item.modified_at),
    created_at: item => formatToDisplay(item.created_at),
    added_at: item => formatToDisplay(item.added_at),
    tags: () => 'Tags: ...' // Deferred loading
};

/**
 * Get field render value
 *
 * @param {MetadataField} field - The string identifier of the field
 * @param {AssetItem} item - Data object
 * @returns {string} String representation of that data
 */
export const getFieldRenderValue = (field: MetadataField, item: AssetItem): string => {
    return fieldFormatters[field]?.(item) || '';
};

/**
 * AssetCardOverlay - Renders the metadata overlaid on the image (on hover).
 *
 * @param {AssetMetadataViewProperties} properties - Input data
 * @returns {JSX.Element} Structural component
 */
export const AssetCardOverlay: Component<AssetMetadataViewProperties> = properties => {
    return (
        <div class="item-overlay">
            <For each={properties.visibleFields}>
                {field => (
                    <Show when={getFieldRenderValue(field, properties.item)}>
                        {value => (
                            <span class={`item-metadata-field field-${field}`}>{value()}</span>
                        )}
                    </Show>
                )}
            </For>
        </div>
    );
};
