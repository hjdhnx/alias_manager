<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import StatusBar from './components/StatusBar.vue'
import AliasList from './components/AliasList.vue'
import AliasForm from './components/AliasForm.vue'
import ShareImportDialog from './components/ShareImportDialog.vue'
import * as api from './api.js'

const aliases = ref([])
const status = ref(null)
const loading = ref(false)
const toast = ref(null)
const formVisible = ref(false)
const editing = ref(null)
const dialog = ref(null) // { mode: 'share' | 'import', text }

let toastTimer = null
function showToast(msg, type = 'info') {
  toast.value = { msg, type }
  clearTimeout(toastTimer)
  toastTimer = setTimeout(() => (toast.value = null), 3200)
}

async function refresh() {
  loading.value = true
  try {
    const [list, s] = await Promise.all([api.listAliases(), api.getStatus()])
    aliases.value = list
    status.value = s
  } catch (e) {
    showToast('加载失败：' + e, 'error')
  } finally {
    loading.value = false
  }
}

function onAdd() {
  editing.value = null
  formVisible.value = true
}

function onEdit(alias) {
  editing.value = { ...alias }
  formVisible.value = true
}

async function onSubmit(payload) {
  try {
    if (payload.id) {
      await api.updateAlias(payload.id, payload.name, payload.command, payload.description)
      showToast('已更新')
    } else {
      await api.addAlias(payload.name, payload.command, payload.description)
      showToast('已添加，新终端即可使用')
    }
    formVisible.value = false
    await refresh()
  } catch (e) {
    showToast('保存失败：' + e, 'error')
  }
}

async function onDelete(alias) {
  if (!confirm(`确定删除别名「${alias.name}」？`)) return
  try {
    await api.deleteAlias(alias.id)
    showToast('已删除')
    await refresh()
  } catch (e) {
    showToast('删除失败：' + e, 'error')
  }
}

async function onToggle(alias, enabled) {
  try {
    await api.toggleAlias(alias.id, enabled)
    await refresh()
  } catch (e) {
    showToast('切换失败：' + e, 'error')
    await refresh()
  }
}

async function onEnsurePath() {
  try {
    const changed = await api.ensurePath()
    showToast(changed ? '已加入 PATH，请重新打开终端使其生效' : 'PATH 已配置')
    await refresh()
  } catch (e) {
    showToast('配置失败：' + e, 'error')
  }
}

async function onOpenBin() {
  try {
    await api.openBinDir()
  } catch (e) {
    showToast('打开失败：' + e, 'error')
  }
}

async function onShare(ids) {
  try {
    const text = await api.exportAliases(ids)
    dialog.value = { mode: 'share', text }
  } catch (e) {
    showToast('分享失败：' + e, 'error')
  }
}

function onImport() {
  dialog.value = { mode: 'import', text: '' }
}

async function doImport(data) {
  try {
    const r = await api.importAliases(data)
    let msg = `已导入 ${r.imported} 条`
    if (r.skipped) msg += `，跳过 ${r.skipped} 条（重名或无效）`
    showToast(msg)
    dialog.value = null
    await refresh()
  } catch (e) {
    showToast('导入失败：' + e, 'error')
  }
}

async function onBatchEnable(ids) {
  try {
    const n = await api.setEnabled(ids, true)
    showToast(`已启用 ${n} 条`)
    await refresh()
  } catch (e) {
    showToast('操作失败：' + e, 'error')
  }
}

async function onBatchDisable(ids) {
  try {
    const n = await api.setEnabled(ids, false)
    showToast(`已禁用 ${n} 条`)
    await refresh()
  } catch (e) {
    showToast('操作失败：' + e, 'error')
  }
}

async function onBatchDelete(ids) {
  if (!confirm(`确定删除选中的 ${ids.length} 个别名？`)) return
  try {
    const n = await api.deleteAliases(ids)
    showToast(`已删除 ${n} 条`)
    await refresh()
  } catch (e) {
    showToast('删除失败：' + e, 'error')
  }
}

onMounted(refresh)
onUnmounted(() => clearTimeout(toastTimer))
</script>

<template>
  <div class="app">
    <header class="app-header">
      <div class="brand">
        <div class="logo">⚡</div>
        <div>
          <h1>别名管理器</h1>
          <p class="subtitle">一处配置 · 任意终端通用</p>
        </div>
      </div>
      <button class="btn primary" @click="onAdd">+ 新增别名</button>
    </header>

    <StatusBar
      v-if="status"
      :status="status"
      @ensure-path="onEnsurePath"
      @open-bin="onOpenBin"
    />

    <main class="content">
      <AliasList
        :aliases="aliases"
        :loading="loading"
        @edit="onEdit"
        @delete="onDelete"
        @toggle="onToggle"
        @add="onAdd"
        @share="onShare"
        @import="onImport"
        @batch-enable="onBatchEnable"
        @batch-disable="onBatchDisable"
        @batch-delete="onBatchDelete"
      />
    </main>

    <AliasForm
      v-if="formVisible"
      :alias="editing"
      @submit="onSubmit"
      @close="formVisible = false"
    />

    <ShareImportDialog
      v-if="dialog"
      :mode="dialog.mode"
      :text="dialog.text"
      @close="dialog = null"
      @import="doImport"
    />

    <Transition name="toast">
      <div v-if="toast" :class="['toast', toast.type]">{{ toast.msg }}</div>
    </Transition>
  </div>
</template>
