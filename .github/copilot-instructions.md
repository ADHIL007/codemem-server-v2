## CodeMem Integration

This project uses [codemem](https://github.com/codemem/codemem) for persistent memory across AI coding sessions. A codemem MCP server is configured in `.mcp.json`.

### MCP Tools  -  Full Reference

#### Memory Operations
- **`store_memory`**  -  Store a new memory (params: `content`, `memory_type`, `importance`, `tags`, `links`, `namespace`)
- **`recall`**  -  Semantic search over memories (params: `query`, `k`, `memory_type`, `namespace`, `expand`, `expansion_depth`, `include_impact`, `min_importance`, `min_confidence`, `exclude_tags`)
- **`delete_memory`**  -  Delete a memory by ID (params: `id`)
- **`refine_memory`**  -  Update/evolve a memory (params: `id`, `content`, `destructive`)
- **`split_memory`**  -  Split a memory into parts (params: `id`, `parts`)
- **`merge_memories`**  -  Merge multiple memories into one (params: `ids`, `content`)
- **`associate_memories`**  -  Link two memories (params: `source_id`, `target_id`, `relationship`)
- **`get_decision_chain`**  -  Follow decision evolution over time (params: `topic`, `file_path`)
- **`get_node_memories`**  -  Get memories linked to a graph node (params: `node_id`)
- **`node_coverage`**  -  Check if nodes have documentation (params: `node_ids`)

#### Code Search & Symbols
- **`search_code`**  -  Find code by name or concept (params: `query`, `mode`: "text"|"semantic"|"hybrid", `k`, `kind`)
- **`get_symbol_info`**  -  Get full symbol details (params: `qualified_name`)
- **`get_symbol_graph`**  -  Trace call graphs/dependencies (params: `qualified_name`, `direction`: "incoming"|"outgoing"|"both", `depth`)

#### Graph Traversal
- **`graph_traverse`**  -  Walk relationships from any node (params: `start_id`, `max_depth`, `algorithm`, `include_relationships`, `exclude_kinds`, `include_kinds`)
- **`summary_tree`**  -  Browse file/package hierarchy (params: `start_id`, `max_depth`)
- **`find_important_nodes`**  -  Find most critical symbols by PageRank (params: `top_k`, `include_kinds`, `damping`)
- **`find_related_groups`**  -  Find related symbol clusters via Louvain (params: `resolution`)
- **`get_cross_repo`**  -  Cross-repository memories (params: `namespace`)

#### Analysis & Health
- **`codemem_status`**  -  Check graph size and health (params: `include`: ["stats", "health", "metrics"])
- **`detect_patterns`**  -  Detect recurring patterns (params: `min_frequency`)
- **`consolidate`**  -  Deduplicate/clean memories (params: `mode`: "cluster"|"forget"|"creative"|"decay"|"summarize"|"auto", `similarity_threshold`, `importance_threshold`, `threshold_days`, `cluster_size`)

#### Namespace & Session
- **`list_namespaces`**  -  List all namespaces
- **`namespace_stats`**  -  Stats for a namespace (params: `namespace`)
- **`delete_namespace`**  -  Delete a namespace (params: `namespace`)
- **`session_checkpoint`**  -  Session progress snapshot (params: `session_id`)
- **`session_context`**  -  Get session context (params: `namespace`, `k`)

### Best Practices

1. **Start of session**: Use `recall` to check for relevant existing memories before solving problems
2. **Architecture decisions**: Store with `store_memory` (type: "decision", importance: 0.8+)
3. **Discovered patterns**: Store with type: "pattern" and link to relevant nodes
4. **Code understanding**: Use `search_code` (semantic) + `get_symbol_graph` to understand impact
5. **Before changes**: Check `get_symbol_graph` direction: "incoming" to understand blast radius
6. **After refactors**: Use `consolidate` mode: "cluster" to deduplicate outdated memories
7. **Link memories to code**: Always pass `links: ["sym:FunctionName", "file:path/to/file.rs"]` for better retrieval
