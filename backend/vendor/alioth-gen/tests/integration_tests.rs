//! 集成测试主入口
//!
//! 运行所有 E2E 测试套件

mod e2e;
mod regression;

// 重新导出 E2E 测试模块
pub use e2e::*;

/// 运行所有测试的辅助函数
#[cfg(test)]
mod test_runner {
    use std::time::Instant;

    #[test]
    fn run_all_integration_tests() {
        println!("Running Phase 28 Integration Tests...");
        println!("=====================================");

        let start = Instant::now();

        // Tests are automatically run by cargo test
        // This is a placeholder for potential future test orchestration

        println!("All integration tests completed in {:?}", start.elapsed());
    }
}
