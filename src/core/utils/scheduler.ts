/**
 * System Scheduler Utility
 *
 * Centralizes requestAnimationFrame (rAF) and throttling for high-performance operations
 * like viewport scrolling, window resizing, and layout updates.
 *
 * This prevents multiple components from competing for frame budget and ensures
 * smooth execution of layout-heavy tasks.
 *
 * @module Scheduler
 */

type ScheduledTask = () => void;

class Scheduler {
    private rafId: number | null = null;
    private tasks: Set<ScheduledTask> = new Set();
    private isScheduled = false;

    /**
     * Schedules a task to run in the next animation frame.
     * Duplicate tasks are only stored once.
     *
     * @param task - The function to execute.
     */
    schedule(task: ScheduledTask): void {
        this.tasks.add(task);
        if (!this.isScheduled) {
            this.isScheduled = true;
            this.rafId = requestAnimationFrame(this.runTasks);
        }
    }

    /**
     * Cancels a previously scheduled task.
     *
     * @param task - The function to remove.
     */
    cancel(task: ScheduledTask): void {
        this.tasks.delete(task);
        if (this.tasks.size === 0 && this.rafId !== null) {
            cancelAnimationFrame(this.rafId);
            this.rafId = null;
            this.isScheduled = false;
        }
    }

    private runTasks = (): void => {
        this.isScheduled = false;
        this.rafId = null;

        const currentTasks = Array.from(this.tasks);
        this.tasks.clear();

        currentTasks.forEach(task => {
            try {
                task();
            } catch (error) {
                console.error('Error executing scheduled task:', error);
            }
        });
    };
}

/**
 * Global singleton instance of the System Scheduler.
 */
export const scheduler = new Scheduler();

/**
 * Debounce utility using the System Scheduler.
 * Useful for high-frequency events like resize.
 */
export function debounceFrame(task: ScheduledTask): ScheduledTask {
    let pending = false;

    return () => {
        if (pending) return;
        pending = true;
        scheduler.schedule(() => {
            task();
            pending = false;
        });
    };
}
