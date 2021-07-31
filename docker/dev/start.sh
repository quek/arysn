#!/bin/sh

TRACE_SQL=1
export TRACE_SQL
cd /app/arysn-test
/wait-for-it.sh db:5432 -- \
  cargo watch --ignore tmp --ignore src/generated -x 'test --features "with-tokio-1_x-gis" -- --nocapture'
