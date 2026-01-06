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
    import DockerConfigDialog from "./DockerConfigDialog.svelte";
    import DockerBuildDialog from "./DockerBuildDialog.svelte";
    import DeploymentWizard from "./DeploymentWizard.svelte";
    import {
        pushImage,
        getDockerConfig,
        updateDockerConfig,
        getImageDeployments,
        type ImageDeployment,
    } from "$lib/api";

    import {
        File,
        Folder,
        CornerLeftUp,
        ChevronLeft,
        Settings,
        Hammer,
        Upload,
        RefreshCw,
        CheckCircle2,
        Loader2,
        Minus,
    } from "lucide-svelte";
    import { Badge } from "$lib/components/ui/badge";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";
    import clsx from "clsx";

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

    interface DockerImage {
        id: string;
        tags: string[];
        created: number;
        size: number;
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

    // Docker State
    let hasDockerfile = $state(false);
    let dockerImageName = $state("");
    let configOpen = $state(false);
    let buildOpen = $state(false);
    let images = $state<DockerImage[]>([]);
    let loadingImages = $state(false);

    // Push State
    let deployDialogOpen = $state(false);
    let imageToDeploy = $state("");

    // Deployment Status State
    let deploymentStatus = $state<Map<string, ImageDeployment[]>>(new Map());
    let loadingDeploymentStatus = $state(false);
    let deployingImageTag = $state<string | null>(null);

    function openDeployDialog(imageTag: string) {
        imageToDeploy = imageTag;
        deployDialogOpen = true;
    }

    async function handlePush(serverId: number) {
        toast.promise(pushImage(serverId, imageToDeploy), {
            loading: `Pushing image ${imageToDeploy}...`,
            success: `Image ${imageToDeploy} pushed successfully!`,
            error: (e) => `Failed to push image: ${e.message || e}`,
        });
    }

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

            // Load Docker Config
            try {
                const config = await getDockerConfig(path);
                if (config.docker_image_name) {
                    dockerImageName = config.docker_image_name;
                    loadImages();
                }
            } catch (e) {
                console.error("Failed to load docker config", e);
            }
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
                // Check if Dockerfile exists in root
                if (dirPath === path) {
                    hasDockerfile = files.some(
                        (f) => !f.is_dir && f.name === "Dockerfile",
                    );
                }
            }
        } catch (e) {
            console.error(e);
        }
    }
    async function loadImages() {
        if (!dockerImageName) return;
        loadingImages = true;
        try {
            const res = await fetch(
                `http://localhost:3000/api/docker/tags?image=${dockerImageName}`,
            );
            if (res.ok) {
                images = await res.json();
                // Load deployment status after images are loaded
                await loadDeploymentStatus();
            }
        } catch (e) {
            console.error(e);
        } finally {
            loadingImages = false;
        }
    }

    async function loadDeploymentStatus() {
        if (!dockerImageName) return;
        loadingDeploymentStatus = true;
        try {
            const deployments = await getImageDeployments(dockerImageName);
            // Group by image tag
            const statusMap = new Map<string, ImageDeployment[]>();
            for (const d of deployments) {
                const existing = statusMap.get(d.image_tag) || [];
                existing.push(d);
                statusMap.set(d.image_tag, existing);
            }
            deploymentStatus = statusMap;
        } catch (e) {
            console.error("Failed to load deployment status", e);
        } finally {
            loadingDeploymentStatus = false;
        }
    }

    function getDeploymentsForTag(tag: string): ImageDeployment[] {
        return deploymentStatus.get(tag) || [];
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

    // Save image name when it changes
    $effect(() => {
        if (path && dockerImageName) {
            updateDockerConfig(path, dockerImageName).catch((e) =>
                console.error("Failed to save config", e),
            );
        }
    });

    $effect(() => {
        // Reset currentPath when root path changes
        if (path) {
            currentPath = path;
            loadData();
        }
    });

    let activeTab = $state("files");

    onMount(() => {
        const params = new URLSearchParams(window.location.search);
        const t = params.get("tab");
        if (t && ["files", "status", "log", "images"].includes(t)) {
            activeTab = t;
        }
    });

    function handleTabChange(val: string) {
        activeTab = val;
        const url = new URL(window.location.href);
        url.searchParams.set("tab", val);
        window.history.replaceState({}, "", url);
    }
</script>

<div class="space-y-4">
    <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
            <Button variant="outline" size="icon" onclick={onback}>
                <ChevronLeft size={20} />
            </Button>
            <div class="flex flex-col">
                <h2 class="text-2xl font-bold">{getFolderName(path)}</h2>
                <span class="text-xs text-muted-foreground">{path}</span>
            </div>
        </div>
        <div class="flex items-center gap-2">
            {#if hasDockerfile}
                {#if !dockerImageName}
                    <Button
                        variant="outline"
                        size="sm"
                        onclick={() => (configOpen = true)}
                    >
                        <div class="flex items-center gap-2">
                            <span class="text-xl">🐳</span>
                            <span>{$t("docker.action.configure")}</span>
                            <Settings class="h-4 w-4" />
                        </div>
                    </Button>
                {:else}
                    <div class="flex gap-2">
                        <Button
                            variant="outline"
                            size="sm"
                            onclick={() => (configOpen = true)}
                        >
                            <Settings class="h-4 w-4" />
                        </Button>
                        <Button
                            variant="default"
                            size="sm"
                            onclick={() => (buildOpen = true)}
                        >
                            <div class="flex items-center gap-2">
                                <span class="text-xl">🐳</span>
                                <span>{$t("docker.action.build")}</span>
                                <Hammer class="h-4 w-4" />
                            </div>
                        </Button>
                        <Button
                            variant="default"
                            size="sm"
                            onclick={() => {
                                imageToDeploy = "";
                                deployDialogOpen = true;
                            }}
                        >
                            <div class="flex items-center gap-2">
                                <span class="text-xl">🚀</span>
                                <span>{$t("docker.action.deploy")}</span>
                                <Upload class="h-4 w-4" />
                            </div>
                        </Button>
                    </div>
                {/if}
            {/if}
        </div>
    </div>

    <Tabs value={activeTab} class="w-full" onValueChange={handleTabChange}>
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
            {#if hasDockerfile && dockerImageName}
                <TabsTrigger value="images"
                    >{$t("docker.tabs.images")}</TabsTrigger
                >
            {/if}
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

        <TabsContent value="images">
            <Card>
                <CardHeader>
                    <div class="flex items-center justify-between">
                        <CardTitle>{$t("docker.images_panel.title")}</CardTitle>
                        <Button variant="outline" size="sm" onclick={loadImages}
                            >{$t("docker.images_panel.refresh")}</Button
                        >
                    </div>
                </CardHeader>
                <CardContent>
                    {#if loadingImages}
                        <div>Loading...</div>
                    {:else if images.length === 0}
                        <div class="text-muted-foreground">
                            {$t("docker.images_panel.no_images_found", {
                                name: dockerImageName,
                            })}
                        </div>
                    {:else}
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead
                                        >{$t(
                                            "docker.images_panel.table.tag",
                                        )}</TableHead
                                    >
                                    <TableHead
                                        >{$t(
                                            "docker.images_panel.table.id",
                                        )}</TableHead
                                    >
                                    <TableHead
                                        >{$t(
                                            "docker.images_panel.table.size",
                                        )}</TableHead
                                    >
                                    <TableHead
                                        >{$t(
                                            "docker.images_panel.table.created",
                                        )}</TableHead
                                    >
                                    <TableHead
                                        >{$t(
                                            "docker.images_panel.table.status",
                                        )}</TableHead
                                    >
                                    <TableHead
                                        >{$t("common.actions", {
                                            default: "Actions",
                                        })}</TableHead
                                    >
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {#each images as image}
                                    <TableRow>
                                        <TableCell class="font-medium">
                                            <div class="flex flex-wrap gap-1">
                                                {#each image.tags as tag}
                                                    {@const version = tag
                                                        .split(":")
                                                        .pop()}
                                                    <Badge
                                                        variant="outline"
                                                        class={version ===
                                                        "latest"
                                                            ? "bg-green-100 text-green-800 hover:bg-green-200 border-green-200"
                                                            : ""}
                                                    >
                                                        {version}
                                                    </Badge>
                                                {/each}
                                            </div>
                                        </TableCell>
                                        <TableCell class="font-mono text-xs"
                                            >{image.id}</TableCell
                                        >
                                        <TableCell
                                            >{(
                                                image.size /
                                                1024 /
                                                1024
                                            ).toFixed(2)} MB</TableCell
                                        >
                                        <TableCell
                                            >{new Date(
                                                image.created * 1000,
                                            ).toLocaleString()}</TableCell
                                        >
                                        <TableCell>
                                            {@const primaryTag =
                                                image.tags[0] || ""}
                                            {@const deploys =
                                                getDeploymentsForTag(
                                                    primaryTag,
                                                )}
                                            {@const isDeploying =
                                                deployingImageTag ===
                                                primaryTag}
                                            {#if isDeploying || loadingDeploymentStatus}
                                                <div
                                                    class="flex items-center justify-center"
                                                    title={$t(
                                                        "docker.images_panel.deploying",
                                                    )}
                                                >
                                                    <Loader2
                                                        class="h-4 w-4 animate-spin text-blue-500"
                                                    />
                                                </div>
                                            {:else if deploys.length > 0}
                                                <div
                                                    class="flex items-center justify-center cursor-help"
                                                    title={`${$t("docker.images_panel.deployed_to")} ${deploys.map((d) => `${d.server_name} (${d.compose_name})`).join(", ")}`}
                                                >
                                                    <CheckCircle2
                                                        class="h-4 w-4 text-green-500"
                                                    />
                                                </div>
                                            {:else}
                                                <div
                                                    class="flex items-center justify-center"
                                                    title={$t(
                                                        "docker.images_panel.not_deployed",
                                                    )}
                                                >
                                                    <Minus
                                                        class="h-4 w-4 text-gray-400"
                                                    />
                                                </div>
                                            {/if}
                                        </TableCell>
                                        <TableCell>
                                            {#if image.tags.length > 0}
                                                <Button
                                                    variant="ghost"
                                                    size="icon"
                                                    title="Deploy to Server"
                                                    onclick={() =>
                                                        openDeployDialog(
                                                            image.tags[0],
                                                        )}
                                                >
                                                    <Upload class="h-4 w-4" />
                                                </Button>
                                            {/if}
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

    <DockerConfigDialog
        bind:open={configOpen}
        bind:imageName={dockerImageName}
    />
    <DockerBuildDialog
        bind:open={buildOpen}
        imageName={dockerImageName}
        {path}
        onSuccess={loadImages}
    />
    <DeploymentWizard
        bind:open={deployDialogOpen}
        imageTag={imageToDeploy}
        existingImages={images.flatMap((i) => i.tags)}
        {path}
        repoImageName={dockerImageName}
        onDeployStart={(tag) => {
            deployingImageTag = tag;
        }}
        onDeploySuccess={() => {
            deployDialogOpen = false;
            deployingImageTag = null;
            // Refresh deployment status after successful deployment
            loadDeploymentStatus();
        }}
    />
</div>
