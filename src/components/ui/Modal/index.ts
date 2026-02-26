/**
 * Modal components
 *
 * @module Modal
 * @description
 * The Modal component is a specialized component for displaying modal dialogs.
 *
 * @example
 * ```tsx
 * import { Modal } from '@/components/ui';
 *
 * // Standard Modal usage
 * <Modal
 *   isOpen={componentProperties.isOpen}
 *   onClose={componentProperties.onClose}
 *   title={componentProperties.isSmartFolderMode ? 'Edit Smart Folder' : 'Advanced Search'}
 *   class="advanced-search-modal"
 *   size="xl"
 *   footer={}>
 *   <p>This is the content of the modal.</p>
 * </Modal>
 * ```
 *
 * @example
 * ```tsx
 * import { ConfirmModal } from '@/components/ui';
 *
 * // ConfirmModal usage
 * <ConfirmModal
 *   isOpen={isConfirmOpen}
 *   onClose={handleConfirmClose}
 *   onConfirm={handleConfirmAction}
 *   title="Confirm Action"
 *   message="Are you sure you want to proceed?"
 *   confirmText="Confirm"
 *   cancelText="Cancel"
 * >
 *   <p>This is the content of the modal.</p>
 * </ConfirmModal>
 * ```
 *
 * @example
 * ```tsx
 * import { PromptModal } from '@/components/ui';
 *
 * // PromptModal usage
 * <PromptModal
 *   isOpen={isPromptOpen}
 *   onClose={handlePromptClose}
 *   onConfirm={handlePromptSubmit}
 *   title="Enter Value"
 *   message="Please enter a value:"
 *   confirmText="Submit"
 *   cancelText="Cancel"
 *   defaultValue=""
 * >
 *   <p>This is the content of the modal.</p>
 * </PromptModal>
 * ```
 */
export * from './types';
export * from './ModalContext';
export * from './ModalRoot';
export * from './ModalOverlay';
export * from './ModalContent';
export * from './ModalHeader';
export * from './ModalBody';
export * from './ModalFooter';
export * from './ModalTitle';
export * from './ModalCloseButton';
export * from './Modal';
export * from './ConfirmModal';
export * from './PromptModal';
