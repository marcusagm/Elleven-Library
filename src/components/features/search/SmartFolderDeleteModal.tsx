import { ConfirmModal } from '../../ui';
import { Component } from 'solid-js';
import { SmartFolder } from '../../../core/store/metadataStore';
import { useMetadata, useNotification } from '../../../core/hooks';

/**
 * Properties for the SmartFolderDeleteModal component.
 */
interface SmartFolderDeleteModalProperties {
    /** Whether the modal is currently open. */
    isOpen: boolean;
    /** Callback invoked when the modal requests closure. */
    onClose: () => void;
    /** The smart folder metadata object to be deleted. */
    folder: SmartFolder | null;
}

/**
 * Modal dialog to confirm the deletion of a smart folder (saved search).
 *
 * @param componentProperties - Properties for the component.
 * @returns The rendered SmartFolderDeleteModal.
 */
export const SmartFolderDeleteModal: Component<
    SmartFolderDeleteModalProperties
> = componentProperties => {
    const metadata = useMetadata();
    const notification = useNotification();

    /**
     * Handles the smart folder deletion confirmation.
     */
    const handleConfirm = async () => {
        if (!componentProperties.folder) {
            return;
        }

        const folderName = componentProperties.folder.name;
        try {
            await metadata.deleteSmartFolder(componentProperties.folder.id);
            notification.success('Smart Folder Deleted', `Removed "${folderName}"`);
        } catch (error) {
            console.error('Delete failed:', error);
            notification.error('Failed to Delete Smart Folder');
        } finally {
            componentProperties.onClose();
        }
    };

    return (
        <ConfirmModal
            isOpen={componentProperties.isOpen}
            onClose={componentProperties.onClose}
            onConfirm={handleConfirm}
            title="Delete Smart Folder"
            kind="danger"
            confirmText="Delete"
            message=""
        >
            <div class="delete-confirmation-content">
                <p>
                    Are you sure you want to delete the smart folder{' '}
                    <strong>"{componentProperties.folder?.name}"</strong>?
                </p>
                <p class="delete-warning">
                    This will only remove the saved search. Your images and actual folders will not
                    be affected.
                </p>
            </div>
        </ConfirmModal>
    );
};
