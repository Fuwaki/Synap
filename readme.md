# Synap

一个基于有向无环图（DAG）的极简思维捕获与路由中枢。

本项目旨在消除传统笔记软件中由于“强制分类”和“树形目录”带来的心智负担。它不要求你在记录前进行结构化思考，而是忠实地记录意识的流转、发散与收束。

## 核心理念

* 零阻力捕获：放弃文件夹概念，即开即写，将灵感捕获与系统化整理彻底解耦。
* 绝对不可变账本：放弃传统关系型数据库，拥抱纯 Rust KV 数据库（Key-Value）。底层仅存在“文本块”与“语义指针”，所有操作严格遵循只读追加（Append-Only），绝不执行原地的修改与删除。
* 读时过滤视图：冲突解决不再发生于写入或同步阶段。系统通过“滤网（Reducer）”在读取时动态跨越死神指针和重定向指针，将碎片化的历史事实渲染为连贯的网状图谱。
* 极简去中心化同步：由于底层数据的不可变性与全局唯一 ID，多端同步被彻底降维为毫无心智负担的“集合求并集（Set Union）”，从物理法则上断绝并发冲突。
* 类 ZFS 滞后回收：借鉴写时复制（CoW）哲学，思维的废弃与修改仅产生新的指针。真正的物理存储释放，交由本地异步的标记-清除（Mark and Sweep）垃圾回收机制在离线时静默完成。

## 工程架构

本项目采用 Monorepo 组织结构。当前 Rust workspace 成员为 `core`、`coreffi`、`cli`、`desktop`、`xtask`；`android` 与 `web` 分别由 Gradle 与 Vite 管理。

* `core/`：Rust 逻辑内核。负责纯 Rust KV 数据落盘、不可变 DAG 状态机维护、读时过滤渲染算法以及同步协议。
* `coreffi/`：Rust FFI 封装层。通过 UniFFI 将 `core` 暴露给原生平台调用。
* `cli/`：Rust 命令行前端。提供纯终端环境下的捕获、检索、图谱与同步入口。
* `desktop/`：Rust 桌面端 UI。
* `android/`：Kotlin 原生应用。Gradle 在构建期编译 `coreffi`，并接入自动生成的 UniFFI Kotlin 绑定。
* `xtask/`：Rust 工具目标。当前主要用于从 `coreffi/src/synap.udl` 生成 Android 侧 UniFFI Kotlin 绑定。
* `web/`：Svelte + Vite 前端实验壳层。当前不在 Rust workspace 中。

## 构建与运行

确保已安装 Rust 工具链。若需要构建 Android，还需要可用的 JDK、Android SDK 与 NDK。

进入工作区根目录，编译命令行工具：

```bash
cargo build --release -p synap-cli
./target/release/synap --help
```

运行桌面端：

```bash
cargo run -p synap-desktop
```

构建 Android 调试包：

```bash
cd android
./gradlew assembleDebug
```

Android 的 `preBuild` 会先执行两件事：

* 编译 `coreffi` 对应的 Android 动态库。
* 通过 `cargo run -p xtask -- gen-uniffi-kotlin ...` 生成 Kotlin UniFFI 绑定到 `build/generated/...`。

启动 Web 实验壳层：

```bash
cd web
pnpm install
pnpm dev
```

## 仓库约定

这个仓库是单仓多端，但不要求每个 feature 一次性完成全平台适配。主线维护的是“共享核心 + 当前正在维护的平台集合”，不是“所有壳层永远同步完成”。约定如下：

* 不直接在 `master` 上做长期开发。新功能、新重构、新实验统一从 `feat/*`、`refactor/*`、`spike/*` 分支开始，`master` 只保留可运行、可回退的提交。
* 一个跨端功能按“层”拆提交，而不是按“今天改了什么”拆提交。推荐顺序是 `core/` -> `coreffi/` -> `android|desktop|cli|web/` -> `docs|build`。
* 一个 feature 可以逐个平台落地，但合入 `master` 的提交不应该把本次涉及的平台直接打坏。
* 允许本地存在 WIP 提交，但整理进主线前需要压成一串可解释、可回退的提交。
* 构建产物、本地数据库、UniFFI 生成绑定、`jniLibs` 与其他本地产物不进入版本管理；它们应由构建流程重新生成。
* 长周期并行开发优先使用 `git worktree`，不要把所有事情都堆在一个工作区里。

一个典型的跨端功能建议这样推进：

```bash
git switch -c feat/note-tag-flow
git add core coreffi
git commit -m "feat(core): add note tag service flow"
git add android/app/src/main/java android/app/src/main/AndroidManifest.xml android/app/build.gradle.kts
git commit -m "feat(android): wire note tag flow into app"
```

如果某个功能会做很多天，额外开一个工作树会更稳：

```bash
git worktree add ../synap-note-tag feat/note-tag-flow
```
