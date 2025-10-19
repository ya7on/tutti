#!/usr/bin/env bash
# infinite_sigint_kills.sh - endless loop without signal handlers
# expects that SIGINT (Ctrl-C or kill -INT PID) kills script

echo "Endless loop started"
count=0
while true; do
  ((count++))
  echo "tick $count"
  sleep 1
done
