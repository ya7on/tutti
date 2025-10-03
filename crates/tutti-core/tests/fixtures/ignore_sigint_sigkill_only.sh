#!/usr/bin/env bash
# ignore_sigint_sigkill_only.sh — ignores SIGINT, checking that SIGKILL kills the process
# trap '' ignores SIGINT;

trap '' SIGINT

echo "Process started, SIGINT ignored. To kill: use SIGKILL (kill -9)."
echo "PID=$$"

count=0
while true; do
  ((count++))
  echo "alive $count"
  sleep 1
done
