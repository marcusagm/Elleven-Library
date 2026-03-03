import { Component } from 'solid-js';
import { type AssetItem } from '../../../../types';
import { CommonMetadata } from '../base/CommonMetadata';
import { AssetMetadata } from './AssetMetadata.tsx';
import { InspectorTags } from '../base/InspectorTags';
import { AdvancedMetadata } from './AdvancedMetadata.tsx';
import { ColorPaletteSection } from './ColorPaletteSection';
import { Accordion } from '../../../ui';
import './ImageInspector.css';

interface ImageInspectorProps {
    item: AssetItem;
}

export const ImageInspector: Component<ImageInspectorProps> = props => {
    return (
        <div class="inspector-content">
            <div class="inspector-preview square">
                <img
                    class="preview-image"
                    src={
                        props.item.thumbnail_path
                            ? `thumb://localhost/${encodeURIComponent(props.item.thumbnail_path.split(/[\\/]/).pop() || '')}`
                            : ''
                    }
                    alt={props.item.filename}
                />
            </div>

            <Accordion>
                <CommonMetadata item={props.item} />
                <AssetMetadata item={props.item} />
                <InspectorTags itemId={props.item.id} />
                <AdvancedMetadata item={props.item} />
                <ColorPaletteSection item={props.item} />
            </Accordion>
        </div>
    );
};
