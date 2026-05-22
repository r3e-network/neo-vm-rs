# neo-vm-rs 技术学习指南

这份指南把 `neo-vm-rs` 当作 Neo N4 的一个技术单元来解释。它不是源码阅读图，而是帮助读者理解：这个单元负责什么、哪些技术假设保证它正确、数据如何移动、状态如何变化、证据如何被验证、它如何接入 Neo N4 的整体架构。

## 技术契约

| 维度 | 含义 |
| --- | --- |
| 层级 | 共享虚拟机核心 |
| 目的 | NeoVM 3.9.x 语义的 Rust 共享核心，供 RISC-V 与 zkVM 路径复用。 |
| 输入 | NeoVM 字节码 <br> 初始栈 <br> 系统调用宿主回调 |
| 职责 | 解码标准操作码 <br> 执行栈与状态语义 <br> 暴露可复用运行时 API |
| 输出 | halt/fault 结果 <br> 最终栈 <br> gas/计费证据 |
| 消费方 | neo-riscv-vm <br> neo-zkvm <br> Neo N4 执行核心 |

## 图表集合

| # | 图 | 学什么 |
| --- | --- | --- |
| 1 | [系统位置图](figures/position.zh.svg) | 它在 Neo N4 中的位置。 |
| 2 | [技术原理图](figures/principles.zh.svg) | 保证设计正确的技术规则。 |
| 3 | [概念架构图](figures/architecture.zh.svg) | 主要技术块和边界。 |
| 4 | [工作流图](figures/workflow.zh.svg) | 运行时的有序过程。 |
| 5 | [数据流图](figures/dataflow.zh.svg) | 信息、承诺和证据如何移动。 |
| 6 | [状态模型图](figures/state-model.zh.svg) | 状态归属、转换和终局性。 |
| 7 | [证明与证据流图](figures/proof-flow.zh.svg) | 声明如何变成可验证证据。 |
| 8 | [信任边界图](figures/trust-boundaries.zh.svg) | 哪些内容被信任、检查、拒绝或观测。 |
| 9 | [集成关系图](figures/integration-map.zh.svg) | 该单元如何接入更大的 N4 栈。 |
| 10 | [运行生命周期图](figures/lifecycle.zh.svg) | 从配置到执行、证据和运维的生命周期。 |

## 架构模型

`neo-vm-rs` 接收 NeoVM 字节码 | 初始栈 | 系统调用宿主回调，拥有的边界是：解码标准操作码 | 执行栈与状态语义 | 暴露可复用运行时 API。它输出 halt/fault 结果 | 最终栈 | gas/计费证据，然后由 neo-riscv-vm | neo-zkvm | Neo N4 执行核心 消费。

分层规则：VM 语义与 host 上下文和链策略分离。

## 工作流

1. 加载脚本
2. 解码 OpCode
3. 执行语义
4. 调用宿主 syscall
5. 返回 VM 结果

失败路径：非法 opcode、栈不匹配、gas 耗尽、syscall 拒绝或 fault。

## 数据流

1. 字节码 + 栈
2. 共享运行时
3. 状态转换
4. 执行证据

承诺信号：脚本哈希、最终栈摘要、halt/fault 状态和 gas。

## 状态、证明和信任

- 状态转换：opcode 语义、栈规则、gas 和 syscall 合同定义状态转换。
- 终局条件：VM 成功 halt 且 host 接受最终栈和副作用。
- 信任模型：信任标准 NeoVM 语义，不信任脚本作者。
- 验证边界：opcode、栈类型、gas、jump target 和 syscall 响应必须合法。
- 重放与顺序：VM 上下文绑定脚本和 host 状态。

## 集成和运行

- NeoFS DA：NeoFS 保存批次数据、见证或轨迹摘要以及可取回证据。
- 证明系统：证明系统把 L2 执行声明压缩为可验证证据。
- Gateway/API：Gateway 负责用户路由、查询、提交和健康状态聚合。
- 桥与异构链：桥规则统一 L1-L2、L2-L2 和异构链消息与资产。
- 可观测证据：opcode 进度、gas、栈摘要、fault 原因和 syscall 边界。

在 Neo N4 仓库根目录重新生成这些技术图：

```powershell
python tools/docs/generate_crate_visual_docs.py
```
