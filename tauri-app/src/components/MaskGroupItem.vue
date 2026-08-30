<script setup lang="ts">
import { computed } from 'vue';

import MaskGroupSelect from './MaskGroupSelect.vue'


const props = defineProps<{
    /// id of the mask group this row edits, echoed back with every emit
    group_id: number,
    /// whether this row's name collides with another group in the list.
    /// The owner decides this, because only it can see the whole list.
    duplicated: boolean,
}>()

const emit = defineEmits<{
    remove: [id: number],
    /// the user finished editing the name, the owner should validate it
    commit_name: [id: number],
}>()

const name = defineModel<string>('name', { required: true })
const targets = defineModel<string[]>('targets', { required: true })

const name_classes = computed(() => ({
    'mask_group_item-name': true,
    duplicated: props.duplicated,
}))

</script>

<template>
    <li class="mask_group_item">
        <input :class="name_classes" type="text" v-model="name" @change="emit('commit_name', props.group_id)">
        <button class="mask_group_item-remove" @click="emit('remove', props.group_id)">remove</button>
        <MaskGroupSelect class="mask_group_item-selector" v-model="targets"></MaskGroupSelect>
    </li>
</template>

<style scoped>
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
