<script>
  import Counter from './lib/Counter.svelte'
  import { greet, add } from '../pkg/grav_sim'
</script>

<div>
  <p>{greet('grav-sim')}</p>
  <p>Rust says 3 + 2 = {add(3,2)}</p>
  <Counter />
</div>