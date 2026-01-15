<script lang="ts">
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
    } from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Label } from "$lib/components/ui/label";
    import * as Select from "$lib/components/ui/select";
    import { Input } from "$lib/components/ui/input";
    import {
        listServers,
        listComposes,
        pushImage,
        getComposeContent,
        updateComposeContent,
        operateCompose,
        updateDockerConfig,
    } from "$lib/api";
    import {
        Loader2,
        ServerIcon,
        FileText,
        CheckCircle2,
        AlertCircle,
        Hammer,
    } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import clsx from "clsx";
    import { t } from "svelte-i18n";
    import { RadioGroup, RadioGroupItem } from "$lib/components/ui/radio-group";

    // Helper to get i18n value or fallback
    const i18n = (
        key: string,
        fallback: string,
        values?: Record<string, string | number>,
    ) => {
        const result = $t(key, { values, default: fallback });
        return result || fallback;
    };

    let {
        open = $bindable(false),
        imageTag,
        onDeploySuccess,
        onDeployStart,
        existingImages = [], // List of strings (tags)
        path,
        repoImageName = "",
        defaultServerId,
        defaultComposePath,
    }: {
        open: boolean;
        imageTag?: string; // If provided, pre-selects "Existing Image" with this tag
        onDeploySuccess?: () => void;
        onDeployStart?: (imageTag: string) => void;
        existingImages?: string[];
        path?: string;
        repoImageName?: string;
        defaultServerId?: number;
        defaultComposePath?: string;
    } = $props();

    // Steps: 1=Source, 2=Target, 3=Deploying, 4=Done
    let step = $state(1);

    // Step 1: Source
    let sourceType = $state<"build" | "existing">("existing");
    let selectedImageTag = $state("");

    // Build specific
    let buildVersion = $state("1.0.0");
    let buildTags = $state<string[]>([]);
    let building = $state(false);
    let fetchingTags = $state(false);
    let buildSuccessTag = $state("");

    async function fetchTags() {
        if (!repoImageName) return;
        fetchingTags = true;
        try {
            const res = await fetch(
                `http://localhost:3000/api/docker/tags?image=${repoImageName}`,
            );
            if (res.ok) {
                const data: any[] = await res.json();
                const allTags = data.flatMap((d) => d.tags);
                const versions = allTags
                    .map((t) => {
                        const parts = t.split(":");
                        return parts.length > 1 ? parts.pop() : "";
                    })
                    .filter((v) => v && v !== "latest");

                const uniqueVersions = Array.from(new Set(versions));
                uniqueVersions.sort((a, b) => {
                    const parse = (v: any) =>
                        v
                            .toString()
                            .split(".")
                            .map((p: any) => parseInt(p, 10));
                    const pa = parse(a);
                    const pb = parse(b);
                    for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
                        const na = pa[i] !== undefined ? pa[i] : 0;
                        const nb = pb[i] !== undefined ? pb[i] : 0;
                        if (isNaN(na) || isNaN(nb)) return b.localeCompare(a);
                        if (na > nb) return -1;
                        if (na < nb) return 1;
                    }
                    return 0;
                });

                buildTags = uniqueVersions as string[];
                // Calc next version
                if (buildTags.length > 0) {
                    try {
                        const latest = buildTags[0];
                        const parts = latest.split(".").map(Number);
                        if (parts.length >= 3 && !parts.some(isNaN)) {
                            parts[2]++; // Increment patch
                            buildVersion = parts.join(".");
                        }
                    } catch (e) {
                        console.error("Error calculating next version", e);
                    }
                }
            }
        } catch (e) {
            console.error(e);
        } finally {
            fetchingTags = false;
        }
    }

    $effect(() => {
        if (open && sourceType === "build") {
            fetchTags();
        }
    });

    async function handleInlineBuild() {
        if (!buildVersion) {
            toast.error(i18n("deploy.version_required", "Version required"));
            return;
        }
        if (buildTags.includes(buildVersion)) {
            toast.error(
                i18n(
                    "deploy.version_exists",
                    `Version ${buildVersion} already exists`,
                    { version: buildVersion },
                ),
            );
            return;
        }

        building = true;

        const buildTask = async () => {
            const res = await fetch("http://localhost:3000/api/docker/build", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    path,
                    image_name: repoImageName,
                    version: buildVersion,
                }),
            });

            if (!res.ok) {
                const err = await res.text();
                throw new Error(err);
            }
            return res;
        };

        const promise = buildTask();

        toast.promise(promise, {
            loading: i18n(
                "deploy.building",
                `Building ${repoImageName}:${buildVersion}...`,
                { image: repoImageName, version: buildVersion },
            ),
            success: () => {
                const newTag = `${repoImageName}:${buildVersion}`;
                return i18n(
                    "deploy.build_success",
                    `Built ${newTag} successfully`,
                    {
                        tag: newTag,
                    },
                );
            },
            error: (e: any) =>
                i18n("deploy.build_failed", "Build failed") +
                ": " +
                (e.message || e),
        });

        try {
            await promise;
            // Success
            const newTag = `${repoImageName}:${buildVersion}`;
            buildSuccessTag = newTag;
            selectedImageTag = newTag;
            return true;
        } catch (e: any) {
            console.error(e);
            return false;
        } finally {
            building = false;
        }
    }

    async function handleNextStep() {
        // Check for defaults
        const hasDefaults = defaultServerId && defaultComposePath;

        if (sourceType === "build") {
            // Validate version first
            if (!buildVersion) {
                toast.error(
                    i18n("deploy.version_required", "Version required"),
                );
                return;
            }
            if (buildTags.includes(buildVersion)) {
                toast.error(
                    i18n(
                        "deploy.version_exists",
                        `Version ${buildVersion} already exists`,
                        { version: buildVersion },
                    ),
                );
                return;
            }

            // Optimization: If defaults exist, run BUILD + DEPLOY in background
            if (hasDefaults) {
                open = false; // Close immediately

                // Run chain in background
                (async () => {
                    // 1. Build
                    const success = await handleInlineBuild();
                    if (!success) return; // Toast already handled in inlineBuild

                    // 2. Deploy
                    selectedServerId = defaultServerId!.toString();
                    selectedComposePath = defaultComposePath!;
                    startDeploy();
                })();
                return;
            }

            // No defaults: Must wait for build, then show Step 2
            const success = await handleInlineBuild();
            if (!success) return;
        }

        // Proceed to next step (Existing Image or After Build)

        // Optimization: If defaults exist, skip step 2 and deploy immediately
        if (hasDefaults) {
            selectedServerId = defaultServerId!.toString();
            selectedComposePath = defaultComposePath!;
            open = false; // Close dialog specifically requested
            startDeploy();
            return;
        }

        loadServers();
        step = 2;
    }

    // Step 2: Target

    // Step 2: Target
    let servers = $state<any[]>([]);
    let selectedServerId = $state<string>("");
    let composes = $state<any[]>([]);
    let selectedComposePath = $state("");
    let loadingTargets = $state(false);

    // Step 3: Deploying
    let deployStatus = $state<
        {
            step: number;
            key: string;
            fallback: string;
            state: "pending" | "running" | "success" | "error";
        }[]
    >([
        {
            step: 1,
            key: "deploy.status.pushing",
            fallback: "Pushing Image",
            state: "pending",
        },
        {
            step: 2,
            key: "deploy.status.reading",
            fallback: "Reading Compose",
            state: "pending",
        },
        {
            step: 3,
            key: "deploy.status.updating",
            fallback: "Updating Version",
            state: "pending",
        },
        {
            step: 4,
            key: "deploy.status.saving",
            fallback: "Saving Compose",
            state: "pending",
        },
        {
            step: 5,
            key: "deploy.status.restarting",
            fallback: "Restarting Service",
            state: "pending",
        },
    ]);
    let deployError = $state("");

    $effect(() => {
        if (open) {
            reset();
            // Apply defaults if available
            if (defaultServerId) {
                selectedServerId = defaultServerId.toString();
            }
            if (defaultComposePath) {
                selectedComposePath = defaultComposePath;
            }

            if (imageTag) {
                selectedImageTag = imageTag;
                sourceType = "existing";

                // Optimization: If defaults exist, straight to deploy
                if (defaultServerId && defaultComposePath) {
                    selectedServerId = defaultServerId.toString();
                    selectedComposePath = defaultComposePath;
                    // Need to wait for mount/render? startDeploy is async.
                    // But we need to ensure local state is set.
                    startDeploy();
                } else {
                    step = 2; // Auto skip to target if tag provided
                    loadServers();
                }
            } else {
                step = 1;
            }
        }
    });

    function reset() {
        step = 1;
        deployError = "";
        selectedServerId = "";
        selectedComposePath = "";
        composes = [];
        deployStatus.forEach((s) => (s.state = "pending"));
    }

    async function loadServers() {
        loadingTargets = true;
        try {
            const res = await listServers();
            servers = res.map((s: any) => ({
                value: s.id.toString(),
                label: `${s.name} (${s.host})`,
            }));
        } catch (e) {
            console.error(e);
            toast.error("Failed to load servers");
        } finally {
            loadingTargets = false;
        }
    }

    async function loadComposes(serverId: number) {
        loadingTargets = true;
        composes = [];
        selectedComposePath = "";
        try {
            const res = await listComposes(serverId);
            // res is array of {name, path, ...}
            composes = res.map((c: any) => ({ value: c.path, label: c.name }));
        } catch (e) {
            console.error(e);
            toast.error("Failed to load composes");
        } finally {
            loadingTargets = false;
        }
    }

    $effect(() => {
        if (selectedServerId) {
            loadComposes(parseInt(selectedServerId));
        }
    });

    async function startDeploy() {
        if (!selectedImageTag || !selectedServerId || !selectedComposePath)
            return;

        // User requested to close dialog and use toast for deployment
        open = false;

        // If dialog is open, show step 3 UI (No longer reachable if we close it above, but keeping for safety/ref logic)
        // Since we closed it, open is false.

        const serverId = parseInt(selectedServerId);

        // Auto-save config if not set
        if (
            (!defaultServerId || !defaultComposePath) &&
            path &&
            repoImageName
        ) {
            try {
                updateDockerConfig(
                    path,
                    repoImageName,
                    serverId,
                    selectedComposePath,
                )
                    .then(() => {
                        toast.success(
                            i18n(
                                "deploy.config_saved",
                                "Configuration saved as default",
                            ),
                        );
                    })
                    .catch((e) =>
                        console.error("Failed to auto-save config", e),
                    );
            } catch (e) {
                console.error(e);
            }
        }

        // Notify parent that deployment is starting
        if (onDeployStart) onDeployStart(selectedImageTag);

        const deployTask = async () => {
            // 1. Push Image
            if (open) updateStatus(1, "running");
            await pushImage(serverId, selectedImageTag);
            if (open) updateStatus(1, "success");

            // 2. Read Compose
            if (open) updateStatus(2, "running");
            const content = await getComposeContent(
                serverId,
                selectedComposePath,
            );
            if (open) updateStatus(2, "success");

            // 3. Update Version
            if (open) updateStatus(3, "running");

            const [repo, tag] = splitImageTag(selectedImageTag);
            const imageBase = repo;

            // Regex: image: (whitespace) imageBase(:tag)?
            const regex = new RegExp(
                `image:\\s+${escapeRegExp(imageBase)}(:[\\w\\.-]+)?`,
                "g",
            );

            if (!regex.test(content)) {
                throw new Error(
                    i18n(
                        "deploy.image_not_found",
                        `Could not find image "${imageBase}" in compose file.`,
                        { image: imageBase },
                    ),
                );
            }

            const newContent = content.replace(
                regex,
                `image: ${selectedImageTag}`,
            );
            if (open) updateStatus(3, "success");

            // 4. Save Compose
            if (open) updateStatus(4, "running");
            // Check compose name from list
            // If composes isn't loaded (e.g. fast skip), we might need to fetch it or rely on ID?
            // Actually `operateCompose` needs composeName.
            // If we skipped loadServers/loadComposes, `composes` is empty!
            // We must ensure composes are loaded or we fetch the name differently.
            // For now, assume loaded or fetch on fly?
            // Better: fetch single compose helper? Or listComposes again?
            let composeName = composes.find(
                (c) => c.value === selectedComposePath,
            )?.label;

            if (!composeName) {
                // Try to fetch specific if possible, or list all
                const all = await listComposes(serverId);
                composeName = all.find(
                    (c: any) => c.path === selectedComposePath,
                )?.name;
            }

            if (!composeName)
                throw new Error("Could not determine compose name");

            await updateComposeContent(
                serverId,
                composeName,
                selectedComposePath,
                newContent,
            );
            if (open) updateStatus(4, "success");

            // 5. Restart Service
            if (open) updateStatus(5, "running");
            await operateCompose(
                serverId,
                composeName,
                selectedComposePath,
                "up",
            );
            if (open) updateStatus(5, "success");

            return `Deployed ${selectedImageTag} to ${composeName}`;
        };

        const promise = deployTask();

        // If dialog is closed (or we want to show toast anyway), use toast
        // User requested toast usage.
        toast.promise(promise, {
            loading: i18n("deploy.deploying_toast", "Deploying service..."),
            success: (msg) => msg,
            error: (e: any) => {
                deployError = e.message || "Unknown error";
                if (open) {
                    const currentStep = deployStatus.find(
                        (s) => s.state === "running",
                    );
                    if (currentStep) currentStep.state = "error";
                }
                return i18n("deploy.failed", "Deployment failed: ") + e.message;
            },
        });

        try {
            await promise;
            // Done
            if (open) {
                // toast.success handled by promise above
                setTimeout(() => {
                    step = 4;
                    if (onDeploySuccess) onDeploySuccess();
                }, 1000);
            } else {
                if (onDeploySuccess) onDeploySuccess();
            }
        } catch (e: any) {
            console.error(e);
            // Error handling in toast promise
        }
    }

    function updateStatus(
        stepIdx: number,
        state: "pending" | "running" | "success" | "error",
    ) {
        const s = deployStatus.find((s) => s.step === stepIdx);
        if (s) s.state = state;
    }

    function splitImageTag(fullTag: string) {
        const lastColon = fullTag.lastIndexOf(":");
        if (lastColon === -1) return [fullTag, "latest"];
        return [
            fullTag.substring(0, lastColon),
            fullTag.substring(lastColon + 1),
        ];
    }

    function escapeRegExp(string: string) {
        return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    }

    let hasDefaults = $derived(!!(defaultServerId && defaultComposePath));
