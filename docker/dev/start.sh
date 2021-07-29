#!/bin/sh

TRACE_SQL=1
export TRACE_SQL
cd /app
/wait-for-it.sh db:5432 -- \
  cargo watch --ignore tmp --ignore src/generated -x 'test --features "gis" -- --nocapture'
