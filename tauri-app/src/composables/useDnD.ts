import { useVueFlow, type Node  } from '@vue-flow/core'
import { ref, watch } from 'vue'
import type { NodeTypeName, NodeData } from '../components/nodes/NodeTypes'


let id = 0

function getId(type: NodeTypeName) {
    return `${type}_${id++}`
}

const state = {
    draggedType: ref<NodeTypeName | null>(null),
    draggedTypeData: ref<NodeData | null>(null),
    isDragOver: ref(false),
    isDragging: ref(false),
}

export default function useDragAndDrop() {
    const { draggedType, draggedTypeData, isDragOver, isDragging } = state

    const { addNodes, screenToFlowCoordinate, onNodesInitialized, updateNode } = useVueFlow('animation_graph')

    watch(isDragging, (dragging) => {
        console.log('watch(isDragging)', dragging)
        document.body.style.userSelect = dragging ? 'none' : ''
    })

    function onDragStart(event: DragEvent, type: NodeTypeName, data: NodeData) {
        console.log('onDragStart', event, type, data)
        if (event.dataTransfer) {
            console.log('---', event.dataTransfer)
            event.dataTransfer.setData('application/vueflow', type)
            event.dataTransfer.effectAllowed = 'move'
        }

        draggedType.value = type
        draggedTypeData.value = data
        isDragging.value = true

        document.addEventListener('drop', onDragEnd)
    }

    function onDragOver(event: DragEvent) {
        console.log('onDragOver', event)
        event.preventDefault()

        if (draggedType.value) {
            isDragOver.value = true

            if (event.dataTransfer) {
                event.dataTransfer.dropEffect = 'move'
            }
        }
    }

    function onDragLeave() {
        console.log('onDragLeave')
        isDragOver.value = false
    }

    function onDragEnd() {
        console.log('onDragEnd')
        isDragging.value = false
        isDragOver.value = false
        draggedType.value = null
        document.removeEventListener('drop', onDragEnd)
    }

    function onDrop(event: DragEvent) {
        console.log('onDrop', event)
        const position = screenToFlowCoordinate({
            x: event.clientX,
            y: event.clientY,
        })

        if (draggedType.value === null) {
            return
        }
        if (draggedTypeData.value === null) {
            return
        }
        const nodeId = getId(draggedType.value)

        draggedTypeData.value.label_id = nodeId

        const newNode: Node = {
            id: nodeId,
            type: draggedType.value,
            position,
            data: draggedTypeData.value,
        }

        const { off } = onNodesInitialized(() => {
            updateNode(nodeId, (node) => ({
                position: {
                    x: node.position.x - node.dimensions.width / 2,
                    y: node.position.y - node.dimensions.height / 2,
                }
            }))

            off()
        })

        addNodes(newNode)
    }

    return {
        draggedType,
        draggedTypeData,
        isDragOver,
        isDragging,

        onDragStart,
        onDragLeave,
        onDragOver,
        onDrop,
    }
}
