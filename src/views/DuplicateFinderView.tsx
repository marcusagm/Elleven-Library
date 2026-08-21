import { Component, JSX, createSignal, Show } from 'solid-js';
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '../components/ui';
import {
    DuplicateGroupList,
    DuplicateComparisonPanel,
    mockGroups
} from '../components/features/duplicates';

import './duplicate-finder-view.css';

export interface DuplicateFinderViewProperties {
    /** The title bar to render at the top */
    header: JSX.Element;
}

/**
 * The Duplicate Finder view interface.
 * Shows a list of duplicate groups on the left and group details/actions on the right.
 *
 * @param {DuplicateFinderViewProperties} props - Component properties.
 * @returns {JSX.Element} The rendered duplicate finder view.
 */
export const DuplicateFinderView: Component<DuplicateFinderViewProperties> = props => {
    const [selectedGroupId, setSelectedGroupId] = createSignal<string | null>(null);

    const handleSelectGroup = (groupId: string) => {
        setSelectedGroupId(groupId);
    };

    const selectedGroup = () => mockGroups.find(group => group.id === selectedGroupId());

    return (
        <div class="duplicate-finder">
            {props.header}
            <ResizablePanelGroup direction="horizontal" class="duplicate-finder-body">
                <ResizablePanel id="list-panel" defaultSize={30} minSize={20} maxSize={50}>
                    <div class="duplicate-finder-sidebar">
                        <DuplicateGroupList
                            groups={mockGroups}
                            selectedGroupId={selectedGroupId()}
                            onSelectGroup={handleSelectGroup}
                        />
                    </div>
                </ResizablePanel>

                <ResizableHandle />

                <ResizablePanel id="details-panel" defaultSize={70}>
                    <div class="duplicate-finder-content">
                        <Show
                            when={selectedGroup()}
                            fallback={
                                <div class="duplicate-finder-empty">
                                    <h2>Select a group from the list</h2>
                                </div>
                            }
                        >
                            {group => <DuplicateComparisonPanel group={group()} />}
                        </Show>
                    </div>
                </ResizablePanel>
            </ResizablePanelGroup>
        </div>
    );
};
