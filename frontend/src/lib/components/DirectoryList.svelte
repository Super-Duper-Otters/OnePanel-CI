<script lang="ts">
  import { onMount } from "svelte";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { t } from "svelte-i18n";
  import { Info } from "lucide-svelte";

  let { refreshTrigger = 0, onselect } = $props<{
    refreshTrigger?: number;
    onselect?: (path: string) => void;
  }>();

  interface GitStatus {
    path: string;
    branch?: string;
    last_commit_message?: string;
    last_commit_time?: string;
    is_clean: boolean;
  }

  interface DirectoryResponse {
    path: string;
    git_status?: GitStatus;
    error?: string;
  }

  let directories = $state<DirectoryResponse[]>([]);

  async function fetchDirectories() {
    try {
      const res = await fetch("http://localhost:3000/api/directories");
      directories = await res.json();
    } catch (e) {
      console.error(e);
    }
  }

  async function removeDirectory(path: string) {
    try {
      await fetch("http://localhost:3000/api/directories", {
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path }),
      });
      fetchDirectories();
    } catch (e) {
      console.error(e);
    }
  }

  // Watch for changes to refreshTrigger
  $effect(() => {
    // Access refreshTrigger to track it
    refreshTrigger;
    fetchDirectories();
  });

  function getFolderName(path: string) {
    // Handle both forward and backward slashes
    const normalized = path.replace(/\\/g, "/");
    return normalized.split("/").pop() || path;
  }
</script>

<Table>
  <TableHeader>
    <TableRow>
      <TableHead>{$t("directory.list_header.path")}</TableHead>
      <TableHead>{$t("directory.list_header.branch")}</TableHead>
      <TableHead>{$t("directory.list_header.last_update")}</TableHead>
      <TableHead>{$t("directory.list_header.status")}</TableHead>
      <TableHead>{$t("directory.list_header.action")}</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    {#each directories as dir}
      <TableRow>
        <TableCell
          class="cursor-pointer hover:bg-muted/50 transition-colors"
          onclick={() => onselect?.(dir.path)}
        >
          <div class="flex flex-col">
            <span class="font-bold text-base text-primary hover:underline">
              {getFolderName(dir.path)}
            </span>
            <span class="text-xs text-muted-foreground">
              {dir.path}
            </span>
          </div>
        </TableCell>
        <TableCell>{dir.git_status?.branch || "-"}</TableCell>
        <TableCell>
          {#if dir.git_status?.last_commit_time}
            {new Date(dir.git_status.last_commit_time).toLocaleString()}
          {:else}
            -
          {/if}
        </TableCell>
        <TableCell>
          {#if dir.error}
            <Badge variant="destructive">{$t("directory.status.error")}</Badge>
          {:else if dir.git_status?.is_clean}
            <Badge variant="outline" class="bg-green-100 text-green-800"
              >{$t("directory.status.clean")}</Badge
            >
          {:else}
            <Badge variant="outline" class="bg-yellow-100 text-yellow-800"
              >{$t("directory.status.dirty")}</Badge
            >
          {/if}
        </TableCell>
        <TableCell>
          <div class="flex gap-2">
            <Button
              variant="outline"
              size="icon"
              onclick={() => onselect?.(dir.path)}
            >
              <Info size={16} />
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onclick={() => removeDirectory(dir.path)}
              >{$t("directory.remove")}</Button
            >
          </div>
        </TableCell>
      </TableRow>
    {/each}
  </TableBody>
</Table>
