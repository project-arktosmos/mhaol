#!/bin/bash
# Claude Code PreToolUse hook: runs CI checks before git commit
# This script receives the tool input as JSON on stdin.
# If the Bash command is a git commit, it runs lint/check/build/test first.
# Exit non-zero to block the commit if checks fail.

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | grep -o '"command":"[^"]*"' | head -1 | sed 's/"command":"//;s/"$//')

# Only intercept git commit commands
if echo "$COMMAND" | grep -q "git commit"; then
  echo "Running pre-commit checks: pnpm lint && pnpm check && pnpm build && pnpm test"
  pnpm lint && pnpm check && pnpm build && pnpm test
  exit $?
fi

# Allow all other commands
exit 0
