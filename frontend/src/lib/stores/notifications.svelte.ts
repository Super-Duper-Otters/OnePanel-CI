export interface NotificationItem {
    id: string;
    type: 'build' | 'upload' | 'deploy';
    timestamp: number;
    duration?: number; // in milliseconds
    title: string;
    detail: string;
    status: 'success' | 'error';
    serverName?: string; // Optional for deployments
}

class NotificationStore {
    notifications = $state<NotificationItem[]>([]);

    constructor() {
        // Load from localStorage if needed, for now just in-memory
    }

    add(item: Omit<NotificationItem, "id" | "timestamp">) {
        const newItem: NotificationItem = {
            ...item,
            id: crypto.randomUUID(),
            timestamp: Date.now(),
        };
        // Add to beginning of list
        this.notifications = [newItem, ...this.notifications];
    }

    clear() {
        this.notifications = [];
    }

    get all() {
        return this.notifications;
    }
}

export const notificationStore = new NotificationStore();
