import { mount } from 'svelte'
import './styles/tokens.css'
import './styles/base.css'
import './styles/ui.css'
import App from './App.svelte'
import init from '../pkg/grav_sim'

// load + instantiate the .wasm before any component calls into Rust
await init()

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
