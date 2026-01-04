# OnePanel CI/CD Manager

OnePanel CI/CD Manager 是一个用于管理本地 Git 仓库并集成 OnePanel 进行持续集成/持续部署（CI/CD）的工具。它由高性能的 Rust 后端和现代化的 Svelte 5 前端组成。

## ✨ 当前功能 (Current Features)

- **本地仓库管理**：
  - 支持添加任意本地 Git 仓库路径进行追踪。
  - 支持从列表中移除仓库。
- **Git 状态监控**：
  - 实时显示当前分支 (Branch)。
  - 显示最后一次提交的信息和时间。
  - **状态检查**：直观展示仓库是 "Clean" (干净) 还是 "Dirty" (有未提交更改)。
- **现代化 UI**：
  - 基于 Svelte 5 Runes API 构建。
  - 使用 Shadcn-Svelte 组件库和 Tailwind CSS v4，提供极佳的视觉体验和交互。
- **高性能后端**：
  - 使用 Rust (Axum) 提供 API 服务。
  - 集成 `git2` 库进行高效的 Git 操作。
  - 集成 Scalar 自动生成交互式 API 文档。

## 🛠️ 技术栈 (Tech Stack)

- **Backend**: Rust, Axum, Tokio, git2, Utoipa (OpenAPI)
- **Frontend**: Svelte 5, Vite, Tailwind CSS v4, Shadcn-Svelte
- **Tools**: 1Panel (目标集成平台), Docker

## 🚀 快速开始 (Getting Started)

### 后端 (Backend)
```bash
cd backend
cargo run
# 服务运行在 http://localhost:3000
# API 文档: http://localhost:3000/scalar
```

### 前端 (Frontend)
```bash
cd frontend
npm run dev
# 服务运行在 http://localhost:5173
```

## 🗺️ 未来计划 (Roadmap)

1.  **数据持久化**：
    - [ ] 目前仅支持内存存储，重启后数据丢失。计划添加 SQLite 或 JSON 文件存储，持久化保存仓库列表。
2.  **Docker 集成**：
    - [ ] 支持根据仓库内的 `Dockerfile` 构建镜像。
    - [ ] 管理本地 Docker 镜像和容器。
3.  **OnePanel 对接**：
    - [ ] 集成 OnePanel API，实现自动化应用部署。
    - [ ] 支持将构建好的镜像推送到 OnePanel 环境。
4.  **CI/CD 流程自动化**：
    - [ ] 定义简单的构建/部署流水线。
    - [ ] 监听 Git 提交自动触发构建。
5.  **Git 功能增强**：
    - [ ] 查看提交历史详情。
    - [ ] 支持简单的 Git 操作（如 Pull, Push）。