</script>

<Dialog bind:open>
    <DialogContent class="max-w-2xl">
        <DialogHeader>
            <DialogTitle>{i18n("deploy.title", "Deploy Service")}</DialogTitle>
        </DialogHeader>

        <div class="py-4 space-y-6">
            {#if step === 1}
                <div class="space-y-4">
                    <div class="space-y-2">
                        <Label class="text-base"
                            >{i18n(
                                "deploy.step1_title",
                                "1. Select Image Source",
                            )}</Label
                        >
                        <RadioGroup
                            bind:value={sourceType}
                            class="grid grid-cols-2 gap-4"
                        >
                            <div>
                                <RadioGroupItem
                                    value="existing"
                                    id="existing"
                                    class="peer sr-only"
                                />
                                <Label
                                    for="existing"
                                    class="flex flex-col items-center justify-between rounded-md border-2 border-muted bg-popover p-4 hover:bg-accent hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary"
                                >
                                    <FileText class="mb-3 h-6 w-6" />
                                    {i18n(
                                        "deploy.source.existing",
                                        "Existing Image",
                                    )}
                                </Label>
                            </div>
                            <div>
                                <RadioGroupItem
                                    value="build"
                                    id="build"
                                    class="peer sr-only"
                                />
                                <Label
                                    for="build"
                                    class="flex flex-col items-center justify-between rounded-md border-2 border-muted bg-popover p-4 hover:bg-accent hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary"
                                >
                                    <Hammer class="mb-3 h-6 w-6" />
                                    {i18n(
                                        "deploy.source.build",
                                        "Build New Image",
                                    )}
                                </Label>
                            </div>
                        </RadioGroup>
                    </div>

                    {#if sourceType === "existing"}
                        <div class="space-y-2">
                            <Label
                                >{i18n(
                                    "deploy.select_image_tag",
                                    "Select Image Tag",
                                )}</Label
                            >
                            <Select.Root
                                type="single"
                                bind:value={selectedImageTag}
                            >
                                <Select.Trigger>
                                    {selectedImageTag ||
                                        i18n(
                                            "deploy.select_image_placeholder",
                                            "Select an image...",
                                        )}
                                </Select.Trigger>
                                <Select.Content>
                                    {#each existingImages as img}
                                        <Select.Item value={img} label={img}
                                            >{img}</Select.Item
                                        >
                                    {/each}
                                </Select.Content>
                            </Select.Root>
                        </div>
                    {:else}
                        <div
                            class="p-4 border rounded-md bg-muted/50 space-y-4"
                        >
                            <div class="grid grid-cols-4 items-center gap-4">
                                <Label class="text-right"
                                    >{i18n("deploy.version", "Version")}</Label
                                >
                                <Input
                                    bind:value={buildVersion}
                                    placeholder="1.0.0"
                                    class="col-span-3"
                                />
                            </div>

                            {#if fetchingTags}
                                <div
                                    class="text-sm text-muted-foreground ml-[25%]"
                                >
                                    {i18n(
                                        "deploy.checking_tags",
                                        "Checking tags...",
                                    )}
                                </div>
                            {:else if buildTags.length > 0}
                                <div
                                    class="text-sm text-muted-foreground ml-[25%]"
                                >
                                    {i18n("deploy.existing_tags", "Existing")}: {buildTags
                                        .slice(0, 5)
                                        .join(", ")}{buildTags.length > 5
                                        ? "..."
                                        : ""}
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>
                <div class="flex justify-end gap-2">
                    <Button variant="ghost" onclick={() => (open = false)}
                        >{i18n("deploy.cancel", "Cancel")}</Button
                    >
                    <Button
                        onclick={handleNextStep}
                        disabled={(sourceType === "existing" &&
                            !selectedImageTag) ||
                            (sourceType === "build" &&
                                (!buildVersion || building || fetchingTags))}
                    >
                        {#if building}
                            <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                            {i18n("deploy.building", "Building...")}
                        {:else if hasDefaults}
                            {i18n("docker.action.deploy", "Deploy")}
                        {:else}
                            {i18n("deploy.next_target", "Next: Select Target")}
                        {/if}
                    </Button>
                </div>
            {:else if step === 2}
                <div class="space-y-4">
                    <div class="bg-muted p-3 rounded-md text-sm mb-4">
                        {i18n("deploy.deploying_image", "Deploying")}:
                        <span class="font-mono font-bold"
                            >{selectedImageTag}</span
                        >
                    </div>

                    <div class="space-y-2">
                        <Label
                            >{i18n(
                                "deploy.target_server",
                                "Target Server",
                            )}</Label
                        >
                        <Select.Root
                            type="single"
                            bind:value={selectedServerId}
                        >
                            <Select.Trigger>
                                {servers.find(
                                    (s) => s.value === selectedServerId,
                                )?.label ||
                                    i18n(
                                        "deploy.select_server",
                                        "Select Server...",
                                    )}
                            </Select.Trigger>
                            <Select.Content>
                                {#each servers as s}
                                    <Select.Item value={s.value} label={s.label}
                                        >{s.label}</Select.Item
                                    >
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    </div>

                    <div class="space-y-2">
                        <Label
                            >{i18n(
                                "deploy.target_compose",
                                "Target Compose File",
                            )}</Label
                        >
                        <Select.Root
                            type="single"
                            bind:value={selectedComposePath}
                            disabled={!selectedServerId || loadingTargets}
                        >
                            <Select.Trigger>
                                {composes.find(
                                    (c) => c.value === selectedComposePath,
                                )?.label ||
                                    i18n(
                                        "deploy.select_compose",
                                        "Select Compose...",
                                    )}
                            </Select.Trigger>
                            <Select.Content>
                                {#each composes as c}
                                    <Select.Item value={c.value} label={c.label}
                                        >{c.label}</Select.Item
                                    >
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    </div>
                </div>

                <div class="flex justify-end gap-2 mt-6">
                    {#if !imageTag}
                        <!-- Only allow back if not auto-started with tag -->
                        <Button variant="ghost" onclick={() => (step = 1)}
                            >{i18n("deploy.back", "Back")}</Button
                        >
                    {/if}
                    <Button
                        onclick={startDeploy}
                        disabled={!selectedServerId || !selectedComposePath}
                    >
                        {i18n("deploy.start_deploy", "Start Deployment")}
                    </Button>
                </div>
            {:else if step === 3 || step === 4}
                <div class="space-y-4">
                    {#each deployStatus as s}
                        <div class="flex items-center gap-3">
                            <div class="w-6 flex justify-center">
                                {#if s.state === "pending"}
                                    <div
                                        class="w-2 h-2 rounded-full bg-muted-foreground/30"
                                    ></div>
                                {:else if s.state === "running"}
                                    <Loader2
                                        class="h-4 w-4 animate-spin text-blue-500"
                                    />
                                {:else if s.state === "success"}
                                    <CheckCircle2
                                        class="h-4 w-4 text-green-500"
                                    />
                                {:else if s.state === "error"}
                                    <AlertCircle class="h-4 w-4 text-red-500" />
                                {/if}
                            </div>
                            <span
                                class={clsx(
                                    "text-sm",
                                    s.state === "pending" &&
                                        "text-muted-foreground",
                                    s.state === "running" &&
                                        "font-medium text-blue-600",
                                    s.state === "success" && "text-green-600",
                                    s.state === "error" && "text-red-600",
                                )}>{i18n(s.key, s.fallback)}</span
                            >
                        </div>
                    {/each}

                    {#if deployError}
                        <div
                            class="p-3 bg-red-50 text-red-600 text-sm rounded-md border border-red-200 mt-4"
                        >
                            {i18n("deploy.error", "Error")}: {deployError}
                        </div>
                    {/if}
                </div>

                <div class="flex justify-end mt-6">
                    {#if step === 4}
                        <Button onclick={() => (open = false)}
                            >{i18n("deploy.close", "Close")}</Button
                        >
                    {:else if deployError}
                        <Button variant="outline" onclick={() => (step = 2)}
                            >{i18n("deploy.retry", "Retry")}</Button
                        >
                    {/if}
                </div>
            {/if}
        </div>
    </DialogContent>
</Dialog>
