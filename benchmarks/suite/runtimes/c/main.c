/*
 * C Confrontation Benchmark Workload (C23 / GCC 14 -O3)
 * Multi-Stack Mission-Critical Benchmark
 * Author: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group
 * License: Apache 2.0
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <time.h>
#include <assert.h>

typedef struct {
    int64_t id;
    int64_t balance;
    int64_t tx_count;
    char state_hash[65];
} LedgerRecord;

static inline bool mutate_record(LedgerRecord* record, int64_t delta) {
    if (delta < 0 && record->balance < (-delta)) {
        return false;
    }
    record->balance += delta;
    record->tx_count += 1;
    assert(record->balance >= 0);
    return true;
}

static double get_time_ms(struct timespec start, struct timespec end) {
    return ((end.tv_sec - start.tv_sec) * 1000.0) +
           ((end.tv_nsec - start.tv_nsec) / 1000000.0);
}

int main(int argc, char** argv) {
    long iterations = 100000;
    const char* env_iter = getenv("ITERATIONS");
    if (env_iter) {
        iterations = atol(env_iter);
        if (iterations <= 0) iterations = 100000;
    }

    struct timespec t_start, t_setup, t_end;
    clock_gettime(CLOCK_MONOTONIC, &t_start);

    LedgerRecord* records = (LedgerRecord*)malloc(sizeof(LedgerRecord) * iterations);
    if (!records) {
        fprintf(stderr, "Allocation failed\n");
        return 1;
    }

    for (long i = 0; i < iterations; i++) {
        records[i].id = i + 1;
        records[i].balance = 1000;
        records[i].tx_count = 0;
        memset(records[i].state_hash, 'a', 64);
        records[i].state_hash[64] = '\0';
    }

    clock_gettime(CLOCK_MONOTONIC, &t_setup);

    long success_count = 0;
    for (long i = 0; i < iterations; i++) {
        if (mutate_record(&records[i], 50)) {
            success_count++;
        }
    }

    clock_gettime(CLOCK_MONOTONIC, &t_end);

    double setup_ms = get_time_ms(t_start, t_setup);
    double mutate_ms = get_time_ms(t_setup, t_end);
    double total_ms = get_time_ms(t_start, t_end);
    double ops_sec = (double)iterations / (mutate_ms / 1000.0);

    printf("{\n");
    printf("  \"language\": \"c\",\n");
    printf("  \"iterations\": %ld,\n", iterations);
    printf("  \"setup_time_ms\": %.3f,\n", setup_ms);
    printf("  \"mutate_time_ms\": %.3f,\n", mutate_ms);
    printf("  \"total_time_ms\": %.3f,\n", total_ms);
    printf("  \"ops_per_sec\": %.1f,\n", ops_sec);
    printf("  \"success_count\": %ld\n", success_count);
    printf("}\n");

    free(records);
    return 0;
}

