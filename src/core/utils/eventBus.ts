/**
 * Simple, typesafe Event Bus for domain-level communication.
 * Allows decoupling stores and components by providing a central
 * pub/sub mechanism.
 *
 * @module EventBus
 */

type EventCallback<T = unknown> = (data: T) => void;

/**
 * Registry of available domain events and their payload types.
 */
export interface DomainEvents {
    'metadata:changed': {
        type: 'tag' | 'location' | 'smart-folder' | 'stats';
        ids?: (string | number)[];
    };
    'assets:metadata-updated': {
        assetIds: string[];
        fields: string[];
    };
    'search:executed': {
        query: string;
        isAdvanced: boolean;
    };
}

class EventBus {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    private listeners: Map<keyof DomainEvents, Set<EventCallback<any>>> = new Map();

    /**
     * Subscribes to a specific domain event.
     *
     * @param event - The event name.
     * @param callback - Function to execute when the event is emitted.
     * @returns A function to unsubscribe.
     */
    on<K extends keyof DomainEvents>(
        event: K,
        callback: EventCallback<DomainEvents[K]>
    ): () => void {
        if (!this.listeners.has(event)) {
            this.listeners.set(event, new Set());
        }
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        this.listeners.get(event)!.add(callback as EventCallback<any>);

        return () => {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            this.listeners.get(event)?.delete(callback as EventCallback<any>);
        };
    }

    /**
     * Emits a domain event with the specified data.
     *
     * @param event - The event name.
     * @param data - The payload for the event.
     */
    emit<K extends keyof DomainEvents>(event: K, data: DomainEvents[K]): void {
        this.listeners.get(event)?.forEach(callback => {
            try {
                callback(data);
            } catch (error) {
                console.error(`Error in event listener for ${event}:`, error);
            }
        });
    }
}

/**
 * Global singleton instance of the Event Bus.
 */
export const eventBus = new EventBus();
