<script lang="ts">
    import { onMount } from "svelte";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
        CardDescription,
    } from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Button } from "$lib/components/ui/button";
    import { Trash2, Edit, BookText } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let { server, ondelete, onedit } = $props<{
        server: {
            id: number;
            name: string;
            host: string;
            port: number;
            api_key?: string;
        };
        ondelete: (id: number) => void;
        onedit: (server: any) => void;
    }>();

    // status structure based on updated DashboardResponse: { code, message, data: OsInfo | null }
    let status = $state<{
        code: number;
        message: string;
        data: { os: string; platform: string; platform_family: string } | null;
    } | null>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);

    async function fetchStatus() {
        loading = true;
        error = null;
        try {
            const res = await fetch(
                `http://localhost:3000/api/servers/${server.id}/status`,
            );
            if (res.ok) {
                // Backend returns DashboardResponse even on business error (if it can parse it)
                // Check if parsing failed in backend (502) or if 1Panel returned non-200 code
                const data = await res.json();
                // data matches DashboardResponse
                status = data;
                if (status?.code !== 200) {
                    error = status?.message || "Error";
                }
            } else {
                error = "Failed to reach backend";
            }
        } catch (e) {
            error = "Network error";
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        fetchStatus();
    });
</script>

<Card>
    <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
    >
        <div class="space-y-1">
            <CardTitle>{server.name}</CardTitle>
            <CardDescription>{server.host}:{server.port}</CardDescription>
        </div>
        <div class="flex gap-1">
            <Button
                variant="ghost"
                size="icon"
                title={$t("servers.swagger") || "API Docs"}
                onclick={(e) => {
                    e.stopPropagation();
                    window.open(`/api/servers/${server.id}/docs`, "_blank");
                }}
            >
                <BookText size={20} />
            </Button>
            <Button
                variant="ghost"
                size="icon"
                onclick={(e) => {
                    e.stopPropagation();
                    onedit(server);
                }}
            >
                <Edit size={20} />
            </Button>
            <Button
                variant="ghost"
                size="icon"
                class="text-destructive"
                onclick={(e) => {
                    e.stopPropagation();
                    ondelete(server.id);
                }}
            >
                <Trash2 size={20} />
            </Button>
        </div>
    </CardHeader>
    <CardContent>
        {#if loading}
            <div class="flex items-center gap-2">
                <span class="animate-pulse bg-muted h-4 w-12 rounded"></span>
                <span class="text-muted-foreground text-sm">Loading...</span>
            </div>
        {:else if error}
            <div class="flex flex-col gap-1">
                <Badge variant="destructive">Error</Badge>
                <span class="text-xs text-destructive break-all">{error}</span>
            </div>
        {:else if status && status.data}
            <div class="grid gap-1">
                <div class="flex items-center gap-2">
                    <Badge variant="outline" class="bg-green-100 text-green-800"
                        >Online</Badge
                    >
                    <span class="text-sm font-medium"
                        >{status.data.os} / {status.data.platform}</span
                    >
                </div>
                <p class="text-xs text-muted-foreground">
                    {status.data.platform_family}
                </p>
            </div>
        {/if}
    </CardContent>
</Card>
