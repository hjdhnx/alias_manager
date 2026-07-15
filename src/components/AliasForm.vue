<script setup>
import { ref, watch } from 'vue'

const props = defineProps({
  alias: { type: Object, default: null }
})
const emit = defineEmits(['submit', 'close'])

const name = ref('')
const command = ref('')
const description = ref('')
const error = ref('')

watch(
  () => props.alias,
  (a) => {
    name.value = a?.name || ''
    command.value = a?.command || ''
    description.value = a?.description || ''
    error.value = ''
  },
  { immediate: true }
)

function submit() {
  if (!name.value.trim()) {
    error.value = '请填写名称'
    return
  }
  if (!command.value.trim()) {
    error.value = '请填写命令'
    return
  }
  emit('submit', {
    id: props.alias?.id || null,
    name: name.value.trim(),
    command: command.value.trim(),
    description: description.value.trim()
  })
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal">
      <h2>{{ alias ? '编辑别名' : '新增别名' }}</h2>
      <div class="field">
        <label>名称（在终端输入这个名字触发别名）</label>
        <input
          v-model="name"
          type="text"
          placeholder="如 claudex（字母/数字/_/-）"
          @keydown.enter="submit"
        />
      </div>
      <div class="field">
        <label>命令（别名展开后的完整命令）</label>
        <input
          v-model="command"
          type="text"
          placeholder="如 claude --dangerously-skip-permissions"
          @keydown.enter="submit"
        />
      </div>
      <div class="field">
        <label>说明（可选）</label>
        <input
          v-model="description"
          type="text"
          placeholder="简单描述用途"
          @keydown.enter="submit"
        />
      </div>
      <p class="hint">
        生效后输入 <code>{{ name || '名称' }}</code> 等同于执行
        <code>{{ command || '命令' }}</code>，额外参数会自动透传。
      </p>
      <p v-if="error" class="form-error">{{ error }}</p>
      <div class="modal-footer">
        <button class="btn ghost" @click="emit('close')">取消</button>
        <button class="btn primary" @click="submit">保存</button>
      </div>
    </div>
  </div>
</template>
