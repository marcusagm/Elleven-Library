import { Component, For, Show } from 'solid-js';
import { AssetMetadataViewProperties, getFieldRenderValue } from './AssetCardOverlay';

/**
 * AssetCardStacked - Renders the metadata immediately placed below the thumbnail.
 *
 * @param {AssetMetadataViewProperties} properties - Input data
 * @returns {JSX.Element} Structural component
 */
export const AssetCardStacked: Component<AssetMetadataViewProperties> = properties => {
    return (
        <div class="item-stacked-info">
            <For each={properties.visibleFields}>
                {field => (
                    <Show when={getFieldRenderValue(field, properties.item)}>
                        {value => <div class={`item-metadata-field field-${field}`}>{value()}</div>}
                    </Show>
                )}
            </For>
        </div>
    );
};
