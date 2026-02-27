import { describe, it, expect, vi, beforeEach } from 'vitest';
import { scheduler } from '../../utils/scheduler';

describe('Scheduler', () => {
    beforeEach(() => {
        // Mock requestAnimationFrame
        vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => {
            return setTimeout(() => cb(performance.now()), 0);
        });
        vi.stubGlobal('cancelAnimationFrame', (id: number) => {
            clearTimeout(id);
        });

        // Clear pending tasks by running a frame if needed or directly manipulating if exported
        // As it stands, it's encapsulated, so we'll just wait a tick.
    });

    it('should execute a scheduled task', async () => {
        const task = vi.fn();
        scheduler.schedule(task);

        expect(task).not.toHaveBeenCalled();

        // Wait for the next tick (rAF mock uses setTimeout 0)
        await new Promise(resolve => setTimeout(resolve, 10));

        expect(task).toHaveBeenCalledTimes(1);
    });

    it('should deduplicate the same task scheduled multiple times', async () => {
        const task = vi.fn();
        scheduler.schedule(task);
        scheduler.schedule(task);
        scheduler.schedule(task);

        await new Promise(resolve => setTimeout(resolve, 10));

        // Because of the Set, the same reference is only executed once per frame
        expect(task).toHaveBeenCalledTimes(1);
    });

    it('should be able to cancel a scheduled task', async () => {
        const task = vi.fn();
        scheduler.schedule(task);
        scheduler.cancel(task);

        await new Promise(resolve => setTimeout(resolve, 10));

        expect(task).not.toHaveBeenCalled();
    });

    it('should execute multiple distinct tasks in one frame', async () => {
        const task1 = vi.fn();
        const task2 = vi.fn();

        scheduler.schedule(task1);
        scheduler.schedule(task2);

        await new Promise(resolve => setTimeout(resolve, 10));

        expect(task1).toHaveBeenCalledTimes(1);
        expect(task2).toHaveBeenCalledTimes(1);
    });
});
