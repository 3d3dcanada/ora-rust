# Week 2: Email Sequence - The Developer Onboarding
*Target: Developers who signed up for the free tier or starred the repo.*

---
## Email 1: The Quick Start (Send 1 hour after Signup)
**Subject:** Welcome to ORA. Your A5 Vault is ready.

Hey {{First_Name}}, 

Thanks for joining the ORA deployment. You are now part of a community that believes AI should be fast, and it should be secure by default.

The easiest way to see what ORA can do is to hook it up to Claude Desktop via the Model Context Protocol (MCP). 

**Takes 2 minutes:**
1. Open terminal: `brew install ora-cli` (or `cargo install ora-rust`)
2. Run: `ora setup claude`
3. Restart Claude Desktop.

You'll see the ORA icon appear in your Claude tools. From that moment on, your local memory retrievals are fully governed by our Rust security kernel. 

Hit reply if you run into any issues. I handle support directly. 

Cheers,
Wess

---
## Email 2: The Core Feature (Send Day 3)
**Subject:** 1.8 milliseconds. That's it.

Hey {{First_Name}},

Most developers we talk to think that adding a "military-grade security kernel" to their AI agents is going to slow their app down to a crawl. 

If we built this in Python, they'd be right. 

Because ORA is written entirely in Rust, our AST (Abstract Syntax Tree) layer scans every single prompt for malicious injection loops in just **1.8 milliseconds**. 

*(Embed `ora_latency_graph.png`)*

You get 100% Zero-Trust security with zero UX penalty. 

If you haven't deployed your vault yet, copy-paste this one line into your terminal to start the local server:
`ora serve --mcp-mode`

Best,
Wess

---
## Email 3: The Enterprise Upsell (Send Day 7)
**Subject:** Taking ORA to production at {{Company_Name}}

Hey {{First_Name}},

Hopefully you've had a chance to play around with ORA locally and see the prompt injection shields at work. 

When you're ready to move your team's agents into production, running a local vector DB and managing cryptographic vaults relies on a lot of DevOps overhead. 

We offer a **Dedicated Managed Cloud** for ORA. We host the post-quantum vault, manage the Pulz-Rust vector memory scaling, and provide a 99.99% SLA for your enterprise agents. 

Check out the pricing tiers here if your CTO is looking to harden your AI stack this quarter. 

Stay secure,
Wess
