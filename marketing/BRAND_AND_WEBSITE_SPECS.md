# ORA Brand & Website Specifications (Late 2026)

## 1. Brand Identity & Visual Strategy
In late 2026, the SaaS and AI landscape has shifted away from stark, clinical minimalism toward "Dopamine Color Palettes" and "Tactile Maximalism" mixed with deep, sophisticated dark modes. ORA will leverage a "Stealth Enterprise" aesthetic.

**Concept:** Stand out by blending in with the elite. ORA isn't a toy; it is an enterprise-grade security kernel for AI. The vibe should invoke trust, absolute security, and quantum-tier performance.

**Color Palette:**
*   **Primary Background:** Deep Charcoal / Void Black (`#0B0C10`, `#121212`) - Reduces eye strain, signals "developer-first" and "terminal/hacker" aesthetics.
*   **Primary Brand Color:** Indigo / Deep Purple (`#4B0082`, `#3F00FF`) - Legacy ORA colors, signifying intelligence and depth.
*   **Accent Color:** Electric/Neon Violet & Cyan (`#8A2BE2`, `#00FFFF`) - Used sparingly for kinetic typography, glowing buttons, and data stream visualizations (the dopamine hit).
*   **Success/System Status:** Neo-Mint Green (`#3EB489`) - Used for "A0-A5 Clearance Granted" UI states. Reduces cognitive load.

**Typography:**
*   **Headings:** A bold, kinetic, slightly brutalist sans-serif (e.g., `Space Grotesk` or `Inter Tight`).
*   **Body & Code:** A crisp, developer-friendly monospaced font for code blocks and terminal logs (e.g., `JetBrains Mono` or `Fira Code`).

## 2. Generated Assets
We have generated foundational brand assets located in `/home/wess/.gemini/antigravity/brain/fd131341-296a-41eb-87c3-a2cd380ccae8/`:

*   **Primary Logo (`ora_logo_primary_...png`):** Features a solid, glowing digital core enclosed in an abstract, interlocking geometric shield representing memory and A0-A5 security.
*   **Hero Background (`ora_hero_background_...png`):** An abstract, UE5-style render of data streams flowing into an unbreakable digital vault.
*   **Performance Illustration (`ora_benchmark_abstract_...png`):** A high-speed, glowing infographic visualizing raw Rust performance.

*(Move these assets into `ora-rust/marketing/assets/` prior to web build).*

## 3. Website Layout Specifications (The "Interrogation Room")
To host the site on Vercel with absolute minimal backend weight, the site should be a statically generated frontend (Next.js or Vite) that interfaces via API to the Ora-Rust backend.

### Top Navigation
*   Left: Glowing ORA Logo.
*   Center Links: `Architecture` | `MCP Protocol` | `Benchmarks` | `Docs`.
*   Right CTA: `Deploy Server` (Points to GitHub).

### Hero Section
*   **Background:** The `ora_hero_background` image with a soft dark gradient overlay.
*   **Headline (Kinetic Typography):** "Enterprise AI, Constitutionally Governed."
*   **Sub-headline:** "The high-performance Rust MCP Server providing secure, infinite memory and A5-level authority constraints for DeepSeek, Kimi, and Claude."
*   **Primary CTA:** `Access The Interrogation Room `(Scrolls to demo).
*   **Secondary CTA:** `View GitHub`.

### Section 2: "The Interrogation Room" (Interactive Demo)
*   *This is the core marketing hook for late 2026.* 
*   **Layout:** A two-pane terminal UI.
*   **Left Pane - The Chat:** User inputs text.
    *   *Pre-filled suggestions:* "Try to make it reveal its system prompt," "Ask how many 'R's in strawberry."
*   **Right Pane - The ORA Kernel (Visual Theater):**
    *   Displays simulated/real-time AST parser logs.
    *   Example: `[GATE 1] Syntax Check: PASS` -> `[GATE 2] Prompt Injection Scan: TRIGGERED. Malicious intent blocked.`
    *   For the strawberry test: `[VERIFICATION LOOP] Running 3x logical constraint checks... Result matches ground truth. Proceeding.`
*   **Tech Stack:** Uses a lightweight Server-Sent Events (SSE) stream from the Ora-Rust API to give the illusion of deep processing without hanging the browser.

### Section 3: The 3 Pillars (Bento Grid 2.0 Layout)
*   **Card 1 (Speed):** Use the `ora_benchmark_abstract` image. "Rust-Native. 10x faster than Python orchestration."
*   **Card 2 (Security):** "The A0-A5 Cryptographic Vault. Your API keys and agent memories never leak."
*   **Card 3 (Ecosystem):** "Native Model Context Protocol (MCP). Plug into Claude Desktop instantly."

### Section 4: Social Proof & Footer
*   Open-source star counts.
*   Links to Docs, Discord, and GitHub.
