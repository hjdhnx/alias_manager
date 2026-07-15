<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  aliases: { type: Array, default: () => [] },
  loading: Boolean
})
const emit = defineEmits([
  'edit',
  'delete',
  'toggle',
  'add',
  'share',
  'import',
  'batch-enable',
  'batch-disable',
  'batch-delete'
])

const keyword = ref('')
const selectedIds = ref(new Set())

const filtered = computed(() => {
  const k = keyword.value.trim().toLowerCase()
  if (!k) return props.aliases
  return props.aliases.filter(
    (a) =>
      a.name.toLowerCase().includes(k) ||
      a.command.toLowerCase().includes(k) ||
      (a.description || '').toLowerCase().includes(k)
  )
})

const selectedCount = computed(() => selectedIds.value.size)
const allSelected = computed(
  () =>
    filtered.value.length > 0 &&
    filtered.value.every((a) => selectedIds.value.has(a.id))
)
const indeterminate = computed(
  () => selectedCount.value > 0 && !allSelected.value
)

function toggleAll(e) {
  const next = new Set(selectedIds.value)
  if (e.target.checked) filtered.value.forEach((a) => next.add(a.id))
  else filtered.value.forEach((a) => next.delete(a.id))
  selectedIds.value = next
}
function toggleOne(id, checked) {
  const next = new Set(selectedIds.value)
  if (checked) next.add(id)
  else next.delete(id)
  selectedIds.value = next
}
function clearSelection() {
  selectedIds.value = new Set()
}
function onShare() {
  emit('share', Array.from(selectedIds.value))
  selectedIds.value = new Set()
}
function batchEnable() {
  emit('batch-enable', Array.from(selectedIds.value))
}
function batchDisable() {
  emit('batch-disable', Array.from(selectedIds.value))
}
function batchDelete() {
  emit('batch-delete', Array.from(selectedIds.value))
  selectedIds.value = new Set()
}
function copyCommand(cmd) {
  navigator.clipboard?.writeText(cmd)
}
</script>

<template>
  <div class="list-wrap">
    <div class="toolbar">
      <input
        v-model="keyword"
        class="search"
        type="text"
        placeholder="搜索名称 / 命令 / 描述..."
      />
      <span class="count">共 {{ aliases.length }} 条</span>
      <button class="btn primary sm" @click="emit('import')">导入</button>
    </div>

    <div v-if="selectedCount" class="batchbar">
      <span class="batch-info">已选 {{ selectedCount }} 项</span>
      <div class="batch-actions">
        <button class="btn ghost sm" @click="batchEnable">启用</button>
        <button class="btn ghost sm" @click="batchDisable">禁用</button>
        <button class="btn ghost sm" @click="onShare">分享</button>
        <button class="btn danger sm" @click="batchDelete">删除</button>
      </div>
      <button class="btn link sm" @click="clearSelection">取消选择</button>
    </div>

    <div v-if="loading && !aliases.length" class="empty">加载中...</div>

    <div v-else-if="!filtered.length" class="empty">
      <template v-if="aliases.length">
        <p class="empty-title">没有匹配的别名</p>
      </template>
      <template v-else>
        <p class="empty-title">还没有别名</p>
        <p class="empty-desc">
          例如配置：<code>claudex = claude --dangerously-skip-permissions</code><br />
          保存后即可在 PowerShell / CMD / Git Bash 直接输入 claudex 使用
        </p>
        <button class="btn primary" @click="emit('add')">+ 新增第一个别名</button>
      </template>
    </div>

    <div v-else class="table">
      <div class="row head">
        <span class="col-sel">
          <input
            type="checkbox"
            :checked="allSelected"
            :indeterminate="indeterminate"
            @change="toggleAll"
          />
        </span>
        <span class="col-name">名称</span>
        <span class="col-cmd">命令</span>
        <span class="col-desc">说明</span>
        <span class="col-en">启用</span>
        <span class="col-op">操作</span>
      </div>
      <div
        v-for="a in filtered"
        :key="a.id"
        :class="['row', { disabled: !a.enabled }]"
      >
        <span class="col-sel">
          <input
            type="checkbox"
            :checked="selectedIds.has(a.id)"
            @change="toggleOne(a.id, $event.target.checked)"
          />
        </span>
        <span class="col-name mono">{{ a.name }}</span>
        <span
          class="col-cmd mono cmd-cell"
          :title="`点击复制：${a.command}`"
          @click="copyCommand(a.command)"
        >{{ a.command }}</span>
        <span class="col-desc desc">{{ a.description || '—' }}</span>
        <span class="col-en">
          <label class="switch">
            <input
              type="checkbox"
              :checked="a.enabled"
              @change="emit('toggle', a, $event.target.checked)"
            />
            <span class="slider"></span>
          </label>
        </span>
        <span class="col-op">
          <button class="btn ghost sm" @click="emit('edit', a)">编辑</button>
          <button class="btn danger sm" @click="emit('delete', a)">删除</button>
        </span>
      </div>
    </div>
  </div>
</template>
