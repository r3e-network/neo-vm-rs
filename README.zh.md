# neo-vm-rs

<!-- N4-CRATE-VISUAL-GUIDE-ZH:START -->

## 可视化学习指南

这些图是 `neo-vm-rs` 自己目录下的 crate 专属学习资料，用来说明它在 Neo N4 中的位置、自己负责的技术边界、内部工作流，以及数据如何流经它。

| 视图 | 图片 | 源文件 |
| --- | --- | --- |
| 在 Neo N4 中的位置 | ![位置](docs/figures/position.zh.svg) | [Mermaid](docs/figures/position.zh.mmd) |
| 技术原理 | ![技术原理](docs/figures/principles.zh.svg) | [Mermaid](docs/figures/principles.zh.mmd) |
| 架构 | ![架构](docs/figures/architecture.zh.svg) | [Mermaid](docs/figures/architecture.zh.mmd) |
| 工作流 | ![工作流](docs/figures/workflow.zh.svg) | [Mermaid](docs/figures/workflow.zh.mmd) |
| 数据流 | ![数据流](docs/figures/dataflow.zh.svg) | [Mermaid](docs/figures/dataflow.zh.mmd) |

### 在 Neo N4 中的作用

- **层级:** 共享虚拟机核心
- **目的:** NeoVM 3.9.x 语义的 Rust 共享核心，供 RISC-V 与 zkVM 路径复用。
- **主要输入:** NeoVM 字节码、初始栈、系统调用宿主回调
- **主要输出:** halt/fault 结果、最终栈、gas/计费证据
- **下游使用者:** neo-riscv-vm、neo-zkvm、Neo N4 执行核心

### 边界与职责

- **本 crate 负责:** 解码标准操作码、执行栈与状态语义、暴露可复用运行时 API
- **本 crate 消费:** NeoVM 字节码、初始栈、系统调用宿主回调
- **本 crate 产出:** halt/fault 结果、最终栈、gas/计费证据
- **主要被谁使用:** neo-riscv-vm、neo-zkvm、Neo N4 执行核心

### 学习路径

1. 先看位置图，明确这个 crate 为什么存在、上游是谁、下游是谁。
2. 再看技术原理图，理解它的核心不变量、职责边界和维护规则。
3. 然后看架构图，把公开入口、内部组件、依赖边界和输出产物串起来。
4. 最后看工作流和数据流，再进入源码和测试文件会更容易理解。

<!-- N4-CRATE-VISUAL-GUIDE-ZH:END -->
