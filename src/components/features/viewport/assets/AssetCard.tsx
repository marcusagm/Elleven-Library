import { Component, JSX, Show } from 'solid-js';
import { Thumbnail } from './Thumbnail';
import { AssetItemContainer } from './AssetItemContainer';
import { AssetCardOverlay } from './AssetCardOverlay';
import { AssetCardStacked } from './AssetCardStacked';
import { AssetItem } from '../../../../types';
import { useViewportPreferences } from '../../../../core/hooks/useViewportPreferences';
import './asset-card.css';

/**
 * AssetCardProps - Re-defined to consume the full AssetItem
 */
export interface AssetCardProps {
    /**
     * Full asset item data
     * @type {AssetItem}
     */
    item: AssetItem;

    /**
     * State (controlled externally)
     * @type {boolean}
     */
    isSelected: boolean;
    /**
     * Whether it has keyboard focus
     * @type {boolean | undefined}
     */
    isFocused?: boolean;
    /**
     * Positioning style for virtualization
     * @type {JSX.CSSProperties}
     */
    style: JSX.CSSProperties;
    /**
     * Optional Extra Wrapper Class
     * @type {string | undefined}
     */
    className?: string;

    /**
     * Callbacks (lifted to parent)
     */
    onSelect: (id: string, modifiers: { multi: boolean; shift: boolean }) => void;
    onOpen: (id: string) => void;
    onContextMenu?: (event: MouseEvent, id: string) => void;

    /**
     * DnD Support
     */
    getSelectedIds: () => string[];
    getItemInfo: (id: string) => { path: string; thumbnail_path: string | null } | undefined;
}

/**
 * AssetCard - Main Presentational Facade Component
 *
 * Wraps the abstract drag container and renders either the Overlay
 * or Stacked variants of metadata display, based on user preferences.
 *
 * @param {AssetCardProps} props - The complete asset card parameters.
 * @returns {JSX.Element} The visual composition.
 */
export const AssetCard: Component<AssetCardProps> = (props: AssetCardProps): JSX.Element => {
    /**
     * Viewport preferences store
     */
    const preferences = useViewportPreferences();

    return (
        <AssetItemContainer
            id={props.item.id}
            path={props.item.path}
            thumbnailPath={props.item.thumbnail_path}
            filename={props.item.filename}
            isSelected={props.isSelected}
            isFocused={props.isFocused}
            style={props.style}
            class={props.className}
            onSelect={props.onSelect}
            onOpen={props.onOpen}
            onContextMenu={props.onContextMenu}
            getSelectedIds={props.getSelectedIds}
            getItemInfo={props.getItemInfo}
        >
            {() => (
                <div
                    class="asset-card-content"
                    style={{
                        width: '100%',
                        height: '100%',
                        'pointer-events': 'none',
                        display: 'flex',
                        'flex-direction': 'column'
                    }}
                >
                    <div style={{ position: 'relative', 'flex-grow': 1, 'min-height': 0 }}>
                        <Thumbnail
                            id={props.item.id}
                            src={props.item.path}
                            thumbnail={props.item.thumbnail_path}
                            alt={props.item.filename}
                            width={props.item.width}
                            height={props.item.height}
                        />

                        {/* Metadata Overlay Component */}
                        <Show when={preferences.metadataPosition === 'overlay'}>
                            <AssetCardOverlay
                                item={props.item}
                                visibleFields={preferences.visibleFields}
                            />
                        </Show>
                    </div>

                    {/* Metadata Stacked Component */}
                    <Show when={preferences.metadataPosition === 'stacked'}>
                        <AssetCardStacked
                            item={props.item}
                            visibleFields={preferences.visibleFields}
                        />
                    </Show>
                </div>
            )}
        </AssetItemContainer>
    );
};
