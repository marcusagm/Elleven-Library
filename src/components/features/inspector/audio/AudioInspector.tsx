import { Component, onMount } from 'solid-js';
import { type AssetItem } from '../../../../types';
import { Accordion, AudioPlayer } from '../../../ui';
import { InspectorTags } from '../base/InspectorTags';
import { CommonMetadata } from '../base/CommonMetadata';
import { useAudioSource } from '../../../../core/hooks/useAudioSource';
import { accordionActions, accordionStoreState } from '../../../../core/store/accordionStore';
import './AudioInspector.css';

interface AudioInspectorProps {
    item: AssetItem;
}

export const AudioInspector: Component<AudioInspectorProps> = props => {
    const { audioUrl } = useAudioSource(
        () => props.item.id,
        () => props.item.path
    );

    onMount(() => {
        accordionActions.initializeAccordion('inspector_audio', ['common']);
    });

    return (
        <div class="inspector-content">
            <div class="inspector-preview audio-preview">
                <AudioPlayer
                    src={audioUrl()}
                    filePath={props.item.path}
                    variant="compact"
                    class="inspector-audio-player"
                />
            </div>

            <Accordion
                type="multiple"
                value={accordionStoreState['inspector_audio'] || []}
                onValueChange={val => accordionActions.setExpandedItems('inspector_audio', val)}
            >
                <CommonMetadata item={props.item} />
                <InspectorTags itemId={props.item.id} />
            </Accordion>
        </div>
    );
};
