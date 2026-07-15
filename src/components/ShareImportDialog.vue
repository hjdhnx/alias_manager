<script setup>
import { ref, watch } from 'vue'

const props = defineProps({
  mode: { type: String, required: true }, // 'share' | 'import'
  text: { type: String, default: '' }
})
const emit = defineEmits(['close', 'import', 'copied'])

const input = ref('')
const copied = ref(false)

watch(
  () => [props.text, props.mode],
  ([t, m]) => {
    input.value = m === 'share' ? t : ''
    copied.value = false
  },
  { immediate: true }
)

async function copy() {
  try {
    await navigator.clipboard?.writeText(input.value)
    copied.value = true
    emit('copied')
    setTimeout(() => (copied.value = false), 1500)
  } catch (_) {
    // 剪贴板不可用时提示手动复制
  }
}

function doImport() {
  if (!input.value.trim()) return
  emit('import', input.value.trim())
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal">
      <h2>{{ mode === 'share' ? '分享别名' : '导入别名' }}</h2>
      <p class="hint" v-if="mode === 'share'">
        复制下方文本发给对方，对方点「导入」粘贴即可还原这批别名。
      </p>
      <p class="hint" v-else>
        粘贴对方分享的文本，名称已存在的别名会自动跳过。
      </p>
      <textarea
        v-model="input"
        class="share-area"
        :readonly="mode === 'share'"
        rows="9"
        :placeholder="mode === 'import' ? '在此粘贴分享文本...' : ''"
      ></textarea>
      <div class="modal-footer">
        <button class="btn ghost" @click="emit('close')">关闭</button>
        <button v-if="mode === 'share'" class="btn primary" @click="copy">
          {{ copied ? '已复制' : '复制全部' }}
        </button>
        <button v-else class="btn primary" @click="doImport">导入</button>
      </div>
    </div>
  </div>
</template>
