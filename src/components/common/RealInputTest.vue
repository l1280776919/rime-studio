<script setup lang="ts">
import { ref, watch, useId } from "vue";
const visible = ref(false);
const inputId = useId();
const text = ref("");
// Keep the field native: composition, candidate selection and shortcuts belong to the IME.
// Input stays in component memory and is discarded when the dialog closes.
watch(visible, (open) => {
  if (!open) text.value = "";
});
</script>

<template>
  <el-button size="small" @click="visible = true">真实输入验证</el-button>
  <el-dialog
    v-model="visible"
    top="5vh"
    title="验证已部署的输入配置"
    width="min(680px, 94vw)"
    append-to-body
    destroy-on-close
  >
    <p>先完成保存和部署，再用 Win + 空格切换到小狼毫，点击输入框测试。</p>
    <ul>
      <li>输入刚添加的短语编码，检查候选词。</li>
      <li>输入当前方案的拼音或双拼编码，检查方案和候选窗外观。</li>
      <li>输入方案支持的日期等 Lua 触发码，检查扩展功能。</li>
    </ul>
    <label :for="inputId">测试文本（关闭后清空，不保存）</label>
    <textarea
      :id="inputId"
      v-model="text"
      rows="7"
      spellcheck="false"
      autocomplete="off"
      placeholder="切换到小狼毫后，在这里输入…"
      @keydown.stop
    />
    <p class="helper-text">候选窗由系统输入法显示；请自行确认当前使用的是小狼毫。</p>
    <template #footer
      ><el-button @click="text = ''">清空</el-button
      ><el-button type="primary" @click="visible = false">完成</el-button></template
    >
  </el-dialog>
</template>

<style scoped>
textarea {
  display: block;
  box-sizing: border-box;
  width: 100%;
  margin-top: 10px;
  padding: 14px;
  resize: vertical;
  font: inherit;
  line-height: 1.6;
  color: var(--ink-800);
  background: var(--color-surface);
  border: 1px solid var(--el-border-color);
  border-radius: var(--radius-md);
}
textarea:focus {
  outline: 2px solid var(--el-color-primary);
  outline-offset: 2px;
}
li {
  margin: 8px 0;
}
</style>
