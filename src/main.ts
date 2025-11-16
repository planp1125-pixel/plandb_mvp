import { createApp } from 'vue'
import './assets/global.css'
import App from './App.vue'
import './style.css' 
import './assets/theme.css'

// Set initial theme before the app mounts to avoid flicker/flash
const savedTheme = localStorage.getItem('theme')
const prefersDark = window.matchMedia?.('(prefers-color-scheme: dark)').matches
const initialDark = savedTheme ? savedTheme === 'dark' : !!prefersDark

const root = document.documentElement
root.classList.toggle('dark', initialDark)
root.setAttribute('data-theme', initialDark ? 'dark' : 'light')

createApp(App).mount('#app')

// import { createApp } from 'vue'
// import App from './App.vue'
// import './style.css'  // ← This line must exist
// import './assets/theme.css'
// import './assets/global.css'

// createApp(App).mount('#app')
