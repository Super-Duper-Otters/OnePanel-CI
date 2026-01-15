const API_BASE = "http://localhost:3000/api";

export async function listServers() {
    const res = await fetch(`${API_BASE}/servers`);
    if (!res.ok) throw new Error("Failed to fetch servers");
    return res.json();
}

export async function getServerStatus(id: number) {
    const res = await fetch(`${API_BASE}/servers/${id}/status`);
    if (!res.ok) throw new Error("Failed to fetch server status");
    return res.json();
}

export async function getServer(id: number) {
    const res = await fetch(`${API_BASE}/servers/${id}`);
    if (!res.ok) throw new Error("Failed to fetch server details");
    return res.json();
}

export interface Container {
    containerID: string;
    name: string;
    imageName: string;
    state: string; // "running", "exited", etc.
    createTime: string;
    ports: string[];
    // Add other fields as needed
}

export async function listContainers(serverId: number): Promise<Container[]> {
    const res = await fetch(`${API_BASE}/servers/${serverId}/containers`);
    if (!res.ok) throw new Error("Failed to fetch containers");
    return res.json();
}

export async function operateContainer(serverId: number, names: string[], operation: string) {
    const res = await fetch(`${API_BASE}/servers/${serverId}/containers/operate`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ names, operation }),
    });
    if (!res.ok) throw new Error("Failed to operate container");
}

export async function getContainerLogs(serverId: number, container: string) {
    // Note: logs endpoint returns text, not JSON
    const res = await fetch(`${API_BASE}/servers/${serverId}/containers/logs?container=${container}`);
    if (!res.ok) throw new Error("Failed to fetch logs");
    return res.text();
}
export async function pushImage(serverId: number, imageTag: string) {
    const res = await fetch(`${API_BASE}/deploy/image`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ serverId, imageTag }),
    });
    if (!res.ok) {
        const text = await res.text();
        throw new Error(`Failed to push image: ${text}`);
    }
}
export async function listComposes(serverId: number) {
    const res = await fetch(`${API_BASE}/servers/${serverId}/composes`);
    if (!res.ok) {
        throw new Error(await res.text());
    }
    return res.json();
}

export async function getComposeContent(serverId: number, path: string) {
    const res = await fetch(`${API_BASE}/servers/${serverId}/composes/content`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ path }),
    });
    if (!res.ok) {
        throw new Error(await res.text());
    }
    return res.text();
}

export async function updateComposeContent(serverId: number, name: string, path: string, content: string) {
    const res = await fetch(`${API_BASE}/servers/${serverId}/composes/content/update`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            name: name ?? "",
            path: path ?? "",
            content: content ?? ""
        }),
    });
    if (!res.ok) {
        throw new Error(await res.text());
    }
}

export async function listImages(serverId: number) {
    const res = await fetch(`${API_BASE}/servers/${serverId}/images`);
    if (!res.ok) {
        throw new Error(await res.text());
    }
    return res.json();
}

export async function removeImage(serverId: number, id: string, force: boolean) {
    const res = await fetch(`${API_BASE}/servers/${serverId}/images/remove`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ id, force }),
    });
    if (!res.ok) {
        throw new Error(await res.text());
    }
}

export async function operateCompose(serverId: number, name: string, path: string, operation: string) {
    const res = await fetch(`${API_BASE}/servers/${serverId}/composes/operate`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ name, path, operation }),
    });
    if (!res.ok) {
        throw new Error(await res.text());
    }
}

export async function getDockerConfig(path: string) {
    const res = await fetch(`${API_BASE}/directories/config/get`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path }),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
}

export async function pruneImages() {
    const res = await fetch(`${API_BASE}/docker/prune`, {
        method: "POST",
    });
    if (!res.ok) throw new Error(await res.text());
    return res.text();
}

export async function updateDockerConfig(
    path: string,
    docker_image_name: string,
    defaultServerId?: number,
    defaultComposePath?: string
) {
    const res = await fetch(`${API_BASE}/directories/config/update`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            path,
            docker_image_name,
            default_server_id: defaultServerId,
            default_compose_path: defaultComposePath
        }),
    });
    if (!res.ok) throw new Error(await res.text());
}

export interface ImageDeployment {
    server_id: number;
    server_name: string;
    compose_name: string;
    image_tag: string;
}

export async function getImageDeployments(imageBase: string): Promise<ImageDeployment[]> {
    const res = await fetch(`${API_BASE}/image-deployments?image_base=${encodeURIComponent(imageBase)}`);
    if (!res.ok) throw new Error(await res.text());
    return res.json();
}

export interface VersionResponse {
    version: string;
    latest_version?: string;
    update_available: boolean;
}

export async function getVersion(): Promise<VersionResponse> {
    const res = await fetch(`${API_BASE}/version`);
    if (!res.ok) throw new Error("Failed to fetch version");
    return res.json();
}


export interface Notification {
    id: string;
    type: "build" | "upload" | "deploy";
    title: string;
    detail: string;
    status: "success" | "error";
    timestamp: number;
    duration?: number;
    server_name?: string;
}

export async function listNotifications(): Promise<Notification[]> {
    const res = await fetch(`${API_BASE}/notifications`);
    if (!res.ok) throw new Error("Failed to fetch notifications");
    return res.json();
}

export async function createNotification(notification: Omit<Notification, "id">) {
    const res = await fetch(`${API_BASE}/notifications`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(notification),
    });
    if (!res.ok) throw new Error(await res.text());
}

export async function clearNotifications() {
    const res = await fetch(`${API_BASE}/notifications/clear`, {
        method: "POST",
    });
    if (!res.ok) throw new Error(await res.text());
}
