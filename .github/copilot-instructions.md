## Battleship Probability Engine – Project Summary

### 🎯 Objective

Build a **Battleship probability calculator** that can evaluate the likelihood of ship positions given partial hit/miss states, for use in a mobile or desktop app. The goal is to use precomputed valid board end-states, filter them based on a player’s known observations, and visualize a "heatmap" of probable ship locations to guide decision-making.

---

### 🧩 Game Configuration

- **Board Size:** 9x9 (81 cells)
- **Ships:**
  - 3 ships of length 4
  - 5 ships of length 3
- **Placement Rules:**
  - Ships may not touch each other (no adjacent placements)
  - Ships are only placed horizontally or vertically (no diagonals)
  - All ships must be placed without overlap

---

### 📦 Data Format

#### ✅ End-state Boards (Compressed)

- Each complete board is stored as a **16-byte u128 bitmask** representing ship placements (hits).
- Bit `i` is 1 if there is a ship segment at position `i` (row-major across 9×9 grid).
- No need to store misses, since those are the complement of the hit mask.

#### 🔄 Delta Encoding

- Instead of saving each full board, successive boards are **XOR delta-encoded** against the previous one for better compression.
- This produces a sequence of deltas that can be decompressed back to original states.

#### 🗜 Compression

- Delta-encoded data is further compressed using `zstd` (with experimentation at various levels, including extreme levels like `--ultra -22` yielding \~5MB from \~3.2GB of raw).
- Hash checks with `xxh3` and `sha256sum` verify identical board content across formats.

---

### 🧪 Filtering Pipeline

The app (or CLI) takes in:

- A **user's current board state**, represented as two `u128` masks:
  - `hitMask`: bits that are known hits
  - `missMask`: bits that are known misses
- The app or backend filters through all precomputed valid end-states and returns only those that match the current hit/miss configuration.

---

### 📊 Heatmap Generation

After filtering:

- A heatmap is generated where each cell’s value represents how many valid boards include a ship at that cell.
- These values are visualized in a 9×9 SwiftUI grid (with future plans for color gradients).
- This gives the player a strong probabilistic suggestion on where to strike next.

---

### ⚙️ Implementation Details

#### Swift + Rust

- SwiftUI is used for the front-end visualization on iOS/macOS.
- Rust is used for high-performance backend logic:
  - Decoding zstd-compressed delta-encoded board files
  - Filtering based on current state
  - Accumulating cell-wise counts
- A Rust **FFI-compatible dylib or xcframework** is planned for integration with Swift.

#### Performance Optimization

- Initial generation of 213 million valid boards took \~20 minutes unbuffered.
- After delta encoding and buffered writes, generation time dropped to \~78 seconds.
- Final file sizes:
  - Raw: \~3.2 GB
  - Delta + gzip: \~209 MB
  - Delta + zstd: \~88 MB
  - Delta + zstd + ultra compression: \~5.4 MB

---

### 🧠 Design Tradeoffs Considered

- Used `u128` masks for compact, SIMD-friendly bitwise comparison.
- Switched from storing both hit + miss to just hit (derived miss from complement).
- Chose not to store metadata like last move or guessed ship types to allow state deduplication.
- Avoided over-indexing on `u128` in FFI (split into two `u64`s for cross-platform compatibility).
- Considered but avoided storing adjacency/masking metadata directly—treating blocked neighbors as display-only for now.

---

### 🔮 Next Steps

1. **Build Rust dylib with zstd decoding and delta filtering**
2. **Integrate with Swift app via FFI/XCFramework**
3. **Interactive UI for the player to mark hits/misses**
4. **Dynamically recompute heatmap with each player action**
5. Optionally support:
   - Ship sinking inference
   - Autoplay / bot logic
   - Optimizing next-move suggestions based on entropy reduction
