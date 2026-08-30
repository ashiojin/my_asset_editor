<script setup lang="ts">
import { computed } from 'vue';

import MaskGroupItem from './MaskGroupItem.vue'
import { useMaskGroupListStore } from '../stores/MaskGroupList';


let next_id = 1
const store = useMaskGroupListStore()

function add_mask_group() {
    const id = next_id++
    store.mask_group_list.push({
        id,
        name: make_unique_name_from(`mg-${id}`, id),
        targets: [],
    })
}
function remove_mask_group(id: number) {
    store.mask_group_list = store.mask_group_list.filter(mg => mg.id !== id)
}
const isMaskGroupFull = computed(() => store.mask_group_list.length >= 64)

function get_mask_group(id: number) {
    return store.mask_group_list.find(mg => mg.id === id)
}

/// Is `name` already used by a mask group other than `id`?
/// `id` is excluded so an item never collides with itself.
function is_name_taken(name: string, id: number) {
    return store.mask_group_list.some(mg => mg.id !== id && mg.name === name)
}
function make_unique_name_from(org: string, id: number) {
    let new_name = org
    let i = 1
    while (is_name_taken(new_name, id)) {
        new_name = `${org}-${i++}`
    }
    return new_name
}
function reject_if_not_unique(id: number) {
    const mask_group = get_mask_group(id)
    if (!mask_group) {
        console.error("Mask group not found")
        return
    }
    if (is_name_taken(mask_group.name, id)) {
        // reject
        mask_group.name = make_unique_name_from(mask_group.name, id)
    }
}

/// The names held by more than one mask group.
///
/// Duplicate names are derived state, so this is a computed rather than a
/// flag written back onto each item by a watcher: there is one source of
/// truth and nothing to keep in sync. It is recomputed once per mutation
/// of the list, instead of every row rescanning the list on every render.
const duplicated_names = computed(() => {
    const counts = new Map<string, number>()
    for (const mg of store.mask_group_list) {
        counts.set(mg.name, (counts.get(mg.name) ?? 0) + 1)
    }
    return new Set([...counts].filter(([, n]) => n > 1).map(([name]) => name))
})

</script>

<template>
    <div class="mask_group_manager">
        <button @click="add_mask_group" :disabled="isMaskGroupFull">add</button>
        <ul>
            <MaskGroupItem v-for="mask_group in store.mask_group_list" :key="mask_group.id"
                :group_id="mask_group.id"
                :duplicated="duplicated_names.has(mask_group.name)"
                v-model:name="mask_group.name"
                v-model:targets="mask_group.targets"
                @remove="remove_mask_group"
                @commit_name="reject_if_not_unique"></MaskGroupItem>
        </ul>
    </div>
</template>

<style scoped>
.mask_group_manager {
    background-color: #fefef0;
}

</style>
