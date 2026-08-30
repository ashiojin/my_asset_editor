<script setup lang="ts">
import { Result } from '@praha/byethrow'
import { ref, computed, provide, readonly } from "vue"

import { open } from '@tauri-apps/plugin-dialog'

import { useVueFlow } from '@vue-flow/core'
import useDragAndDrop from '../composables/useDnD.js'

import { invoke } from '@tauri-apps/api/core';

import { MaskTargetListKey, type MaskTarget } from './GraphConsts.ts'

import MaskGroupManager from './MaskGroupManager.vue'

const { getNodes } = useVueFlow('animation_graph')
const { onDragStart } = useDragAndDrop()

const gltf_info = ref<any>({}) // FIXME: any!
const queued_commands = ref<GraphCommand[]>([])

const mask_target_list = ref<MaskTarget[]>([])

provide(MaskTargetListKey, readonly(mask_target_list))

async function open_gltf() {
    const file = await open({
        multiple: false,
        directory: false,
    })

    // FIXME: move Preverer API calls to api.ts
    await invoke('load_gltf', { path: file })
        .then(
            () => console.log('load_gltf', 'ok'),
        )
    .catch((e) => console.log('load_gltf', 'err', e))


    for (let retry_cnt = 0; retry_cnt < 100; retry_cnt++) {
        const ok = await invoke('get_state').then((res_get_info) => {
            console.log('fetch', retry_cnt, res_get_info)
            gltf_info.value = res_get_info
            if (gltf_info.value.gltf_info !== null) {

                if (gltf_info.value.scene_info) {
                    return true
                } else {
                    console.log('wait scene_info')
                    return false
                }

            } else {
                console.log('wait gltf_info')
                return false
            }
        })
        .catch((e) => {
            console.error('get_state error', e)
            gltf_info.value = { status: 'FATAL ERROR' }
            return false
        })

        if (ok) {
            console.log('get_state OK')
            mask_target_list.value.splice(0)
            console.log('bones - ', gltf_info.value.scene_info.bones)
            for (let bone_info of gltf_info.value.scene_info.bones) {
                console.log('add to mask_target_list', bone_info)
                mask_target_list.value.push({ target: bone_info.name, path: bone_info.path })
            }
            break
        }
    }
}

import { convertToApiGraphCommand, getCurrentGraph } from '../previewer/api.ts'

async function send_graph() {
    try {
        const graph = getCurrentGraph()
        await invoke('set_graph', { graph })

    } catch (error) {
        console.error('Fetch failed', { error })
    }
}

const clip_options = computed(() => getNodes.value
    .filter(n => n.type === 'clip').map(n => n.data.label_id)
)
const node_options = computed(() => getNodes.value
    .filter(n => n.type !== 'root').map(n => n.data.label_id)
)

import { GraphCommand, GraphCommandType, check_command, getDefault } from "./Command.ts"
function add_command(type: GraphCommandType) {
    queued_commands.value.push(getDefault(type))
}


async function send_command() {
    console.log(queued_commands.value)
    try {
        const validation_result = queued_commands.value.map(c => check_command(c))
        if (!validation_result.every(r => Result.isSuccess(r))) {
            console.error('Send Command Failed', validation_result.filter(r => Result.isFailure(r)))
            return
        }
        const command_list = queued_commands.value.map(c => convertToApiGraphCommand(c))

        await invoke('issue_graph_commands', { commands: command_list })
        queued_commands.value = []
    } catch (error) {
        console.error('Fetch failed', error)
    }
}

</script>

<template>
    <div class="container">
        <input type="button" @click="open_gltf" value="LoadGltf" />
        <input type="button" @click="send_graph" value="send" />
        <div class="node_palette">
            <div v-for="animation in gltf_info.gltf_info?.animations ?? []" class="vue-flow__node-output node_item"
                :draggable="true" @dragstart="onDragStart($event, 'clip', { weight: 1.0, clip_name: animation.name, masks: [] })">{{
                animation.name }}</div>
            <div class="vue-flow__node-default node_item" :draggable="true"
                @dragstart="onDragStart($event, 'blend', { weight: 1.0 })">Blend Node</div>
            <div class="vue-flow__node-default node_item" :draggable="true"
                @dragstart="onDragStart($event, 'additive-blend', { weight: 1.0 })">Additive Blend Node</div>
        </div>
        <div class="mask_manager">
            <MaskGroupManager></MaskGroupManager>
        </div>
        <div class="command_queue">
            <div>
                <ul>
                    <li v-for="(_command, idx) in queued_commands">
                        <div v-if="queued_commands[idx].type === GraphCommandType.PlayRepeat">
                            Play
                            <select v-model="queued_commands[idx].selected">
                                <option value="" disabled>Select a clip node</option>
                                <option v-for="option in clip_options" :key="option" :value="option">{{ option }}
                                </option>
                            </select>
                        </div>
                        <div v-else-if="queued_commands[idx].type === GraphCommandType.Stop">
                            Stop
                            <select v-model="queued_commands[idx].selected">
                                <option value="" disabled>Select a clip node</option>
                                <option v-for="option in clip_options" :key="option" :value="option">{{ option }}
                                </option>
                            </select>
                        </div>
                        <div v-else-if="queued_commands[idx].type === GraphCommandType.Weight">
                            SetWeight
                            <select v-model="queued_commands[idx].selected">
                                <option value="" disabled>Select a node</option>
                                <option v-for="option in node_options" :key="option" :value="option">{{ option }}
                                </option>
                            </select>
                            <input v-model="queued_commands[idx].weight" min="0.0" max="1.0">
                        </div>
                    </li>
                </ul>
            </div>
            <input type="button" @click="add_command(GraphCommandType.PlayRepeat)" value="Play">
            <input type="button" @click="add_command(GraphCommandType.Weight)" value="Weight">
            <input type="button" @click="add_command(GraphCommandType.Stop)" value="Stop">
            <input type="button" @click="send_command" value="Send">
        </div>
    </div>
</template>

<style scoped>
div.container {
    background: #e0f080;
    border: 1px solid;
    margin: 1em;
    padding: 2px;
}

div.node_palette {
    background: #ffffff;
    margin: 1px;
}

div.node_item {
    margin: 3px;
}
</style>
