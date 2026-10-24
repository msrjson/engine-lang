#!/bin/sh
set -e

echo "==================================================================="
echo "🚀 Engine Forge Test Container (Docker Environment)"
echo "   Engine Lang Compiler: $(enginec version | head -n 1)"
echo "   Target Architecture:  x86_64 / Debian Bookworm"
echo "   Storage Engine:       Single-Level Store (ENG\\x01 / SLS)"
echo "==================================================================="

mkdir -p /app/data

echo "\n[Step 1/3] Formal Z3 SMT Verification of All Framework Modules..."
passed=0
total=0
for f in $(find /app/framework -name "*.eng" | sort); do
    echo "  > Verifying: $f"
    enginec check "$f"
    passed=$((passed+1))
    total=$((total+1))
done
echo "✨ Passed: $passed / $total modules formally verified with Z3 SMT."

echo "\n[Step 2/3] Executing Framework Test Suite..."
enginec run /app/framework/tests/framework_test_suite.eng --db /app/data/engine_test.db

echo "\n[Step 3/3] Executing Framework Application Bootstrap Simulation..."
enginec run /app/framework/app/bootstrap.eng --db /app/data/engine_test.db

echo "\n[Storage Diagnostics] Inspecting Single-Level Store Container..."
enginec inspect /app/data/engine_test.db

echo "\n==================================================================="
echo "✅ ALL DOCKER TEST CONTAINER CHECKS COMPLETED SUCCESSFULLY"
echo "==================================================================="

# If running as entrypoint PID 1 with KEEP_ALIVE=true, keep container running
if [ "$$" -eq 1 ] && [ "$KEEP_ALIVE" = "true" ] && [ "$1" != "once" ]; then
    echo "⚡ Test container daemon active (sleep infinity for interactive testing)."
    exec sleep infinity
else
    echo "🏁 Test execution completed (exit code 0)."
    exit 0
fi

