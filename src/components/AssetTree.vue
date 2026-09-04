<template>
  <div class="asset-tree">
    <template v-for="node in nodes" :key="node.uid">
      <div v-if="node.type === 'node'" class="tree-block">
        <div class="tree-row folder" @click="$emit('toggle', node.id)">
          <span class="twist" :class="{ open: isExpanded(node) }">
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
              <path d="M3 1.5L7 5 3 8.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </span>
          <span class="folder-icon">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none">
              <path d="M1.5 3.5h4l.8 1.2H11.5v6.3H1.5V3.5z" stroke="currentColor" stroke-width="1.1" fill="none"/>
            </svg>
          </span>
          <span class="folder-name" :title="node.name">{{ node.name }}</span>
          <span class="folder-count">{{ node.count }}</span>
        </div>
        <AssetTree
          v-if="isExpanded(node) && node.children?.length"
          :nodes="node.children"
          :expanded-ids="expandedIds"
          :force-expand="forceExpand"
          @toggle="$emit('toggle', $event)"
          @open-asset="$emit('openAsset', $event)"
          @add-tag="$emit('addTag', $event)"
        />
      </div>

      <div
        v-else
        class="tree-row asset"
        @click="$emit('openAsset', node)"
        @dblclick="$emit('openAsset', node)"
      >
        <span class="asset-status" :class="{ connected: isConnected(node.assetId) }"></span>
        <div class="asset-info">
          <div class="asset-title" :title="node.address ? `${node.name} (${node.address})` : node.name">
            {{ node.name }}
          </div>
        </div>
        <div class="asset-action">
          <n-button text size="tiny" title="添加标签" @click.stop="$emit('addTag', node)">
            <template #icon>
              <svg width="13" height="13" viewBox="0 0 13 13" fill="none">
                <path d="M2 2h4l3 3-5 5L2 8V2z" stroke="currentColor" stroke-width="1" fill="none"/>
                <circle cx="8.5" cy="4.5" r="0.8" fill="currentColor"/>
              </svg>
            </template>
          </n-button>
        </div>
        <div v-if="appStore.getAssetTags(node.assetId).length > 0" class="asset-tags">
          <span
            v-for="tag in appStore.getAssetTags(node.assetId)"
            :key="tag"
            class="tag-chip"
          >
            {{ tag }}
            <span class="close-btn" @click.stop="appStore.removeTag(node.assetId, tag)">&times;</span>
          </span>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { NButton } from 'naive-ui'
import { useAppStore } from '../stores/app'

defineOptions({ name: 'AssetTree' })

const props = defineProps({
  nodes: { type: Array, default: () => [] },
  expandedIds: { type: Object, default: () => ({}) },
  forceExpand: { type: Boolean, default: false }
})

defineEmits(['toggle', 'openAsset', 'addTag'])

const appStore = useAppStore()

function isExpanded(node) {
  return props.forceExpand || !!props.expandedIds[node.id]
}

function isConnected(assetId) {
  return appStore.tabs.some(t => t.assetId === assetId && t.connected)
}
</script>

<style scoped>
.asset-tree {
  display: flex;
  flex-direction: column;
}

.tree-block {
  display: flex;
  flex-direction: column;
}

.tree-block > .asset-tree {
  padding-left: 12px;
}

.tree-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px 5px 8px;
  cursor: pointer;
  user-select: none;
  position: relative;
  flex-wrap: wrap;
}

.tree-row:hover {
  background: var(--bg-hover);
}

.twist {
  width: 10px;
  height: 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  flex-shrink: 0;
  transition: transform 0.12s ease;
  transform: rotate(0deg);
}

.twist.open {
  transform: rotate(90deg);
}

.folder-icon {
  display: inline-flex;
  color: var(--text-muted);
  flex-shrink: 0;
}

.folder-name {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.folder-count {
  font-size: 11px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.tree-row.asset {
  padding-left: 20px;
}

.asset-status {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-muted);
  flex-shrink: 0;
}

.asset-status.connected {
  background: var(--success);
}

.asset-info {
  flex: 1;
  min-width: 0;
}

.asset-title {
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.asset-action {
  opacity: 0;
  flex-shrink: 0;
}

.tree-row.asset:hover .asset-action {
  opacity: 1;
}

.asset-tags {
  flex-basis: 100%;
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding-left: 12px;
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 8px;
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.close-btn {
  cursor: pointer;
  font-size: 11px;
  line-height: 1;
}
</style>
