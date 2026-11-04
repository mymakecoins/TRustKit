---
description: "Non-negotiable constitutional invariants for all development and operations in TRustKit"
alwaysApply: true
---

# Constitutional Invariants of TRustKit

The following rules apply unconditionally across the entire repository:

1. Prohibition of Emojis:
   - Absolutely no emojis or graphical icons in source code, comments, documentation, markdown files, command outputs, or git commits.

2. Prohibition of AI Attribution Trailers:
   - Absolutely no git trailer lines containing AI authorship metadata (e.g., Co-Authored-By, Generated-by, or model signatures).

3. Mechanized Verification:
   - All state transitions and compliance checks must be validated by the native trustkit binary.

4. Branch and PR Safeguards:
   - Direct commits and pushes to protected branches (main, master, staging, develop) are forbidden.
   - Pull requests must only be opened when explicitly instructed by the user.
