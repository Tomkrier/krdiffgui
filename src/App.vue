<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const sourceDir = ref("");
const patchDir = ref("");
const outputDir = ref("");
const logs = ref<string[]>([]);
const running = ref(false);

function addLog(message: string) {
  const time = new Date().toLocaleTimeString();
  logs.value.push(`[${time}] ${message}`);
}

async function selectDirectory(target: "source" | "patch" | "output") {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "选择目录",
  });

  if (typeof selected !== "string") {
    return;
  }

  if (target === "source") {
    sourceDir.value = selected;
  } else if (target === "patch") {
    patchDir.value = selected;
  } else {
    outputDir.value = selected;
  }
}

async function runPatch() {
  if (running.value) {
    return;
  }

  logs.value = [];

  if (!sourceDir.value.trim()) {
    addLog("错误: source_dir 不能为空");
    return;
  }

  if (!patchDir.value.trim()) {
    addLog("错误: patch_dir 不能为空");
    return;
  }

  if (!outputDir.value.trim()) {
    addLog("错误: output_dir 不能为空");
    return;
  }

  running.value = true;
  addLog("开始执行补丁...");

  try {
    const result = await invoke<string[]>("apply_krdiff_dir", {
      sourceDir: sourceDir.value,
      patchDir: patchDir.value,
      outputDir: outputDir.value,
    });

    for (const line of result) {
      addLog(line);
    }

    addLog("执行完成");
  } catch (error) {
    addLog(`执行失败: ${String(error)}`);
  } finally {
    running.value = false;
  }
}
</script>

<template>
  <main class="page">
    <section class="panel">
      <h1>KrDiff Patch GUI</h1>

      <div class="field">
        <label for="source-dir">source_dir</label>
        <div class="input-row">
          <input
              id="source-dir"
              v-model="sourceDir"
              placeholder="F:\Wuthering Waves Game"
              spellcheck="false"
          />
          <button type="button" @click="selectDirectory('source')">选择</button>
        </div>
      </div>

      <div class="field">
        <label for="patch-dir">patch_dir</label>
        <div class="input-row">
          <input
              id="patch-dir"
              v-model="patchDir"
              placeholder="F:\Wuthering Waves Game\launcherDownload\3.5.0"
              spellcheck="false"
          />
          <button type="button" @click="selectDirectory('patch')">选择</button>
        </div>
      </div>

      <div class="field">
        <label for="output-dir">output_dir</label>
        <div class="input-row">
          <input
              id="output-dir"
              v-model="outputDir"
              placeholder="F:\Wuthering Waves Game"
              spellcheck="false"
          />
          <button type="button" @click="selectDirectory('output')">选择</button>
        </div>
      </div>

      <button class="run-button" type="button" :disabled="running" @click="runPatch">
        {{ running ? "running..." : "run" }}
      </button>

      <div class="log-box">
        <div v-if="logs.length === 0" class="log-placeholder">
          The log will be displayed here
        </div>
        <div v-for="(line, index) in logs" :key="index" class="log-line">
          {{ line }}
        </div>
      </div>
    </section>
  </main>
</template>

<style scoped>
.page {
  min-height: 100vh;
  padding: 32px;
  box-sizing: border-box;
  background: #f3f4f6;
  color: #111827;
}

.panel {
  width: min(960px, 100%);
  margin: 0 auto;
  padding: 28px;
  border-radius: 16px;
  background: #ffffff;
  box-shadow: 0 10px 30px rgba(15, 23, 42, 0.12);
}

h1 {
  margin: 0 0 28px;
  text-align: center;
  font-size: 28px;
}

.field {
  margin-bottom: 18px;
}

label {
  display: block;
  margin-bottom: 8px;
  font-weight: 700;
  color: #374151;
}

.input-row {
  display: flex;
  gap: 10px;
}

input {
  flex: 1;
  min-width: 0;
  padding: 12px 14px;
  border: 1px solid #d1d5db;
  border-radius: 10px;
  font-size: 15px;
  color: #111827;
  background: #ffffff;
  outline: none;
}

input:focus {
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.16);
}

button {
  border: none;
  border-radius: 10px;
  padding: 0 18px;
  font-size: 15px;
  font-weight: 700;
  color: #ffffff;
  background: #2563eb;
  cursor: pointer;
  transition: background 0.2s, opacity 0.2s;
}

button:hover:not(:disabled) {
  background: #1d4ed8;
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.run-button {
  width: 100%;
  height: 46px;
  margin-top: 8px;
  background: #16a34a;
}

.run-button:hover:not(:disabled) {
  background: #15803d;
}

.log-box {
  height: 320px;
  margin-top: 24px;
  padding: 14px;
  overflow: auto;
  border-radius: 12px;
  background: #111827;
  color: #d1d5db;
  font-family: Consolas, Monaco, "Courier New", monospace;
  font-size: 13px;
  line-height: 1.6;
  text-align: left;
  white-space: pre-wrap;
}

.log-placeholder {
  color: #6b7280;
}

.log-line {
  margin-bottom: 2px;
}

@media (prefers-color-scheme: dark) {
  .page {
    background: #111827;
    color: #f9fafb;
  }

  .panel {
    background: #1f2937;
  }

  label {
    color: #e5e7eb;
  }

  input {
    color: #f9fafb;
    background: #111827;
    border-color: #374151;
  }

  .log-box {
    background: #030712;
  }
}
</style>