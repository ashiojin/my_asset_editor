<script setup lang="ts">
import { computed, inject } from 'vue';
import { MaskTargetListKey } from './GraphConsts';

const target_list = inject(MaskTargetListKey)

if (target_list === undefined) {
    throw new Error('MaskTargetListKey is not provided')
}

const selected = defineModel<string[]>({ required: true })

const options = computed(
    () => target_list.value.map(e => e.target)
)

function print_options() {
    console.log(options)
    console.log(target_list)
}

</script>

<template>
    <select v-model="selected" multiple>
        <option v-for="option in options" :value="option">{{ option }}</option>
    </select>
</template>

<style scoped>
</style>
