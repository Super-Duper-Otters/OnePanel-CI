<script lang="ts">
    import { onMount } from "svelte";
    import ServerCard from "./ServerCard.svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
        DialogTrigger,
        DialogFooter,
    } from "$lib/components/ui/dialog";
    import { Plus } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let servers = $state<
        {
            id: number;
            name: string;
            host: string;
            port: number;
            api_key?: string;
        }[]
    >([]);
    let open = $state(false);

    // Form State
    let editingId = $state<number | null>(null);
    let name = $state("");
    let host = $state("");
    let port = $state<number>(18556);
    let apiKey = $state("");
    let adding = $state(false);

    async function fetchServers() {
        try {
            const res = await fetch("http://localhost:3000/api/servers");
            if (res.ok) {
                servers = await res.json();
            }
        } catch (e) {
            console.error(e);
        }
    }

    function resetForm() {
        editingId = null;
        name = "";
        host = "";
        port = 18556; // Default to 1Panel port? Or 10000? 18556 per user demo
        apiKey = "";
    }

    export function openAddDialog() {
        resetForm();
        open = true;
    }

    function openEditDialog(server: any) {
        editingId = server.id;
        name = server.name;
        host = server.host;
        port = server.port;
        // We don't have api_key in list response usually, but if we do, use it.
        // If we don't, user might need to re-enter it or we keep it if empty?
        // For now assume re-enter if empty or handle in backend.
        // Current backend `list_servers` does NOT return api_key (good security).
        // So user MUST re-enter API key to update, OR backend handles optional update?
        // My `update_server` expects all fields. So user must re-enter key.
        apiKey = "";
        open = true;
    }

    async function saveServer() {
        adding = true;
        try {
            const method = editingId ? "PUT" : "POST";
            const url = editingId
                ? `http://localhost:3000/api/servers/${editingId}`
                : "http://localhost:3000/api/servers";

            const res = await fetch(url, {
                method,
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    name,
                    host,
                    port: Number(port),
                    api_key: apiKey,
                }),
            });
            if (res.ok) {
                open = false;
                fetchServers();
                resetForm();
            } else {
                alert("Failed to save server");
            }
        } catch (e) {
            console.error(e);
            alert("Error saving server");
        } finally {
            adding = false;
        }
    }

    async function deleteServer(id: number) {
        if (!confirm("Are you sure?")) return;
        try {
            await fetch(`http://localhost:3000/api/servers/${id}`, {
                method: "DELETE",
            });
            fetchServers();
        } catch (e) {
            console.error(e);
        }
    }

    onMount(fetchServers);
</script>

<div class="space-y-6">
    <Dialog bind:open>
        <DialogContent class="sm:max-w-[425px]">
            <DialogHeader>
                <DialogTitle
                    >{editingId
                        ? "Edit Server"
                        : $t("servers.add_server_title")}</DialogTitle
                >
            </DialogHeader>
            <div class="grid gap-4 py-4">
                <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="name" class="text-right"
                        >{$t("servers.name")}</Label
                    >
                    <Input
                        id="name"
                        bind:value={name}
                        class="col-span-3"
                        placeholder="My Server"
                    />
                </div>
                <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="host" class="text-right"
                        >{$t("settings.host")}</Label
                    >
                    <Input
                        id="host"
                        bind:value={host}
                        class="col-span-3"
                        placeholder="127.0.0.1"
                    />
                </div>
                <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="port" class="text-right"
                        >{$t("settings.port")}</Label
                    >
                    <Input
                        id="port"
                        type="number"
                        bind:value={port}
                        class="col-span-3"
                    />
                </div>
                <div class="grid grid-cols-4 items-center gap-4">
                    <Label for="apikey" class="text-right"
                        >{$t("settings.api_key")}</Label
                    >
                    <Input
                        id="apikey"
                        type="password"
                        bind:value={apiKey}
                        class="col-span-3"
                        placeholder={editingId
                            ? "Leave blank to keep unchanged (Not supported yet, re-enter)"
                            : ""}
                    />
                    <!-- Note: Current backend update requires API key. -->
                </div>
            </div>
            <DialogFooter>
                <Button onclick={saveServer} disabled={adding}>
                    {adding
                        ? "Saving..."
                        : editingId
                          ? "Update"
                          : $t("directory.add_button")}
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>

    {#if servers.length === 0}
        <div class="text-center text-muted-foreground py-10">
            {$t("servers.no_servers")}
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each servers as server}
                <ServerCard
                    {server}
                    ondelete={deleteServer}
                    onedit={openEditDialog}
                />
            {/each}
        </div>
    {/if}
</div>
