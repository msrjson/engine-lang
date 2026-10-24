// Go Confrontation Benchmark Workload (Go 1.22+)
// Multi-Stack Mission-Critical Benchmark
// Author: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group
// License: Apache 2.0

package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"
)

type LedgerRecord struct {
	ID        int64
	Balance   int64
	TxCount   int64
	StateHash string
}

func NewLedgerRecord(id int64, balance int64, hash string) LedgerRecord {
	if id < 1 || id > 10000000 {
		panic("Refinement invariant violated: id")
	}
	if balance < 0 || balance > 1000000000 {
		panic("Refinement invariant violated: balance")
	}
	if len(hash) != 64 {
		panic("Refinement invariant violated: hash")
	}
	return LedgerRecord{
		ID:        id,
		Balance:   balance,
		TxCount:   0,
		StateHash: hash,
	}
}

func (r *LedgerRecord) MutateRecord(delta int64) bool {
	if delta < 0 && r.Balance < (-delta) {
		return false
	}
	r.Balance += delta
	r.TxCount++
	if r.Balance < 0 {
		panic("Post-condition invariant breached")
	}
	return true
}

type Result struct {
	Language     string  `json:"language"`
	Iterations   int     `json:"iterations"`
	SetupTimeMs  float64 `json:"setup_time_ms"`
	MutateTimeMs float64 `json:"mutate_time_ms"`
	TotalTimeMs  float64 `json:"total_time_ms"`
	OpsPerSec    float64 `json:"ops_per_sec"`
	SuccessCount int     `json:"success_count"`
}

func main() {
	iterations := 100000
	if val := os.Getenv("ITERATIONS"); val != "" {
		if parsed, err := strconv.Atoi(val); err == nil && parsed > 0 {
			iterations = parsed
		}
	}

	dummyHash := strings.Repeat("a", 64)

	tStart := time.Now()
	records := make([]LedgerRecord, iterations)
	for i := 0; i < iterations; i++ {
		records[i] = NewLedgerRecord(int64(i+1), 1000, dummyHash)
	}
	tSetup := time.Now()

	successCount := 0
	for i := 0; i < iterations; i++ {
		if records[i].MutateRecord(50) {
			successCount++
		}
	}
	tEnd := time.Now()

	setupMs := float64(tSetup.Sub(tStart).Nanoseconds()) / 1e6
	mutateMs := float64(tEnd.Sub(tSetup).Nanoseconds()) / 1e6
	totalMs := float64(tEnd.Sub(tStart).Nanoseconds()) / 1e6
	opsSec := float64(iterations) / (mutateMs / 1000.0)

	res := Result{
		Language:     "go",
		Iterations:   iterations,
		SetupTimeMs:  setupMs,
		MutateTimeMs: mutateMs,
		TotalTimeMs:  totalMs,
		OpsPerSec:    opsSec,
		SuccessCount: successCount,
	}

	out, _ := json.MarshalIndent(res, "", "  ")
	fmt.Println(string(out))
}

