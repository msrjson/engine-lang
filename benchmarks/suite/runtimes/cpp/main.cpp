/*
 * C++ Confrontation Benchmark Workload (C++20 / Clang/GCC -O3)
 * Multi-Stack Mission-Critical Benchmark
 * Author: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group
 * License: Apache 2.0
 */

#include <iostream>
#include <vector>
#include <string>
#include <chrono>
#include <cstdint>
#include <cassert>
#include <cstdlib>

struct LedgerRecord {
    int64_t id;
    int64_t balance;
    int64_t tx_count;
    std::string state_hash;

    LedgerRecord(int64_t id_, int64_t balance_, std::string hash_)
        : id(id_), balance(balance_), tx_count(0), state_hash(std::move(hash_)) {
        assert(id >= 1 && id <= 10000000);
        assert(balance >= 0 && balance <= 1000000000);
        assert(state_hash.length() == 64);
    }

    inline bool mutate_record(int64_t delta) noexcept {
        if (delta < 0 && balance < (-delta)) {
            return false;
        }
        balance += delta;
        tx_count += 1;
        assert(balance >= 0);
        return true;
    }
};

int main() {
    long iterations = 100000;
    const char* env_iter = std::getenv("ITERATIONS");
    if (env_iter) {
        iterations = std::atol(env_iter);
        if (iterations <= 0) iterations = 100000;
    }

    const std::string dummy_hash(64, 'a');

    auto t_start = std::chrono::high_resolution_clock::now();

    std::vector<LedgerRecord> records;
    records.reserve(iterations);
    for (long i = 1; i <= iterations; ++i) {
        records.emplace_back(i, 1000, dummy_hash);
    }

    auto t_setup = std::chrono::high_resolution_clock::now();

    long success_count = 0;
    for (auto& record : records) {
        if (record.mutate_record(50)) {
            success_count++;
        }
    }

    auto t_end = std::chrono::high_resolution_clock::now();

    std::chrono::duration<double, std::milli> setup_ms = t_setup - t_start;
    std::chrono::duration<double, std::milli> mutate_ms = t_end - t_setup;
    std::chrono::duration<double, std::milli> total_ms = t_end - t_start;
    double ops_sec = static_cast<double>(iterations) / (mutate_ms.count() / 1000.0);

    std::cout << "{\n";
    std::cout << "  \"language\": \"cpp\",\n";
    std::cout << "  \"iterations\": " << iterations << ",\n";
    std::cout << "  \"setup_time_ms\": " << setup_ms.count() << ",\n";
    std::cout << "  \"mutate_time_ms\": " << mutate_ms.count() << ",\n";
    std::cout << "  \"total_time_ms\": " << total_ms.count() << ",\n";
    std::cout << "  \"ops_per_sec\": " << ops_sec << ",\n";
    std::cout << "  \"success_count\": " << success_count << "\n";
    std::cout << "}\n";

    return 0;
}

