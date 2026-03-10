# Week 1: YouTube & Hashnode Content
*Format: Educational, code-heavy, tutorial based.*

---
## Hashnode Article (Tuesday, 08:00 AM)
**Title:** The Complete Enterprise Guide to the Model Context Protocol (MCP)

**Hero Image:** `ora_hero_background.png`

**Body (Excerpt):**
In late 2025, Anthropic dropped the Model Context Protocol (MCP). It changed everything. Suddenly, Claude Desktop could read your local files, query your Postgres database, and search your GitHub. 

But with great power comes incredible liability. 

If you connect Claude to your local filesystem using a basic, ungoverned MCP server, an indirect prompt injection from a scraped website could trick Claude into reading your `~/.ssh/id_rsa` and exfiltrating it. 

This is why we built ORA. ORA is an MCP server built entirely in Rust, designed around a Zero-Trust architecture... [Continues with full technical tutorial on setting up the `ora_config.json`]

---
## YouTube Video Draft (Thursday, 09:00 AM)
**Thumbnail Idea:** `ora_youtube_thumbnail.png`
**Title:** Secure Your Claude Desktop in 5 Minutes (Zero Trust MCP)

**Video Outline:**
- **0:00 - 0:45 (The Hook):** Show an actual prompt injection attack succeeding on a vulnerable LangChain setup. "Watch this agent leak a database password."
- **0:45 - 2:00 (The Solution):** "Now watch the exact same attack hit the ORA Kernel." (Show the `ora_terminal_ui.png` style logs blocking the attack in real time). "A5 Clearance Denied."
- **2:00 - 6:00 (The Tutorial):** Screen share. Open Terminal. Type `cargo install ora-rust`. Show how to paste the config into `claude_desktop_config.json`. 
- **6:00 - 8:00 (The Deep Dive):** Briefly explain why Rust matters for this (speed + memory safety). Show the `ora_latency_graph.png`. 
- **8:00 (Outro):** Repo link in description. Leave a comment below with your wildest prompt injection attempts.
