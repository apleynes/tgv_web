# TGV on client-side web using leptos

An exercise on deploying scientific computing algorithms on client-side web with Rust frameworks (leptos)

Roadmap
- [x] Initial implementation
- [ ] Parallelization with Rayon on webworkers
- [ ] Better UI for parameter settings
- [x] Deploy to Github Pages

References:
- https://book.leptos.dev/deployment/csr.html
- https://github.com/diversable/deploy_leptos_csr_to_gh_pages



# Enabling wasm-bindgen-rayon

https://github.com/GoogleChromeLabs/wasm-bindgen-rayon/issues/11#issuecomment-932397527


```[Trunk.toml]
[build]
pattern_script = '''
<script type="module">import init, { initThreadPool} from '{base}{js}';await init('{base}{wasm}'); await initThreadPool(navigator.hardwareConcurrency);</script>
'''

[serve]
headers = { "Cross-Origin-Embedder-Policy" = "require-corp", "Cross-Origin-Opener-Policy" = "same-origin" }
```
https://github.com/trunk-rs/trunk/issues/414#issuecomment-2334096961

https://github.com/trunk-rs/trunk/tree/main/examples/wasm_threads

The final key to get it to work was that wasm-bindgen-rayon must be using the `no-bundler` feature

