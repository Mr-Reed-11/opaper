import { ref, watchEffect } from 'vue'

type Theme = 'blue' | 'mono'

const theme = ref<Theme>((localStorage.getItem('opaper-theme') as Theme) || 'blue')

watchEffect(() => {
  document.documentElement.setAttribute('data-theme', theme.value)
  localStorage.setItem('opaper-theme', theme.value)
})

export function useTheme() {
  function toggle() {
    theme.value = theme.value === 'blue' ? 'mono' : 'blue'
  }
  return { theme, toggle }
}
