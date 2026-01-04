<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import {
        Dialog,
        DialogContent,
        DialogHeader,
        DialogTitle,
    } from "$lib/components/ui/dialog";
    import {
        Tabs,
        TabsContent,
        TabsList,
        TabsTrigger,
    } from "$lib/components/ui/tabs";
    import DirectoryPicker from "./DirectoryPicker.svelte";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";

    let { open = $bindable(false), onadded } = $props<{
        open?: boolean;
        onadded?: () => void;
    }>();
    let path = $state("");
    let pickerOpen = $state(false);
    let scanResult = $state<string[]>([]);
    let scanning = $state(false);

    async function addDirectory(pathToAdd: string) {
        if (!pathToAdd) return;

        try {
            const response = await fetch(
                "http://localhost:3000/api/directories",
                {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ path: pathToAdd }),
                },
            );

            if (response.ok) {
                onadded?.(); // Notify parent to refresh
            } else {
                console.error("Failed to add directory: " + pathToAdd);
                toast.error($t("directory.add_failed"));
            }
        } catch (e) {
            console.error(e);
            toast.error($t("directory.add_error"));
        }
    }

    async function handleManualAdd() {
        await addDirectory(path);
        path = "";
        open = false; // Close dialog
    }

    async function handleScan(rootPath: string) {
        pickerOpen = false;
        scanning = true;
        try {
            const res = await fetch("http://localhost:3000/api/fs/scan", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ path: rootPath }),
            });
            if (res.ok) {
                const repos = await res.json();
                scanResult = repos;

                // Auto-add found repos? Or list them? For now, let's auto-add all found.
                let addedCount = 0;
                for (const repo of repos) {
                    await addDirectory(repo);
                    addedCount++;
                }
                toast.success(
                    $t("directory.scan_success", {
                        values: { count: addedCount, path: rootPath },
                    }),
                );
                open = false; // Close dialog
            }
        } catch (e) {
            console.error(e);
            toast.error($t("directory.scan_failed"));
        } finally {
            scanning = false;
        }
    }

    function togglePicker() {
        pickerOpen = !pickerOpen;
    }
</script>

<Dialog bind:open>
    <DialogContent class="sm:max-w-[600px]">
        <DialogHeader>
            <DialogTitle>{$t("directory.add_title")}</DialogTitle>
        </DialogHeader>
        <Tabs value="manual" class="w-full">
            <TabsList class="grid w-full grid-cols-2 mb-4">
                <TabsTrigger value="manual"
                    >{$t("directory.manual_input")}</TabsTrigger
                >
                <TabsTrigger value="scan"
                    >{$t("directory.scan_directory")}</TabsTrigger
                >
            </TabsList>

            <TabsContent value="manual" class="flex gap-4">
                <Input
                    type="text"
                    placeholder={$t("directory.path_placeholder")}
                    bind:value={path}
                />
                <Button onclick={handleManualAdd}
                    >{$t("directory.add_button")}</Button
                >
            </TabsContent>

            <TabsContent value="scan">
                {#if !pickerOpen && !scanning}
                    <Button
                        variant="outline"
                        class="w-full"
                        onclick={togglePicker}
                    >
                        {$t("directory.browse_button")}
                    </Button>
                {/if}

                {#if scanning}
                    <div class="text-center p-4">
                        {$t("directory.scanning")}
                    </div>
                {/if}

                {#if pickerOpen}
                    <DirectoryPicker
                        onselect={handleScan}
                        oncancel={togglePicker}
                    />
                {/if}
            </TabsContent>
        </Tabs>
    </DialogContent>
</Dialog>
