#!/usr/bin/env bash
# TRustKit PreToolUse Hook Integration for Antigravity & Claude Code
# Latency budget: < 10ms. Intercepts forbidden commands, emojis, and unformatted commits.

set -euo pipefail

# Read payload from stdin and evaluate with python
COMMAND_TO_EVAL=$(python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    tool_call = data.get('toolCall', {})
    args = tool_call.get('args', {})
    cmd = args.get('CommandLine', '')
    sys.stdout.write(cmd)
except Exception:
    sys.stdout.write('')
")

# If no command, allow
if [ -z "$COMMAND_TO_EVAL" ]; then
    printf '{"decision":"allow"}\n'
    exit 0
fi

# Run trustkit hook pre-tool
TRUSTKIT_BIN=$(which trustkit 2>/dev/null || echo "/home/mmc/.local/bin/trustkit")

if [ ! -x "$TRUSTKIT_BIN" ]; then
    printf '{"decision":"allow"}\n'
    exit 0
fi

# Evaluate command with trustkit
OUTPUT=$("$TRUSTKIT_BIN" hook pre-tool --command "$COMMAND_TO_EVAL" --json 2>&1) || EXIT_CODE=$?
EXIT_CODE=${EXIT_CODE:-0}

if [ "$EXIT_CODE" -eq 0 ]; then
    printf '{"decision":"allow"}\n'
    exit 0
else
    # Extract violation reason if possible
    REASON=$(python3 -c "
import sys, json
raw = sys.argv[1] if len(sys.argv) > 1 else ''
try:
    data = json.loads(raw)
    reason = data.get('violation', {}).get('reason', 'Command blocked by TRustKit governance policy.')
    print(reason)
except Exception:
    print('Command blocked by TRustKit governance policy.')
" "$OUTPUT" 2>/dev/null || echo "Command blocked by TRustKit governance policy.")

    ESCAPED_REASON=$(printf '%s' "$REASON" | sed 's/"/\\"/g')
    printf '{"decision":"deny","reason":"TRustKit Interception: %s"}\n' "$ESCAPED_REASON"
    exit 0
fi
