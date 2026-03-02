import { Component, JSX, createEffect } from 'solid-js';
import { Thumbnail } from './Thumbnail';
import { assetDragSource } from '../../../core/dnd';
import { useAssetDropZone } from '../../../core/hooks/useAssetDropZone';

/**
 * AssetCard Props - Pure Component Interface
 *
 * This component receives ALL data via props, making it suitable for
 * virtualization where items may be recycled.
 */
export interface AssetCardProps {
    // Identity
    id: number;
    filename: string;
    path: string;

    // Display
    thumbnailPath: string | null;
    width: number | null;
    height: number | null;

    // State (controlled externally)
    isSelected: boolean;
    isFocused?: boolean;
    style: JSX.CSSProperties;
    className?: string;

    // Callbacks (lifted to parent)
    onSelect: (id: number, modifiers: { multi: boolean; shift: boolean }) => void;
    onOpen: (id: number) => void;
    onContextMenu?: (e: MouseEvent, id: number) => void;

    // DnD Support - simplified params for drag source only
    getSelectedIds: () => (number | string)[];
    getItemInfo: (id: number) => { path: string; thumbnail_path: string | null } | undefined;
}

// Register directive for this file
void assetDragSource;

/**
 * AssetCard - Pure Presentational Component
 *
 * Displays a single asset card with thumbnail. This component has NO internal
 * hooks - all state and actions come from props.
 *
 * DnD: Uses assetDragSource for dragging assets.
 * Also accepts drops from tags (Tag-to-Asset drop).
 */
export const AssetCard: Component<AssetCardProps> = props => {
    let ref: HTMLDivElement | undefined;
    const { isDropTarget, dragHandlers } = useAssetDropZone(() => props.id);

    // Sync native focus when virtual focus changes
    createEffect(() => {
        if (props.isFocused && ref) {
            ref.focus({ preventScroll: true });
        }
    });

    return (
        <div
            ref={ref}
            use:assetDragSource={{
                id: props.id,
                path: props.path,
                thumbnailPath: props.thumbnailPath,
                isSelected: props.isSelected,
                getSelectedIds: props.getSelectedIds,
                getItemInfo: props.getItemInfo
            }}
            class={`virtual-item virtual-masonry-item ${props.isSelected ? 'selected' : ''} ${props.isFocused ? 'focused' : ''} ${isDropTarget() ? 'drop-target-active' : ''} ${props.className || ''}`}
            style={props.style}
            // Accessibility
            role="gridcell"
            aria-selected={props.isSelected}
            aria-label={`Asset: ${props.filename}`}
            tabIndex={props.isFocused ? 0 : -1}
            // Events
            onClick={e => {
                e.stopPropagation();
                props.onSelect(props.id, {
                    multi: e.metaKey || e.ctrlKey,
                    shift: e.shiftKey
                });
            }}
            onDblClick={() => props.onOpen(props.id)}
            onContextMenu={e => props.onContextMenu?.(e, props.id)}
            // Drop handlers for Tag-to-Asset
            {...dragHandlers}
        >
            <div style={{ width: '100%', height: '100%', 'pointer-events': 'none' }}>
                <Thumbnail
                    id={props.id}
                    src={props.path}
                    thumbnail={props.thumbnailPath}
                    alt={props.filename}
                    width={props.width}
                    height={props.height}
                />

                <div class="item-overlay">
                    <span class="item-name">
                        #{props.id} - {props.filename}
                    </span>
                </div>
            </div>
        </div>
    );
};
