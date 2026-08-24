#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${RESONIQUE_BASE_URL:-http://127.0.0.1:3000}"

request() {
  curl --silent --show-error --fail-with-body \
    --header 'content-type: application/json' \
    "$@"
}

status() {
  curl --silent --output /dev/null --write-out '%{http_code}' \
    --header 'content-type: application/json' \
    "$@"
}

test_status() {
  local expected="$1"
  shift
  local actual
  actual="$(status "$@")"

  if [[ "$actual" != "$expected" ]]; then
    echo "Expected HTTP $expected, received HTTP $actual"
    exit 1
  fi
}

echo "Checking health endpoint..."
test_status 200 "$BASE_URL/health"

echo "Checking malformed JSON..."
test_status 400 -X POST "$BASE_URL/search" -d '{invalid'

echo "Checking empty query..."
test_status 400 -X POST "$BASE_URL/search" \
  -d '{"collection":"default","query":[],"top_k":1}'

echo "Checking invalid top_k..."
test_status 400 -X POST "$BASE_URL/search" \
  -d '{"collection":"default","query":[1.0],"top_k":0}'

echo "Checking unknown collection..."
test_status 404 -X POST "$BASE_URL/search" \
  -d '{"collection":"missing","query":[1.0],"top_k":1}'

echo "Checking path-like collection input..."
test_status 404 -X POST "$BASE_URL/search" \
  -d '{"collection":"../../etc/passwd","query":[1.0],"top_k":1}'

echo "Security smoke tests passed."