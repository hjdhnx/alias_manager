<script setup>
defineProps({
  status: { type: Object, required: true }
})
const emit = defineEmits(['ensure-path', 'open-bin'])
</script>

<template>
  <div :class="['statusbar', { warn: !status.path_configured }]">
    <div class="status-left">
      <span :class="['dot', status.path_configured ? 'on' : 'off']"></span>
      <span v-if="status.path_configured" class="status-text">
        PATH 已配置 · 启用 {{ status.enabled }} / {{ status.total }}
      </span>
      <span v-else class="status-text">
        <strong>未配置 PATH，别名暂不可用</strong> · 启用
        {{ status.enabled }} / {{ status.total }}
      </span>
    </div>
    <div class="status-right">
      <button class="btn link" @click="emit('open-bin')" title="shim 文件所在目录">
        打开 bin 目录
      </button>
      <button v-if="!status.path_configured" class="btn primary sm" @click="emit('ensure-path')">
        一键配置 PATH
      </button>
      <button v-else class="btn ghost sm" @click="emit('ensure-path')">重新检查</button>
    </div>
  </div>
</template>
