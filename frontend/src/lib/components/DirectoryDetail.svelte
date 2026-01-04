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
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
    } from "$lib/components/ui/dialog";
    import { File, Folder, CornerLeftUp, ChevronLeft } from "lucide-svelte";
    import { Badge } from "$lib/components/ui/badge";
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

    interface FileEntry {
        name: string;
        path: string;
        is_dir: boolean;
    }

    let commits = $state<CommitInfo[]>([]);
    let fileStatuses = $state<FileStatus[]>([]);
    let loading = $state(false);

    // File Browser State
    let currentPath = $state(path);
    let files = $state<FileEntry[]>([]);
    let viewingFile = $state<string | null>(null);
    let fileContent = $state("");
    let loadingFile = $state(false);
    let viewingFileName = $state("");

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

            // Load Files
            await loadFiles(path);
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    }

    async function loadFiles(dirPath: string) {
        currentPath = dirPath;
        try {
            const res = await fetch("http://localhost:3000/api/fs/list", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                // Use 'path' state which is the repo root
                body: JSON.stringify({ path: dirPath, root: path }),
            });
            if (res.ok) {
                files = await res.json();
            }
        } catch (e) {
            console.error(e);
        }
    }

    async function handleFileClick(file: FileEntry) {
        if (file.is_dir) {
            if (file.name === "..") {
                // Should "Up" logic be handled by list_directory return ".." or handled client side?
                // Backend returns ".." entry for parent. So just load it.
            }
            loadFiles(file.path);
        } else {
            // View file
            viewingFileName = file.name;
            viewingFile = file.path;
            loadingFile = true;
            try {
                const res = await fetch("http://localhost:3000/api/fs/read", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ path: file.path }),
                });
                if (res.ok) {
                    fileContent = await res.text();
                } else {
                    fileContent = "Error reading file";
                }
            } catch (e) {
                fileContent = "Error reading file: " + e;
            } finally {
                loadingFile = false;
            }
        }
    }

    function getFolderName(path: string) {
        const normalized = path.replace(/\\/g, "/");
        return normalized.split("/").pop() || path;
    }

    function handleDialogChange(open: boolean) {
        if (!open) viewingFile = null;
    }

    function getDisplayPath(current: string) {
        if (current === path) {
            return $t("directory.detail.root_dir");
        }
        // Get relative path
        const relative = current.replace(path, "").replace(/^[\\/]+/, "");
        return "/" + relative;
    }

    $effect(() => {
        // Reset currentPath when root path changes
        if (path) {
            currentPath = path;
            loadData();
        }
    });
</script>

<div class="space-y-4">
    <div class="flex items-center gap-4">
        <Button variant="outline" size="icon" onclick={onback}>
            <ChevronLeft size={20} />
        </Button>
        <div class="flex flex-col">
            <h2 class="text-2xl font-bold">{getFolderName(path)}</h2>
            <span class="text-xs text-muted-foreground">{path}</span>
        </div>
    </div>

    <Tabs value="files" class="w-full">
        <TabsList>
            <TabsTrigger value="files"
                >{$t("directory.detail.files") || "Files"}</TabsTrigger
            >
            <TabsTrigger value="status"
                >{$t("directory.detail.file_status")}</TabsTrigger
            >
            <TabsTrigger value="log"
                >{$t("directory.detail.commit_history")}</TabsTrigger
            >
        </TabsList>

        <TabsContent value="files">
            <Card>
                <CardHeader>
                    <div class="flex items-center justify-between">
                        <CardTitle>{getDisplayPath(currentPath)}</CardTitle>
                        {#if currentPath !== path}
                            <Button
                                variant="outline"
                                size="sm"
                                onclick={() => loadFiles(path)}
                            >
                                {$t("directory.detail.root_dir")}
                            </Button>
                        {/if}
                    </div>
                </CardHeader>
                <CardContent>
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead class="w-[50px]"></TableHead>
                                <TableHead>Name</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {#each files as file}
                                <TableRow
                                    class="cursor-pointer hover:bg-muted/50"
                                    onclick={() => handleFileClick(file)}
                                >
                                    <TableCell>
                                        {#if file.is_dir}
                                            <Folder
                                                class="h-4 w-4 text-blue-500"
                                            />
                                        {:else}
                                            <File
                                                class="h-4 w-4 text-gray-500"
                                            />
                                        {/if}
                                    </TableCell>
                                    <TableCell class="font-medium">
                                        {file.name}
                                    </TableCell>
                                </TableRow>
                            {/each}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>
        </TabsContent>

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

    <Dialog open={!!viewingFile} onOpenChange={handleDialogChange}>
        <DialogContent class="max-w-4xl max-h-[80vh] flex flex-col">
            <DialogHeader>
                <DialogTitle>{viewingFileName}</DialogTitle>
            </DialogHeader>
            <div
                class="flex-1 overflow-auto bg-muted/50 p-4 rounded-md border font-mono text-sm whitespace-pre"
            >
                {#if loadingFile}
                    Loading...
                {:else}
                    {fileContent}
                {/if}
            </div>
        </DialogContent>
    </Dialog>
</div>
