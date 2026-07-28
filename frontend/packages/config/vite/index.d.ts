import type { UserConfig, UserConfigExport } from "vite";
/**
 * 生成标准模块 Vite 配置
 *
 * 封装所有模块共用的 Vite 配置，消除 16 个模块的复制粘贴。
 *
 * @param options.rootDir 调用模块的 __dirname（vite.config.ts 所在目录）
 * @param options.moduleName 模块名称（用于日志和调试）
 * @param options.port 开发服务器端口（默认 3000）
 * @param options.proxyTarget API 代理目标（默认 http://localhost:8080）
 * @param options.additionalAliases 额外别名（默认空）
 *
 * @example
 * ```ts
 * // vite.config.ts
 * import { defineModuleViteConfig } from "@alioth/config/vite";
 * export default defineModuleViteConfig({ rootDir: __dirname, moduleName: "inventory" });
 * ```
 */
export declare function defineModuleViteConfig(options: {
    rootDir: string;
    moduleName: string;
    port?: number;
    proxyTarget?: string;
    additionalAliases?: Record<string, string>;
}): UserConfigExport;
/**
 * 生成标准模块 Vitest 配置
 *
 * 封装所有模块共用的 Vitest 测试配置，自动解析 Framework 包别名。
 * 默认包含所有 @alioth/* 包的 src 路径映射。
 *
 * @param options.rootDir 调用模块的 __dirname（vitest.config.ts 所在目录）
 * @param options.moduleName 模块名称
 * @param options.additionalAliases 额外别名
 *
 * @example
 * ```ts
 * // vitest.config.ts
 * import { defineModuleVitestConfig } from "@alioth/config/vite";
 * export default defineModuleVitestConfig({ rootDir: __dirname, moduleName: "members" });
 * ```
 */
export declare function defineModuleVitestConfig(options: {
    rootDir: string;
    moduleName: string;
    additionalAliases?: Record<string, string>;
}): UserConfig;
