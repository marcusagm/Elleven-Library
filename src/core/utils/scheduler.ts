/**
 * System Scheduler
 *
 * Centralized requestAnimationFrame manager to prevent performance contention
 * and redundant layout thrashing across multiple components.
 */

type Task = () => void;

class Scheduler {
    private pendingTasks: Set<Task> = new Set();
    private isFrameRequested = false;

    /**
     * Schedules a task to run in the next animation frame.
     * Multiple calls with the same function will be ignored if already scheduled.
     */
    schedule(task: Task): void {
        this.pendingTasks.add(task);
        this.requestFrame();
    }

    /**
     * Cancels a scheduled task.
     */
    cancel(task: Task): void {
        this.pendingTasks.delete(task);
    }

    private requestFrame(): void {
        if (this.isFrameRequested) return;

        this.isFrameRequested = true;
        requestAnimationFrame(this.processFrame);
    }

    private processFrame = (): void => {
        this.isFrameRequested = false;

        // Copy tasks to avoid issues if tasks schedule new tasks
        const tasksToRun = Array.from(this.pendingTasks);
        this.pendingTasks.clear();

        for (const task of tasksToRun) {
            try {
                task();
            } catch (error) {
                console.error('[Scheduler] Error in task:', error);
            }
        }
    };
}

export const scheduler = new Scheduler();
