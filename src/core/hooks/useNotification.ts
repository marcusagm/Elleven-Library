import { toast } from '../../components/ui';

/**
 * Hook providing utility methods for dispatching application notifications across various levels.
 *
 * @returns {Object} Notification dispatch methods.
 */
export const useNotification = () => {
    return {
        success: (
            title: string,
            description?: string,
            action?: { label: string; onClick: () => void }
        ) => {
            toast.success(title, { description, action });
        },
        error: (title: string, description?: string) => {
            toast.error(title, { description });
        },
        info: (title: string, description?: string) => {
            toast.info(title, { description });
        },
        warning: (title: string, description?: string) => {
            toast.warning(title, { description });
        },
        dismiss: (id: string) => {
            toast.dismiss(id);
        }
    };
};
