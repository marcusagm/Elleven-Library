import { Component, createSignal, createEffect } from 'solid-js';
import { Search, Save } from 'lucide-solid';
import { Modal } from '../../ui/Modal';
import { Button } from '../../ui/Button';
import { Input } from '../../ui/Input';
import { useFilters, useMetadata } from '../../../core/hooks';
import { SearchGroup } from '../../../core/store/filterStore';
import { createId } from '../../../lib/primitives/createId';
import { useAdvancedSearch } from './useAdvancedSearch';
import { CriteriaBuilder } from './CriteriaBuilder';
import { QueryEditor } from './QueryEditor';
import './advanced-search-modal.css';

interface AdvancedSearchModalProps {
    isOpen: boolean;
    onClose: () => void;
    isSmartFolderMode?: boolean;
    initialId?: number;
    initialName?: string;
    initialQuery?: SearchGroup;
    onSave?: (name: string, query: SearchGroup, id?: number) => void;
}

export const AdvancedSearchModal: Component<AdvancedSearchModalProps> = props => {
    const filters = useFilters();
    const metadata = useMetadata();

    const [smartFolderName, setSmartFolderName] = createSignal(props.initialName || '');

    const search = useAdvancedSearch(metadata, {
        isOpen: () => props.isOpen,
        initialQuery: () => props.initialQuery
    });

    createEffect(() => {
        if (props.isOpen) {
            setSmartFolderName(props.initialName || '');
        }
    });

    const handleSearch = () => {
        const searchGroup: SearchGroup = {
            id: createId('group'),
            logicalOperator: search.matchMode(),
            items: search.criteria()
        };
        filters.setAdvancedSearch(searchGroup);
        props.onClose();
    };

    const handleSaveSmartFolder = () => {
        if (!smartFolderName().trim()) {
            return;
        }

        const searchGroup: SearchGroup = {
            id: createId('group'),
            logicalOperator: search.matchMode(),
            items: search.criteria()
        };
        props.onSave?.(smartFolderName().trim(), searchGroup, props.initialId);
        props.onClose();
    };

    return (
        <Modal
            isOpen={props.isOpen}
            onClose={props.onClose}
            title={props.isSmartFolderMode ? 'Edit Smart Folder' : 'Advanced Search'}
            class="advanced-search-modal"
            size="xl"
            footer={
                <div class="modal-footer-content">
                    <Button variant="ghost" onClick={props.onClose}>
                        Cancel
                    </Button>
                    <div style={{ flex: 1 }} />
                    {props.isSmartFolderMode ? (
                        <Button
                            onClick={handleSaveSmartFolder}
                            disabled={search.criteria().length === 0 || !smartFolderName().trim()}
                            leftIcon={<Save />}
                        >
                            Save Smart Folder
                        </Button>
                    ) : (
                        <>
                            <div class="smart-folder-creator">
                                <Input
                                    placeholder="Smart Folder Name..."
                                    value={smartFolderName()}
                                    onInput={e => setSmartFolderName(e.currentTarget.value)}
                                    size="sm"
                                    wrapperClass="smart-folder-input-wrapper"
                                />
                                <Button
                                    onClick={handleSaveSmartFolder}
                                    disabled={
                                        search.criteria().length === 0 || !smartFolderName().trim()
                                    }
                                    variant="outline"
                                    leftIcon={<Save />}
                                >
                                    Save as Smart Folder
                                </Button>
                            </div>
                            <Button
                                onClick={handleSearch}
                                disabled={search.criteria().length === 0}
                                leftIcon={<Search />}
                            >
                                Search
                            </Button>
                        </>
                    )}
                </div>
            }
        >
            <div class="advanced-search-modal-content">
                {props.isSmartFolderMode && (
                    <div class="smart-folder-name-section">
                        <label class="section-title">Smart Folder Name</label>
                        <Input
                            placeholder="My awesome compilation..."
                            value={smartFolderName()}
                            onInput={e => setSmartFolderName(e.currentTarget.value)}
                        />
                    </div>
                )}

                <CriteriaBuilder search={search} />

                <QueryEditor search={search} />
            </div>
        </Modal>
    );
};
