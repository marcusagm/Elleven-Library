import { Component, createResource, Show, For } from 'solid-js';
import { type AssetItem } from '../../../../types';
import { AccordionItem, AccordionHeader, AccordionContent } from '../../../ui';
import { List, Loader2 } from 'lucide-solid';
import { useMetadata } from '../../../../core/hooks';
import './AdvancedMetadata.css';

interface AdvancedMetadataProps {
    item: AssetItem;
}

export const AdvancedMetadata: Component<AdvancedMetadataProps> = props => {
    const metadata = useMetadata();
    const [exif] = createResource(
        () => ({ id: props.item.id, path: props.item.path }),
        async ({ id, path }) => await metadata.getAssetExif(id, path)
    );

    return (
        <AccordionItem value="advanced">
            <AccordionHeader title="Advanced Data" icon={<List size={14} />} />
            <AccordionContent>
                <Show
                    when={!exif.loading}
                    fallback={
                        <div class="inspector-loading-spinner">
                            <Loader2 class="animate-spin" size={20} />
                        </div>
                    }
                >
                    <div class="inspector-field-group">
                        <Show
                            when={Object.keys(exif() || {}).length > 0}
                            fallback={<div class="inspector-no-data">No EXIF data found.</div>}
                        >
                            <div class="inspector-exif-grid">
                                <For each={Object.entries(exif() || {})}>
                                    {([key, value]) => (
                                        <div class="inspector-meta-item">
                                            <span class="inspector-meta-label">{key}</span>
                                            <span class="inspector-meta-value exif-value">
                                                {String(value)}
                                            </span>
                                        </div>
                                    )}
                                </For>
                            </div>
                        </Show>
                    </div>
                </Show>
            </AccordionContent>
        </AccordionItem>
    );
};
