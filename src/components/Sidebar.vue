<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useWorkbench } from '../composables/useWorkbench'

const route   = useRoute()
const { currentId } = useWorkbench()
</script>

<template>
  <nav class="sidebar">
    <div class="nav-top">
      <!-- Project-scoped section -->
      <div class="nav-section nav-section--project">

        <!-- Projects list -->
        <router-link
          to="/projects"
          class="nav-item"
          :class="{ active: route.name === 'projects' }"
          title="Projects"
        >
          <svg width="17" height="17" viewBox="0 0 17 17" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2"   y="2"   width="5.5" height="5.5" rx="1.5"/>
            <rect x="9.5" y="2"   width="5.5" height="5.5" rx="1.5"/>
            <rect x="2"   y="9.5" width="5.5" height="5.5" rx="1.5"/>
            <rect x="9.5" y="9.5" width="5.5" height="5.5" rx="1.5"/>
          </svg>
        </router-link>

        <!-- Write (dimmed when no project open) -->
        <router-link
          to="/write"
          class="nav-item"
          :class="{ active: route.name === 'write', 'nav-item--dim': !currentId }"
          :title="currentId ? 'Write' : 'Write (open a project first)'"
        >
          <svg width="17" height="17" viewBox="0 0 17 17" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2.5L14.5 5L7 12.5H4.5V10L12 2.5Z"/>
            <path d="M2.5 14.5H14.5"/>
          </svg>
        </router-link>

        <!-- Workbench (dimmed when no project open) -->
        <router-link
          to="/workbench"
          class="nav-item"
          :class="{ active: route.name === 'workbench', 'nav-item--dim': !currentId }"
          :title="currentId ? 'Workbench' : 'Workbench (open a project first)'"
        >
          <svg width="17" height="17" viewBox="0 0 17 17" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M1.5 5.5L8.5 2L15.5 5.5L8.5 9L1.5 5.5Z"/>
            <path d="M1.5 9.5L8.5 13L15.5 9.5"/>
            <path d="M1.5 12.5L8.5 16L15.5 12.5"/>
          </svg>
        </router-link>

      </div>

      <div class="nav-divider"></div>

      <!-- Global section -->
      <div class="nav-section nav-section--global">

        <router-link
          to="/library"
          class="nav-item"
          :class="{ active: route.name === 'library' }"
          title="Library"
        >
          <svg width="17" height="17" viewBox="0 0 17 17" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 2.5h2.5v12H3z"/>
            <path d="M7 2.5h2.5v12H7z"/>
            <path d="M11.5 2.5l2.4.9-4.4 11.4-2.4-.9z"/>
          </svg>
        </router-link>

        <router-link
          to="/annotations"
          class="nav-item"
          :class="{ active: route.name === 'annotations' }"
          title="Annotations"
        >
          <svg width="17" height="17" viewBox="0 0 17 17" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2.5 12.5L5 10l7.5-7.5a1.414 1.414 0 012 2L7 12l-2.5.5z"/>
            <line x1="1.5" y1="15.5" x2="15.5" y2="15.5"/>
          </svg>
        </router-link>

        <router-link
          to="/pdf"
          class="nav-item"
          :class="{ active: route.name === 'pdf' }"
          title="PDF Viewer"
        >
          <svg width="17" height="17" viewBox="0 0 17 17" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M10 2H4a1 1 0 00-1 1v11a1 1 0 001 1h9a1 1 0 001-1V7L10 2Z"/>
            <path d="M10 2v5h4"/>
            <line x1="5.5" y1="9.5" x2="11.5" y2="9.5"/>
            <line x1="5.5" y1="12" x2="9" y2="12"/>
          </svg>
        </router-link>

      </div>
    </div>

    <div class="nav-bottom">
      <div class="nav-item settings" title="Settings">
        <svg width="17" height="17" viewBox="0 0 17 17" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="8.5" cy="8.5" r="2.5"/>
          <path d="M8.5 1v2M8.5 14v2M1 8.5h2M14 8.5h2M3.05 3.05l1.41 1.41M12.54 12.54l1.41 1.41M3.05 13.95l1.41-1.41M12.54 4.46l1.41-1.41"/>
        </svg>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-w);
  flex-shrink: 0;
  background: var(--bg-chrome);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  padding: 10px 0 10px;
  overflow: hidden;
}

.nav-top, .nav-bottom {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  width: 100%;
}

.nav-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  width: 100%;
}

.nav-divider {
  width: 22px;
  height: 1px;
  background: var(--border-medium);
  margin: 5px auto;
}

/* Base nav item */
.nav-item {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius);
  color: var(--text-tertiary);
  text-decoration: none;
  cursor: pointer;
  transition: background var(--t), color var(--t), opacity var(--t);
  border: none;
  background: none;
  position: relative;
}

.nav-item:hover {
  background: var(--bg-chrome-active);
  color: var(--text-secondary);
}

.nav-item.active {
  background: var(--accent-soft);
  color: var(--accent);
}

/* Dim state for project-dependent items with no active project */
.nav-item--dim {
  opacity: 0.35;
}
.nav-item--dim:hover {
  opacity: 0.55;
  background: none;
  color: var(--text-tertiary);
}

/* Left rail for project-scoped items when active */
.nav-section--project .nav-item {
  width: 100%;
  border-radius: 0;
}

.nav-section--project .nav-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  height: 20px;
  width: 2px;
  background: var(--accent);
  border-radius: 0 2px 2px 0;
}

.nav-item.settings {
  margin-top: 4px;
}
</style>
