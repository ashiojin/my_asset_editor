<script setup lang="ts">
import { useMaskGroupListStore } from '../../../stores/MaskGroupList';

const selected = defineModel<number[]>({ required: true })

const store = useMaskGroupListStore()
</script>

<template>
    <!-- nodrag: without it Vue Flow's node drag gesture kills the mouseup on
         window before the select sees it, so no change event fires and the
         model is never written. nowheel: the listbox scrolls on its own. -->
    <select class="nodrag nowheel" v-model="selected" multiple>
        <option v-for="mask_group in store.mask_group_list" :key="mask_group.id" :value="mask_group.id">
            {{ mask_group.name }}
        </option>
    </select>
</template>

<style scoped>
</style>
