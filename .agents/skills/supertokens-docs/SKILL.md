---
name: supertokens-docs
description: "Use the SuperTokens documentation MCP when implementing or changing SuperTokens integrations, SDK configuration, sessions, recipes, or deployments."
---

# SuperTokens documentation

Use the official SuperTokens documentation MCP for current guidance whenever changing SuperTokens-related code or configuration.

Connect to `https://supertokens.com/docs/mcp`. Search the docs first with `search_docs`, then retrieve relevant pages with `get_page`. The server also offers `list_pages` and `get_navigation` for discovery. Its documented tools are read-only.

Apply the documentation to the repository's existing architecture and the requested change. Check the SDK/API behavior against installed package versions and preserve the project's security boundaries. Do not expose client secrets, Core API keys, session tokens, or verifier credentials to browser code or logs.

If this MCP endpoint is unavailable in the current client, use the `.md` documentation routes advertised by `https://supertokens.com/docs/agent-readability.json` and clearly report that the MCP could not be used.
