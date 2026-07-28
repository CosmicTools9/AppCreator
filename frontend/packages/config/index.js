import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
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
export function defineModuleViteConfig(options) {
    const { rootDir, moduleName, port = 3000, proxyTarget = "http://localhost:8080" } = options;
    const projectRoot = path.resolve(rootDir, "../../..");
    // 映射 @alioth/* workspace 包到源码目录（使 vite dev 能直接解析）
    const frameworkAliases = {
        "@alioth/components": path.resolve(projectRoot, "Framework/frontend/components/src/index.ts"),
        "@alioth/hooks": path.resolve(projectRoot, "Framework/frontend/hooks/src/index.ts"),
        "@alioth/utils": path.resolve(projectRoot, "Framework/frontend/utils/src/index.ts"),
        "@alioth/types": path.resolve(projectRoot, "Framework/frontend/types/src/index.ts"),
        "@alioth/api": path.resolve(projectRoot, "Framework/frontend/api/src/index.ts"),
        "@alioth/i18n": path.resolve(projectRoot, "Framework/frontend/i18n/src/index.ts"),
        "@alioth/ontology": path.resolve(projectRoot, "Framework/frontend/ontology/src/index.ts"),
        "@alioth/test-utils": path.resolve(projectRoot, "Framework/frontend/test-utils/src/index.ts"),
    };
    // 用数组形式以支持正则匹配的子路径别名
    const aliasArray = [
        { find: "@", replacement: path.resolve(rootDir, "./src") },
    ];
    // CSS 子路径精确映射（避免 postcss-import 把 @alioth/components 解析为 index.ts 文件）
    const componentsCssPath = path.resolve(projectRoot, "Framework/frontend/components/src");
    aliasArray.push({ find: "@alioth/components/theme-base.css", replacement: path.resolve(componentsCssPath, "theme-base.css") }, { find: "@alioth/components/styles.css", replacement: path.resolve(componentsCssPath, "styles.css") }, { find: "@alioth/components/components.css", replacement: path.resolve(componentsCssPath, "styles.css") });
    // 用户额外别名
    for (const [find, replacement] of Object.entries(options.additionalAliases ?? {})) {
        aliasArray.push({ find, replacement });
    }
    // Framework CSS 资产（精确路径映射，必须在子路径映射之前）
    aliasArray.push({
        find: "@alioth/components/theme-base.css",
        replacement: path.resolve(projectRoot, "Framework/frontend/components/src/theme-base.css"),
    });
    // @alioth/components/* 子路径映射（Vite 8 正则别名已失效，改用显式映射）
    // 注：所有子路径组件均已从主入口 @alioth/components 重导出，
    // 因此业务代码应优先使用主入口导入；以下映射仅用于向后兼容。
    const componentsSrc = path.resolve(projectRoot, "Framework/frontend/components/src");
    const componentSubPaths = [
        "ai", "dashboard", "schedule", "approval",
        "version", "workspace", "system-config",
    ];
    for (const sub of componentSubPaths) {
        aliasArray.push({
            find: `@alioth/components/${sub}`,
            replacement: path.resolve(componentsSrc, "components", sub, "index.ts"),
        });
    }
    // 精确匹配（按 key 长度降序以避免部分匹配）
    const sortedAliases = Object.entries(frameworkAliases).sort(([a], [b]) => b.length - a.length);
    for (const [key, replacement] of sortedAliases) {
        aliasArray.push({ find: key, replacement });
    }
    return defineConfig(({ mode }) => ({
        plugins: [react()],
        resolve: {
            alias: aliasArray,
        },
        server: {
            port,
            proxy: {
                "/api": {
                    target: proxyTarget,
                    changeOrigin: true,
                },
            },
        },
        esbuild: {
            drop: mode === "production" ? ["console", "debugger"] : [],
        },
        build: {
            outDir: "dist",
            sourcemap: true,
            lib: {
                entry: {
                    index: path.resolve(rootDir, "src/main.tsx"),
                    "single-spa": path.resolve(rootDir, "src/single-spa.tsx"),
                },
                formats: ["es"],
                fileName: (format, entryName) => `${entryName}.${format}.js`,
            },
            rollupOptions: {
                external: [
                    "react",
                    "react-dom",
                    "react-router",
                    "single-spa",
                    /^@alioth\//,
                ],
                output: {
                    assetFileNames: (assetInfo) => {
                        if (assetInfo.name === "theme.css" || assetInfo.name?.endsWith(".css")) {
                            return "theme.css";
                        }
                        return "assets/[name]-[hash][extname]";
                    },
                },
            },
        },
    }));
}
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
export function defineModuleVitestConfig(options) {
    const projectRoot = path.resolve(options.rootDir, "../../..");
    const frameworkAliases = {
        "@alioth/components": path.resolve(projectRoot, "Framework/frontend/components/src/index.ts"),
        "@alioth/hooks": path.resolve(projectRoot, "Framework/frontend/hooks/src/index.ts"),
        "@alioth/utils": path.resolve(projectRoot, "Framework/frontend/utils/src/index.ts"),
        "@alioth/types": path.resolve(projectRoot, "Framework/frontend/types/src/index.ts"),
        "@alioth/api": path.resolve(projectRoot, "Framework/frontend/api/src/index.ts"),
        "@alioth/i18n": path.resolve(projectRoot, "Framework/frontend/i18n/src/index.ts"),
        "@alioth/ontology": path.resolve(projectRoot, "Framework/frontend/ontology/src/index.ts"),
        "@alioth/test-utils": path.resolve(projectRoot, "Framework/frontend/test-utils/src/index.ts"),
        "@alioth/test-utils/vitest-setup": path.resolve(projectRoot, "Framework/frontend/test-utils/src/vitest-setup.ts"),
        ...options.additionalAliases,
    };
    const aliasArray = [
        { find: "@", replacement: path.resolve(options.rootDir, "./src") },
    ];
    // Subpath matches for @alioth/components/* — must come BEFORE exact match
    const componentsSrc = path.resolve(projectRoot, "Framework/frontend/components/src");
    aliasArray.push({
        find: /^@alioth\/components\/(.+)$/,
        replacement: componentsSrc + "/components/$1/index.ts",
    });
    // Exact matches
    const sortedAliases = Object.entries(frameworkAliases).sort(([a], [b]) => b.length - a.length);
    for (const [key, value] of sortedAliases) {
        aliasArray.push({ find: key, replacement: value });
    }
    return defineConfig({
        plugins: [react()],
        resolve: {
            alias: aliasArray,
        },
        test: {
            environment: "jsdom",
            globals: true,
            setupFiles: [
                path.resolve(projectRoot, "Framework/frontend/test-utils/src/vitest-setup.ts"),
            ],
            include: ["src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
        },
    });
}
/**
 * @deprecated 使用 {@link defineModuleViteConfig} 替代
 */
export const baseConfig = defineConfig({
    plugins: [react()],
    resolve: {
        alias: {
            "@": path.resolve(process.cwd(), "./src"),
        },
    },
    build: {
        outDir: "dist",
        sourcemap: true,
    },
});
/**
 * @deprecated 使用 {@link defineModuleViteConfig} 替代
 */
export const libConfig = defineConfig({
    ...baseConfig,
    build: {
        lib: {
            entry: path.resolve(process.cwd(), "src/index.ts"),
            formats: ["es"],
            fileName: (format) => `index.${format}.js`,
        },
        rollupOptions: {
            external: ["react", "react-dom"],
        },
    },
});
