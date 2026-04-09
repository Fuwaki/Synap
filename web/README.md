# Synap Web

基于 Leptos + Axum 的全栈 RS (Rust) Web 应用。

## 技术栈

- **Leptos** - Rust 全栈 Web 框架（SSR + Hydrate）
- **Axum** - 异步 Web 框架（SSR 服务器）
- **cargo-leptos** - 构建工具

## 项目结构

```
web/
├── src/
│   ├── app/           # Leptos 应用组件
│   ├── lib.rs         # 库入口
│   ├── main.rs        # 服务端入口
│   └── server.rs      # Axum 服务器
├── style/             # SCSS 样式
├── public/            # 静态资源
└── end2end/          # Playwright E2E 测试
```

## 运行

```bash
# 开发模式（热重载）
cargo leptos watch --workspace -p web

# 生产构建
cargo leptos build --release -p web
```

## 功能特性

- 支持 SSR (Server-Side Rendering) 和 Hydrate 客户端 hydration
- 依赖 `synap-core` 共享核心业务逻辑
- SCSS 样式编译