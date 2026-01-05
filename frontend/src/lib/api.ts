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
