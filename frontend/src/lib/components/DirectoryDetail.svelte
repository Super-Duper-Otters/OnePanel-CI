<script lang="ts">
    import { onMount } from "svelte";
    import { Button } from "$lib/components/ui/button";
    import {
        Card,
        CardContent,
        CardHeader,
        CardTitle,
    } from "$lib/components/ui/card";
    import {
        Table,
        TableBody,
        TableCell,
        TableHead,
        TableHeader,
        TableRow,
    } from "$lib/components/ui/table";
    import {
        Tabs,
        TabsContent,
        TabsList,
        TabsTrigger,
    } from "$lib/components/ui/tabs";
    import { ChevronLeft } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let { path, onback } = $props<{ path: string; onback: () => void }>();

    interface CommitInfo {
        hash: string;
        author: string;
        message: string;
        date: string | null;
    }

    interface FileStatus {
        path: string;
        status: string;
    }

    let commits = $state<CommitInfo[]>([]);
    let fileStatuses = $state<FileStatus[]>([]);
    let loading = $state(false);

    async function loadData() {
        loading = true;
        try {
            // Load Log
            const logRes = await fetch("http://localhost:3000/api/git/log", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ path, limit: 20 }),
            });
            if (logRes.ok) commits = await logRes.json();

            // Load Status
            const statusRes = await fetch(
                "http://localhost:3000/api/git/status",
                {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ path }),
                },
            );
            if (statusRes.ok) fileStatuses = await statusRes.json();
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadData();
    });
</script>

<div class="space-y-4">
    <div class="flex items-center gap-4">
        <Button variant="outline" size="icon" onclick={onback}>
            <ChevronLeft size={20} />
        </Button>
        <h2 class="text-2xl font-bold truncate">{path}</h2>
    </div>

    <Tabs value="status" class="w-full">
        <TabsList>
            <TabsTrigger value="status"
                >{$t("directory.detail.file_status")}</TabsTrigger
            >
            <TabsTrigger value="log"
                >{$t("directory.detail.commit_history")}</TabsTrigger
            >
        </TabsList>

        <TabsContent value="status">
            <Card>
                <CardHeader>
                    <CardTitle>{$t("directory.detail.file_status")}</CardTitle>
                </CardHeader>
                <CardContent>
                    {#if loading}
                        <div>Loading...</div>
                    {:else if fileStatuses.length === 0}
                        <div class="text-muted-foreground">
                            {$t("directory.detail.no_changes")}
                        </div>
                    {:else}
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead
                                        >{$t(
                                            "directory.detail.file",
                                        )}</TableHead
                                    >
                                    <TableHead
                                        >{$t(
                                            "directory.detail.status",
                                        )}</TableHead
                                    >
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {#each fileStatuses as status}
                                    <TableRow>
                                        <TableCell>{status.path}</TableCell>
                                        <TableCell>{status.status}</TableCell>
                                    </TableRow>
                                {/each}
                            </TableBody>
                        </Table>
                    {/if}
                </CardContent>
            </Card>
        </TabsContent>

        <TabsContent value="log">
            <Card>
                <CardHeader>
                    <CardTitle
                        >{$t("directory.detail.commit_history")}</CardTitle
                    >
                </CardHeader>
                <CardContent>
                    {#if loading}
                        <div>Loading...</div>
                    {:else}
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead
                                        >{$t(
                                            "directory.detail.hash",
                                        )}</TableHead
                                    >
                                    <TableHead
                                        >{$t(
                                            "directory.detail.message",
                                        )}</TableHead
                                    >
                                    <TableHead
                                        >{$t(
                                            "directory.detail.author",
                                        )}</TableHead
                                    >
                                    <TableHead
                                        >{$t(
                                            "directory.detail.date",
                                        )}</TableHead
                                    >
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {#each commits as commit}
                                    <TableRow>
                                        <TableCell class="font-mono text-xs"
                                            >{commit.hash.substring(
                                                0,
                                                7,
                                            )}</TableCell
                                        >
                                        <TableCell>{commit.message}</TableCell>
                                        <TableCell>{commit.author}</TableCell>
                                        <TableCell>
                                            {commit.date
                                                ? new Date(
                                                      commit.date,
                                                  ).toLocaleString()
                                                : "-"}
                                        </TableCell>
                                    </TableRow>
                                {/each}
                            </TableBody>
                        </Table>
                    {/if}
                </CardContent>
            </Card>
        </TabsContent>
    </Tabs>
</div>
