#!/usr/bin/env bash
# both.sh — interleaved stdout и stderr

for i in {1..8}; do
  if (( i % 2 == 0 )); then
    echo "STDOUT: message $i"
  else
    echo "STDERR: message $i" >&2
  fi
  # small pause to see interleaving
  sleep 0.15
done

echo "both.sh done"
