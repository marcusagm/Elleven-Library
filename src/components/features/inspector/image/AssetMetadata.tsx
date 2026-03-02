import { Component } from 'solid-js';
import { Maximize2, Hash } from 'lucide-solid';
import { AccordionItem, AccordionHeader, AccordionContent } from '../../../ui';
import { type AssetItem } from '../../../../types';
import './AssetMetadata.css';

interface AssetMetadataProps {
    item: AssetItem | null;
}

export const AssetMetadata: Component<AssetMetadataProps> = props => {
    return (
        <AccordionItem value="image-details">
            <AccordionHeader title="Image Details" icon={<Maximize2 size={14} />} />
            <AccordionContent>
                <div class="inspector-grid">
                    <div class="inspector-meta-item">
                        <span class="inspector-meta-label">Dimensions</span>
                        <span class="inspector-meta-value">
                            {props.item?.width || '-'} x {props.item?.height || '-'}
                        </span>
                    </div>
                    <div class="inspector-meta-item">
                        <span class="inspector-meta-label">Megapixels</span>
                        <span class="inspector-meta-value">
                            {props.item?.width && props.item?.height
                                ? ((props.item.width * props.item.height) / 1000000).toFixed(1) +
                                  ' MP'
                                : '-'}
                        </span>
                    </div>
                    <div class="inspector-meta-item">
                        <span class="inspector-meta-label">ID</span>
                        <span class="inspector-meta-value">
                            <Hash size={10} />
                            {props.item?.id}
                        </span>
                    </div>
                </div>
            </AccordionContent>
        </AccordionItem>
    );
};
