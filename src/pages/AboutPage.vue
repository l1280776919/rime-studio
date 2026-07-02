<script setup lang="ts">
import { Collection, Connection, Link } from "@element-plus/icons-vue";
</script>

<template>
  <section class="content-grid about-grid">
    <section class="main-column">
      <el-card class="panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>关于 Rime Studio</span>
          </div>
        </template>

        <div class="about-intro">
          <div class="brand-mark" style="margin-bottom: 16px">R</div>
          <h3>Rime Studio</h3>
          <p>小狼毫输入法配置工作台 v0.1.0</p>
          <p class="helper-text">
            基于 Tauri 2 + Vue 3 + Rust 构建的桌面应用，提供图形化界面来管理 Rime 输入法的外观主题、自定义短语、词库和配置备份。
          </p>
        </div>
      </el-card>
    </section>

    <aside class="side-column">
      <el-card class="panel" shadow="never">
        <template #header>
          <span>相关链接</span>
        </template>
        <div class="about-links">
          <a
            href="https://github.com/l1280776919/rime-studio"
            target="_blank"
            class="about-link-card"
          >
            <el-icon><Collection /></el-icon>
            <span>
              <strong>Rime Studio</strong>
              <small>本项目 — 小狼毫配置工作台</small>
            </span>
            <el-icon class="link-arrow"><Link /></el-icon>
          </a>

          <a
            href="https://github.com/rime/home"
            target="_blank"
            class="about-link-card"
          >
            <el-icon><Connection /></el-icon>
            <span>
              <strong>Rime 中州韻</strong>
              <small>输入法引擎 — 全平台开源输入法框架</small>
            </span>
            <el-icon class="link-arrow"><Link /></el-icon>
          </a>

          <a
            href="https://github.com/iDvel/rime-ice"
            target="_blank"
            class="about-link-card"
          >
            <el-icon><Connection /></el-icon>
            <span>
              <strong>雾凇拼音</strong>
              <small>rime-ice — 长期维护的简体中文词库配置</small>
            </span>
            <el-icon class="link-arrow"><Link /></el-icon>
          </a>
        </div>
      </el-card>

      <el-card class="panel" shadow="never">
        <template #header>
          <span>技术栈</span>
        </template>
        <div class="tech-stack">
          <div><span>桌面框架</span><strong>Tauri 2</strong></div>
          <div><span>前端</span><strong>Vue 3 + Element Plus</strong></div>
          <div><span>后端</span><strong>Rust</strong></div>
          <div><span>平台</span><strong>Windows</strong></div>
        </div>
      </el-card>
    </aside>
  </section>
</template>
