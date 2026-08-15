<script setup lang="ts">
import { ref, computed } from "vue"

import { open } from '@tauri-apps/plugin-dialog'

import { useVueFlow } from '@vue-flow/core'
import useDragAndDrop from '../composables/useDnD.js'

const { getNodes } = useVueFlow('animation_graph')
const { onDragStart } = useDragAndDrop()

const gltf_info = ref({'status': '-'})
const queued_commands = ref([])

async function open_gltf() {
    const file = await open({
        multiple: false,
        directory: false,
    })

    try {
        const res_load_gltf = await fetch('http://localhost:3000/api/load_gltf', {
            method: 'POST',
            body: JSON.stringify({ 'gltf': file }),
            headers: {
                'Content-Type': 'application/json',
            },
        })

        console.log(res_load_gltf)

        for (let retry_cnt =0; retry_cnt < 100; retry_cnt++) {
            const res_get_info = await fetch('http://localhost:3000/api/gltf_info', {
                medhod: 'GET'
            })

            if (res_get_info.ok) {
                gltf_info.value = await res_get_info.json()
                console.log(gltf_info.value)
                if (gltf_info.value.status == "loaded") {
                    break
                }
            } else {
                gltf_info.value = { 'status': 'Error: ' + res_get_info.statusText }
                break
            }
        }
    } catch (error) {
        console.error(error.message)
    }
}

import { getCurrentGraph } from '../api.ts'

function dump() {
    console.log('DUMP', getCurrentGraph())
}

async function send_graph() {
    try {
        const graph = getCurrentGraph()
        const res_set_graph = await fetch('http://localhost:3000/api/set_anim_graph', {
            method: 'POST',
            body: JSON.stringify(graph),
            headers: {
                'Content-Type': 'application/json',
            },
        })

        console.log(res_set_graph)

    } catch (error) {
        console.error(error.message)
    }
}

const clip_options = computed(() => getNodes.value
    .filter(n => n.type === 'clip').map(n => n.data.label_id)
)
const node_options = computed(() => getNodes.value
    .filter(n => n.type !== 'root').map(n => n.data.label_id)
)

let command_id = 0
function add_command(type) {
    const id = command_id++

    switch (type) {
        case "Play":
            queued_commands.value.push({ id, type, selected: '' })
        break
        case "Weight":
            queued_commands.value.push({ id, type, selected: '', weight: 1.0 })
        break
    }
}

async function send_command() {
    console.log(queued_commands.value)
    try {
        const command_list = queued_commands.value.map(c => {
            switch (c.type) {
                case "Play":
                    if (c.selected !== '') {
                        return { "PlayRepeat": c.selected }
                    } else {
                        return { "ERROR": {message: "Node not selected", data: c} }
                    }
                case "Weight":
                    if (c.selected !== '') {
                        return { "SetWeight": [c.selected, new Number(c.weight)] }
                    } else {
                        return { "ERROR": {message: "Node not selected", data: c} }
                    }
                default:
                    return { "FATAL": { message: `c.type = ${c.type}`, data: c} }
            }
        })
        const errors = command_list.filter(r => r.ERROR !== undefined || r.FATAL !== undefined)
        if (errors.length > 0) {
            console.error('send_command', errors)
            return
        }

        const json = JSON.stringify(command_list)
        console.log('JSON', json)
        const res_command = await fetch('http://localhost:3000/api/anim_graph_command', {
            method: 'POST',
            body: json,
            headers: {
                'Content-Type': 'application/json',
            },
        })

        console.log(res_command)

        queued_commands.value = []
    } catch (error) {
        console.error(error.message)
    }
}

</script>

<template>
    <div class="container">
        <input type="button" @click="open_gltf" value="LoadGltf" />
        <div>
            {{ gltf_info.status }}
        </div>
        <input type="button" @click="send_graph" value="send" />
        <div class="node_palette">
            <div v-for="animation in gltf_info.gltf_info?.animations ?? []"
                class="vue-flow__node-output node_item" :draggable="true" @dragstart="onDragStart($event, 'clip', { weight: 1.0, clip_name:animation.name })">{{ animation.name }}</div>
            <div class="vue-flow__node-default node_item" :draggable="true" @dragstart="onDragStart($event, 'blend', { weight: 1.0 })" >Blend Node</div>
            <div class="vue-flow__node-default node_item" :draggable="true" @dragstart="onDragStart($event, 'additive-blend', { weight: 1.0 })" >Additive Blend Node</div>
        </div>
        <div class="Command Queue">
            <div>
                <ul>
                    <li v-for="(command, idx) in queued_commands" >
                        <div v-if="command.type === 'Play'">
                            Play
                            <select v-model="queued_commands[idx].selected">
                                <option value="" disabled>Select a clip node</option>
                                <option v-for="option in clip_options" :key="option" :value="option">{{ option }}</option>
                            </select>
                        </div>
                        <div v-if="command.type === 'Weight'">
                            Weight
                            <select v-model="queued_commands[idx].selected">
                                <option value="" disabled>Select a node</option>
                                <option v-for="option in node_options" :key="option" :value="option">{{ option }}</option>
                            </select>
                            <input v-model="queued_commands[idx].weight">
                        </div>
                    </li>
                </ul>
            </div>
            <input type="button" @click="add_command('Play')" value="Play">
            <input type="button" @click="add_command('Weight')" value="Weight">
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
