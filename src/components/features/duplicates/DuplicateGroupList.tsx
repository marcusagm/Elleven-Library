import { Component, For, createSignal } from 'solid-js';
import { Filter } from 'lucide-solid';
import { Badge, Button } from '../../ui';
import { DropdownMenu } from '../../ui/DropdownMenu';
import { Thumbnail } from '../viewport/assets/Thumbnail';
import { DuplicateGroup } from './types';
import '../../ui/SidebarPanel/sidebar-panel.css';
import './duplicate-group-list.css';

export interface DuplicateGroupListProperties {
    groups: DuplicateGroup[];
    selectedGroupId: string | null;
    onSelectGroup: (groupId: string) => void;
}

export const DuplicateGroupList: Component<DuplicateGroupListProperties> = props => {
    const [showIgnored, setShowIgnored] = createSignal(false);

    return (
        <>
            <header class="ui-sidebar-panel-header">
                <h3 class="ui-sidebar-panel-title">Duplicate Groups</h3>
                <div class="ui-sidebar-panel-actions" role="group">
                    <DropdownMenu
                        align="end"
                        trigger={
                            <Button variant="ghost" size="icon-xs" title="Filter options">
                                <Filter size={14} />
                            </Button>
                        }
                        items={[
                            {
                                type: 'checkbox',
                                label: 'Show ignored groups',
                                checked: showIgnored(),
                                onCheckedChange: setShowIgnored
                            }
                        ]}
                    />
                </div>
            </header>
            <div class="group-list-container">
                <For each={props.groups}>
                    {group => (
                        <div
                            tabIndex={0}
                            onClick={() => props.onSelectGroup(group.id)}
                            onKeyDown={e => {
                                if (e.key === 'Enter' || e.key === ' ') {
                                    e.preventDefault();
                                    props.onSelectGroup(group.id);
                                }
                            }}
                            class={`group-list-item ${
                                props.selectedGroupId === group.id ? 'is-selected' : ''
                            }`}
                        >
                            <div class="group-list-item-header">
                                <Badge>{group.type}</Badge>
                                <span class="group-list-item-count">
                                    {group.candidates.length} files
                                </span>
                            </div>

                            <div class="group-list-item-preview">
                                <div class="group-list-item-thumbnail">
                                    <Thumbnail
                                        id={group.candidates[0].id}
                                        src={group.candidates[0].path}
                                        thumbnail={group.candidates[0].thumbnailUrl || null}
                                        alt={group.candidates[0].name}
                                        mediaType={group.candidates[0].mediaType}
                                    />
                                </div>
                                <span class="group-list-item-name">{group.candidates[0].name}</span>
                            </div>

                            <div class="group-list-item-confidence">
                                Confidence: {(group.confidence * 100).toFixed(0)}%
                            </div>
                        </div>
                    )}
                </For>
            </div>
        </>
    );
};
