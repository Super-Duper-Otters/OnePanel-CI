import { createNotification, listNotifications, clearNotifications, type Notification } from "../api";

export interface NotificationItem extends Notification {
    // Extending API interface to match store usage if needed, but they seem identical except for casing perhaps?
    // API uses snake_case in Rust, but TS interface in api.ts uses camelCase or mixed?
    // Let's check api.ts definition again.
    // In api.ts: id, type, title, detail, status, timestamp, duration, server_name
    // In store: id, type, timestamp, duration, title, detail, status, serverName
    // We need to map serverName <-> server_name
    serverName?: string;
}

class NotificationStore {
    notifications = $state<NotificationItem[]>([]);

    constructor() {
        this.load();
    }

    async load() {
        try {
            const data = await listNotifications();
            // Map API response (server_name) to store item (serverName) if needed
            this.notifications = data.map(n => ({
                ...n,
                serverName: n.server_name
            }));
        } catch (e) {
            console.error("Failed to load notifications", e);
        }
    }

    async add(item: Omit<NotificationItem, "id" | "timestamp">) {
        const newItem: Omit<Notification, "id"> = {
            type: item.type,
            title: item.title,
            detail: item.detail,
            status: item.status,
            timestamp: Date.now(),
            duration: item.duration,
            server_name: item.serverName
        };

        // Optimistic update
        const optimisticId = crypto.randomUUID();
        const optimisticItem: NotificationItem = {
            ...item,
            id: optimisticId,
            timestamp: newItem.timestamp
        };
        this.notifications = [optimisticItem, ...this.notifications];

        try {
            await createNotification(newItem);
            // Ideally backend returns ID, but for now we just fire and forget or reload
        } catch (e) {
            console.error("Failed to save notification", e);
            // Rollback? For now just log.
        }
    }

    async clear() {
        const original = [...this.notifications];
        this.notifications = [];
        try {
            await clearNotifications();
        } catch (e) {
            console.error("Failed to clear notifications", e);
            this.notifications = original;
        }
    }

    get all() {
        return this.notifications;
    }
}

export const notificationStore = new NotificationStore();
