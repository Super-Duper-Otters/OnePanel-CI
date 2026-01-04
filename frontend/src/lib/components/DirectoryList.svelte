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

  let { refreshTrigger = 0 } = $props<{ refreshTrigger?: number }>();

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
</script>

<Table>
  <TableHeader>
    <TableRow>
      <TableHead>Path</TableHead>
      <TableHead>Branch</TableHead>
      <TableHead>Last Update</TableHead>
      <TableHead>Status</TableHead>
      <TableHead>Action</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    {#each directories as dir}
      <TableRow>
        <TableCell class="font-medium">{dir.path}</TableCell>
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
            <Badge variant="destructive">Error</Badge>
          {:else if dir.git_status?.is_clean}
            <Badge variant="outline" class="bg-green-100 text-green-800"
              >Clean</Badge
            >
          {:else}
            <Badge variant="outline" class="bg-yellow-100 text-yellow-800"
              >Dirty</Badge
            >
          {/if}
        </TableCell>
        <TableCell>
          <Button
            variant="destructive"
            size="sm"
            onclick={() => removeDirectory(dir.path)}>Remove</Button
          >
        </TableCell>
      </TableRow>
    {/each}
  </TableBody>
</Table>
