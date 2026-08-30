<script setup lang="ts">
import { computed, ref } from 'vue';

import MaskGroupSelect from './MaskGroupSelect.vue'


type MaskGroupItemData = {
    id: number,
    name: string,
    selected: string[],
}

let next_id = 1
const mask_group_list = ref<MaskGroupItemData[]>([])

function add_mask_group() {
    const id = next_id++
    mask_group_list.value.push({
        id,
        name: make_unique_name_from(`mg-${id}`, id),
        selected: [],
    })
}
function remove_mask_group(id: number) {
    mask_group_list.value = mask_group_list.value.filter(mg => mg.id !== id)
}
const isMaskGroupFull = computed(() => mask_group_list.value.length >= 64)

function get_mask_group(id: number) {
    return mask_group_list.value.find(mg => mg.id === id)
}

/// Is `name` already used by a mask group other than `id`?
/// `id` is excluded so an item never collides with itself.
function is_name_taken(name: string, id: number) {
    return mask_group_list.value.some(mg => mg.id !== id && mg.name === name)
}
function make_unique_name_from(org: string, id: number) {
    let new_name = org
    let i = 1
    while (is_name_taken(new_name, id)) {
        new_name = `${org}-${i++}`
    }
    return new_name
}
function reject_if_not_unique(e: Event, id: number) {
    const new_val = (e.target as HTMLInputElement).value
    const mask_group = get_mask_group(id)
    if (!mask_group) {
        console.error("Mask group not found")
        return
    }
    if (is_name_taken(new_val, id)) {
        // reject
        mask_group.name = make_unique_name_from(new_val, id)
    }
}
function check_not_unique_name(id: number) { // TODO: This is not good. Vue should not have to check for uniqueness on every render. Should this be done in a computed property or a watcher?
    const mask_group = get_mask_group(id)
    if (!mask_group) {
        console.error("Mask group not found")
        return false
    }
    // check the name specified by id is unique in the list
    const name = mask_group.name
    const count = mask_group_list.value.filter(mg => mg.name === name).length

    return count > 1
}

</script>

<template>
    <div class="mask_group_manager">
        <button @click="add_mask_group" :disabled="isMaskGroupFull">add</button>
        <ul>
            <li class="mask_group_item" v-for="mask_group in mask_group_list">
                <input :class="{ 'mask_group_item-name': true, duplicated: check_not_unique_name(mask_group.id) }" type="text" v-model="mask_group.name" @change="reject_if_not_unique($event, mask_group.id)">
                <button class="mask_group_item-remove" @click="remove_mask_group(mask_group.id)">remove</button>
                <MaskGroupSelect class="mask_group_item-selector" v-model="mask_group.selected"></MaskGroupSelect>
            </li>
        </ul>
    </div>
</template>

<style scoped>
.mask_group_manager {
    background-color: #fefef0;
}

.mask_group_item {
    background-color: #ffffff;
    display: grid;
    /*
       +---------------+------+
       | name          |remove|
       +---------------+------+
       | selector             |
       |                      |
       +----------------------+
    */
    grid-template-columns: 1fr auto;
    justify-content: space-between;
    align-items: center;
    border: 1px solid #ddd;
}
.mask_group_item-name {
    grid-column: 1 / 2;
    grid-row: 1 / 2;
}
.mask_group_item-remove {
    grid-column: 2 / 3;
    grid-row: 1 / 2;
}
.mask_group_item-selector {
    grid-column: 1 / 3;
    grid-row: 2 / 3;
}
.duplicated {
    border: 1px solid red;
    color: red;
}

</style>
