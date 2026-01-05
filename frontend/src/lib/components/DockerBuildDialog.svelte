<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
        DialogFooter,
    } from "$lib/components/ui/dialog";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";
    import { Loader2 } from "lucide-svelte";

    let {
        open = $bindable(false),
        imageName,
        path,
        onSuccess,
    } = $props<{
        open?: boolean;
        imageName: string;
        path: string;
        onSuccess?: () => void;
    }>();

    let version = $state("1.0.0");
    let loading = $state(false);
    let tags = $state<string[]>([]);
    let fetchingTags = $state(false);

    $effect(() => {
        if (open && imageName) {
            fetchTags();
        }
    });

    async function fetchTags() {
        fetchingTags = true;
        try {
            const res = await fetch(
                `http://localhost:3000/api/docker/tags?image=${imageName}`,
            );
            if (res.ok) {
                const data: any[] = await res.json();
                // data is DockerImage[]
                const allTags = data.flatMap((d) => d.tags);
                // Extract versions "verba-v2:1.0.0" -> "1.0.0"
                const versions = allTags
                    .map((t) => {
                        const parts = t.split(":");
                        return parts.length > 1 ? parts.pop() : "";
                    })
                    .filter((v) => v && v !== "latest");

                tags = Array.from(new Set(versions)) as string[];

                // Calculate next version
                if (tags.length > 0) {
                    try {
                        // Simple semver sort
                        const sorted = tags.sort((a, b) => {
                            const pa = a.split(".").map(Number);
                            const pb = b.split(".").map(Number);
                            for (let i = 0; i < 3; i++) {
                                if ((pa[i] || 0) > (pb[i] || 0)) return -1;
                                if ((pa[i] || 0) < (pb[i] || 0)) return 1;
                            }
                            return 0;
                        });

                        const latest = sorted[0];
                        const parts = latest.split(".").map(Number);
                        if (parts.length >= 3 && !parts.some(isNaN)) {
                            parts[2]++; // Increment patch
                            version = parts.join(".");
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

    async function handleBuild() {
        if (!version) {
            toast.error($t("docker.build.version_required"));
            return;
        }
        // Check exact tag match (image:version)
        // We need to re-fetch or use parsed tags.
        // If we only store "1.0.0" in tags, we can check that.
        if (tags.includes(version)) {
            toast.error($t("docker.build.version_exists", { version }));
            return;
        }

        loading = true;
        try {
            const res = await fetch("http://localhost:3000/api/docker/build", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    path,
                    image_name: imageName,
                    version,
                }),
            });

            if (res.ok) {
                const output = await res.text();
                // console.log(output);
                toast.success(
                    $t("docker.build.success", { image: imageName, version }),
                );
                open = false;
                onSuccess?.();
            } else {
                const err = await res.text();
                toast.error($t("docker.build.failed", { error: err }));
            }
        } catch (e) {
            console.error(e);
            toast.error($t("docker.build.request_failed"));
        } finally {
            loading = false;
        }
    }
</script>

<Dialog bind:open>
    <DialogContent class="sm:max-w-[500px]">
        <DialogHeader>
            <DialogTitle>{$t("docker.build.title")}</DialogTitle>
        </DialogHeader>
        <div class="grid gap-4 py-4">
            <div class="flex flex-col gap-2">
                <span class="font-semibold"
                    >{$t("docker.build.image_label")}: {imageName}</span
                >
                <span class="text-sm text-muted-foreground"
                    >{$t("docker.build.path_label")}: {path}</span
                >
            </div>

            <div class="grid grid-cols-4 items-center gap-4">
                <span class="text-right"
                    >{$t("docker.build.version_label")}</span
                >
                <Input
                    bind:value={version}
                    placeholder="1.0.0"
                    class="col-span-3"
                />
            </div>

            {#if fetchingTags}
                <div class="text-sm text-muted-foreground">
                    {$t("docker.build.checking_tags")}
                </div>
            {:else if tags.length > 0}
                <div class="text-sm text-muted-foreground">
                    {$t("docker.build.existing_versions")}: {tags
                        .slice(0, 5)
                        .join(", ")}{tags.length > 5 ? "..." : ""}
                </div>
            {/if}
        </div>
        <DialogFooter>
            <Button onclick={handleBuild} disabled={loading || fetchingTags}>
                {#if loading}
                    <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                    {$t("docker.build.building")}
                {:else}
                    {$t("docker.build.action")}
                {/if}
            </Button>
        </DialogFooter>
    </DialogContent>
</Dialog>
