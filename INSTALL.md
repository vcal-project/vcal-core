# VCAL-core Installation & Integration Guide

> **Status:** Early access (not yet on crates.io)  
> For Rust usage details, see [README.md](README.md).  
> This guide focuses on installing **VCAL-core** from GitHub and integrating it into **Python-based chatbots** that use OpenAI, Anthropic, or local LLM APIs such as OpenChat or Ollama.

---

## 1. Clone the Repository

VCAL-core is currently distributed via GitHub only:

```bash
git clone https://github.com/vcal-project/vcal-core.git
cd vcal-core
```

Build and test locally:

```bash
cargo build --release
cargo test --release
```

This will compile the optimized Rust library artifact:

```
target/release/libvcal_core.rlib
```

> The `libvcal_core.rlib` file is used internally by Cargo when linking other Rust crates.  
> You do not need to reference it manually unless building a custom FFI wrapper.

---

## 2. Optional: Use as a Rust Dependency (from GitHub)

Until it’s published to crates.io, you can depend on it directly from GitHub:

```toml
[dependencies]
vcal-core = { git = "https://github.com/vcal-project/vcal-core", features = ["serde"] }
```

Then build:

```bash
cargo build --release
```

Cargo will automatically link against the compiled `libvcal_core.rlib` in your local build cache.

---

## 3. Build a Python Wheel (via PyO3 + Maturin)

To use VCAL-core from Python, build a native wheel.

### Install maturin

```bash
pip install maturin
```

### Build the wheel

```bash
maturin build --release
```

Output example:

```
target/wheels/vcal_core-0.1.1-cp310-cp310-manylinux_x86_64.whl
```

### Install the wheel

```bash
pip install target/wheels/vcal_core-0.1.1-*.whl
```

---

## 4. Using VCAL-core from Python

```python
from vcal_core import Index, Cosine

# Create a 128-dimensional index
idx = Index(dims=128, metric=Cosine)

# Insert a few vectors
idx.insert([0.1] * 128, "id-1")
idx.insert([0.2] * 128, "id-2")

# Search
hits = idx.search([0.1] * 128, k=3)
print(hits)

# Save and reload
idx.save("vcal.index")
idx = Index.load("vcal.index")
```

---

## 5. Integrating into Chatbots

Below are minimal integration examples showing how VCAL-core can cache LLM responses based on semantic similarity of prompts.

### Example A — OpenAI / Anthropic API

```python
from vcal_core import Index, Cosine
from openai import OpenAI
import numpy as np

client = OpenAI(api_key="YOUR_KEY")
idx = Index(dims=1536, metric=Cosine)

def embed(text):
    res = client.embeddings.create(input=text, model="text-embedding-3-small")
    return np.array(res.data[0].embedding, dtype=np.float32)

def ask(prompt):
    q = embed(prompt)
    hits = idx.search(q, 1)
    if hits and hits[0].score > 0.95:
        print("Cache hit:")
        return hits[0].payload

    answer = client.chat.completions.create(
        model="gpt-4o-mini",
        messages=[{"role": "user", "content": prompt}]
    ).choices[0].message.content

    idx.insert(q, answer)
    return answer

print(ask("Explain HNSW indexing in simple terms"))
```

---

### Example B — Local LLM (OpenChat / Ollama)

```python
from vcal_core import Index, Cosine
import subprocess, json

idx = Index(dims=768, metric=Cosine)

def ollama_embed(text):
    out = subprocess.run(
        ["ollama", "embed", "-m", "nomic-embed-text", text],
        capture_output=True, text=True
    )
    return json.loads(out.stdout)["embedding"]

def local_ask(prompt):
    q = ollama_embed(prompt)
    hits = idx.search(q, 1)
    if hits and hits[0].score > 0.9:
        print("Cache hit")
        return hits[0].payload

    out = subprocess.run(
        ["ollama", "run", "mistral", "--json"],
        input=prompt, text=True, capture_output=True
    )
    result = json.loads(out.stdout)
    answer = result.get("response", "")
    idx.insert(q, answer)
    return answer

print(local_ask("What is semantic caching?"))
```

---

## 6. Conceptual Overview

```text
+-------------------+           +-------------------+
|   Chatbot Layer   |<--------->|   VCAL-core Index |
|  (LLM / API)      |           |   (in-process)    |
+-------------------+           +-------------------+
          ^                              |
          |                              v
  Embeddings (OpenAI, Ollama, etc.)   Snapshots (.index.A / .index.B)
```

VCAL-core runs entirely **in-process** — no sockets, no daemons, no HTTP calls.  
It’s ideal for embedding directly inside your AI backend, LangChain agent, or RAG service.

---

## 7. Performance and Safety Notes

- Build in release mode for maximum performance.
- Normalize embeddings if using cosine similarity.
- Always use paired snapshot filenames (`.A` / `.B`) to ensure atomic saves.
- Identical embedding dimensions are required for all inserts and searches.
- The Python wheel is fully self-contained — no runtime server required.

---

## 8. Troubleshooting

| Issue | Likely Cause | Fix |
|-------|---------------|-----|
| `ValueError: dims mismatch` | Inconsistent embedding size | Use same dimension across all inserts |
| `Index.load()` fails | Missing paired snapshot | Ensure `.index.A` or `.index.B` exists |
| Wheel fails to build | Missing Rust toolchain | Run `rustup install stable` |

---

## 9. Next Steps

- Explore **VCAL Server** for HTTP-based access and Prometheus metrics:  
  [https://vcal-project.com/vcal-server.html](https://vcal-project.com/vcal-server.html)
- Follow [VCAL Project](https://vcal-project.com) for future updates and PyPI release.

---

© 2025 VCAL-project contributors — Licensed under Apache-2.0.
