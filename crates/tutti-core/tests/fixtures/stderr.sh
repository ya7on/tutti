#!/usr/bin/env bash
# stderr.sh — prints messages to stderr and exits

echo "ERROR: line 1" >&2
sleep 0.2
echo "ERROR: line 2" >&2
sleep 0.2
echo "ERROR: stderr.sh finished" >&2
