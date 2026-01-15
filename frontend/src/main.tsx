import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './index.css'

console.log('[MAIN] Starting React application...')

const rootElement = document.getElementById('root')
if (!rootElement) {
  console.error('[MAIN] Root element not found!')
  document.body.innerHTML = '<h1 style="color: red;">Error: Root element not found!</h1>'
} else {
  console.log('[MAIN] Root element found, rendering App...')
  try {
    ReactDOM.createRoot(rootElement).render(
      <React.StrictMode>
        <App />
      </React.StrictMode>,
    )
    console.log('[MAIN] App rendered successfully')
  } catch (error) {
    console.error('[MAIN] Error rendering App:', error)
    rootElement.innerHTML = `<h1 style="color: red;">Error: ${error}</h1>`
  }
}
