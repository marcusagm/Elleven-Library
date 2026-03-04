import { Component, For, Show } from 'solid-js';
import { MetadataField } from '../../../../core/store/viewportPreferencesStore';
import { AssetItem } from '../../../../types';

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
 * Format bytes to readable size
 *
 * @param {number} bytes - Size in bytes
 * @returns {string} Readable size
 */
const formatSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;

    const kb = bytes / 1024;
    if (kb < 1024) return `${kb.toFixed(1)} KB`;

    const mb = kb / 1024;
    if (mb < 1024) return `${mb.toFixed(1)} MB`;

    return `${(mb / 1024).toFixed(1)} GB`;
};

/**
 * Formats a given date string to a locale-friendly format.
 *
 * @param {string} dateString - ISO date string
 * @returns {string} Formatted date
 */
const formatDate = (dateString: string): string => {
    if (!dateString) return '---';
    try {
        return new Date(dateString).toLocaleDateString();
    } catch {
        return dateString;
    }
};

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
    size: item => formatSize(item.size),
    rating: item => (item.rating > 0 ? `★ ${item.rating}` : 'Unrated'),
    modified_at: item => formatDate(item.modified_at),
    created_at: item => formatDate(item.created_at),
    added_at: item => formatDate(item.added_at),
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
                            <span class={`item-metadata-field field-${field}`}>
                                {field === 'filename' && (
                                    <span class="item-id-prefix">#{properties.item.id} - </span>
                                )}
                                {value()}
                            </span>
                        )}
                    </Show>
                )}
            </For>
        </div>
    );
};
