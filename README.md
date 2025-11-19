[English](README-en.md)

<div align="center">

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Stars](https://img.shields.io/github/stars/borenchan/fun-cli?logo=github)](https://github.com/borenchan/fun-cli/stargazers)
[![Forks](https://img.shields.io/github/forks/borenchan/fun-cli?logo=github)](https://github.com/borenchan/fun-cli/network/members)
[![GitHub last commit](https://img.shields.io/github/last-commit/mxsm/rocketmq-rust)](https://github.com/mxsm/rocketmq-rust/commits/main)
![GitHub repo size](https://img.shields.io/github/repo-size/borenchan/fun-cli)
![Static Badge](https://img.shields.io/badge/MSRV-1.80.0%2B-25b373)
<br/>
[![Fun CLI](https://img.shields.io/badge/fun-cli)](https://github.com/borenchan/fun-cli)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)

</div>

# 🎮 `fun-cli` —— **你的终端游乐场**  

> *"人生太长，有趣的CLI太少"*  

**`fun-cli` 是一个不断生长的命令行娱乐工具集**，专为那些觉得上班无聊的人打造。在这里，你可以：
- ☔ 随时为你的家人和朋友查询天气
- 🎵 用 `ASCII` 频谱播放收藏夹音乐
- 🕹️ 玩雷霆战机，重获童年快乐（*沉浸式玩耍，按 `Q` 退出*）
- 🟦 俄罗斯方块，经典益智游戏（*方向键控制，空格快速下降*）
- ⚫⚪ 五子棋人机对战，挑战 AI 智慧（*4种难度，地狱模式让你怀疑人生*）
- 💻 监控你的电脑,拥有酷炫的监控面板
- 📝 运行在  **所有平台** （*Windows,MacOS,Linux*）
- ...**或者贡献你的奇思妙想！**  


## 📦 **快速开始**  

### 从发行包安装（支持所有平台）

1. 下载最新版发行包 =>  [release](https://github.com/borenchan/fun-cli/releases)
2. 添加到您的环境变量 `PATH`
3. 恭喜你！可以开始愉快的玩耍了
```bash
fun -h    # 查看帮助
```



### 从源码安装

**环境要求：需要 Rust ≥1.80**
```bash
# 克隆仓库
git clone https://github.com/borenchan/fun-cli.git
cd fun-cli

# 编译并安装
cargo build --release
cargo install --path .  # 安装到全局

# 运行单元测试
cargo test -- --nocapture  # 禁止输出被吞

# 恭喜你！可以开始愉快的玩耍了
fun -h    # 查看帮助

```

## 🎯 **使用示例**

### 游戏功能

#### 五子棋（Gomoku）◉◎
```bash
# 默认难度（简单）
fun game -s 3

# 中等难度
fun game -s 3 -d 2

# 困难模式（Minimax 深度4）
fun game -s 3 -d 3

# 地狱模式（Minimax 深度6 + 高级棋型识别）
fun game -s 3 -d 4

# 自定义棋盘大小（9-19）
fun game -s 3 -w 19 -H 19 -d 3
```

**游戏特性：**
- 🎮 **4种AI难度**：从随机菜鸟到地狱魔王
- 🧠 **智能AI**：Minimax搜索 + Alpha-Beta剪枝 + 置换表优化
- 🎨 **棋型识别**：活四、冲四、活三、双三、四三等复杂棋型
- ⚡ **性能优化**：候选位置智能剪枝，地狱模式也能流畅运行
- 🎯 **完美防守**：AI绝不会漏掉任何威胁

**操作说明：**
- 方向键：移动光标
- Enter：落子
- Q：退出游戏
- R：游戏结束后重新开始

#### 俄罗斯方块（Tetris）🟦
```bash
fun game -s 2
```

#### 雷霆战机（Thunder Fighter）🕹️
```bash
fun game -s 1
```

### 其他功能

#### 天气查询 ☔
```bash
fun weather 北京
```

#### 音乐播放 🎵
```bash
fun music play /path/to/your/music.mp3
```

#### 系统监控 💻
```bash
fun osystem
```

## 🚀 **为什么加入？**
- 用最快乐的方式练习编程 **快乐是编程的根本动力**
- 无需严肃的代码审查，**好玩是第一生产力**
- 摆脱重复且无聊的工作循环，**享受编程最原始的快乐**

## 🤝 **如何贡献？**  
1. 提交一个 PR 并附上：功能说明
2. 写代码时默念三遍「 borrow checker 是朋友」


**我们不需要完美的代码，只需要有趣的灵魂！**  

```shell
0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
0000000OOkxxxddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddxxkkO0000000000
0000OkxoollloooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooolllodxO0000000
000kdlloodddddxddxxxxxddxxxxdxxxxxxdxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxdxxxxxxxxxxxxxxxxxxddddollodO00000
0OxlloddxxxxxxxxxxxxxxxxddxxxdxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxdxxxxxxxxxxxxxxxxdddoloxO000
Odllddxxxxxxxxxxxxxxxxxxxxdxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxddxxxxxxxxxxxxxxxxxdolox000
xllodxxxxxxxxxxxxdolc:::::::::codxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxdl:,''',:odxxxxxxxxxxxxxdolok00
dlodxdxxdxxxxxxxdc..............;oxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxddl,...','...;oxxxxxxxxxxxxxdllx00
oloxxxxxxxxxxxxxc. .;:::::::::'  :dxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxl' .cddxdo;. ,dxxxxxxxxxxxxdoldO0
oloxxxxxxxxxxxxd:. ,cccccccccc;. ,dxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxd:..:dxxxdxo' .lxxxxxxxxxxxxdoldO0
olodxxxxxxdddddd:. ,cccccccccc;. ,odddddxxxxxxxxxxxxxxxxxxxxxxxxxxxc. 'ldddxdc. 'oxxxxxxxxxxxxdoldO0
olodxxxdo:'...... .,cccccccccc;. ......';ldxxxxxxxxxxxxxxxxxxddddxxd:. .,;:;'..'ldxddddxxdxxxxdoldO0
olodxxxl' ..'''''';cccccccccccc;''''''.. .cdxxxxxxxxxxxxxdoc,'..',:odo:'.....,cooc;'..',:ldxxxdoldO0
olodxxd:  ,cccccccccccccccccccccccccccc;. ,dxxxxxxxxxxxxdc...',,'. .:oxddooodddc. .',,'...;oxxdoldO0
olodxxd; .;cccccccccccccccccccccccccccc:. ,dxxxxxxxxxxxdc. 'ldxddo, .:dxxxxxxdc. 'lddddo;. ;dxdoldO0
oloxxxd; .;cccccccccccccccccccccccccccc:. 'dxxxxxxxxxxxd; .cddddxdl. ,oxxxxdxd; .cddddddo. 'oxdoldO0
oloxxxd; .,cccccccccccccccccccccccccccc:. ,oxxxxxxxxxxxd:. ,oddddo;. ;dxxxxdddc. ,odxxdd:. ;dxdoldO0
olodxxd:. ':cccccccccccccccccccccccccc:,. ;dxxxxxxxxxxxxd:. .,;;,. .;oxdooloodd:. .,;;,...,odxdoldO0
olodxxxo;. .......':cccccccccc:'....... .,oxxxxxxxxxxxxxxdo:'....';ldl;'.....':oo:'....';ldxxxdoldO0
olodxxxxdl:;;;;;.  ,cccccccccc;. .,;;;;:ldxxxxxxxxxxxxxxxxxxddddddxd:. .;:c:,. .lddddoddxxxxxxdoldO0
olodxxxxxxxxxxxd:. ,cccccccccc;. ;dxxdxxxxxxxxxxxxxxxxxxxxxxxxxxxxxc. ,odddddc. 'oxxxxxxxxxxxxdoldO0
olodxdxxxxxxxxxd:. ,cccccccccc;. ;dxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx:. ;ddddddo' .lxxxxxxxxxxxxdoldO0
olodddxxxxxxxxxxl. .',,,,,,,,'. .cxxdxxdxxxxxxxxxxxxxxxxxxxxxxxxxxxl' .:odddl;. ;dxxxxxxxxxxxxdllx00
dlodxxxxxxxxxxxxdl,............,cdxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxdl,....'....:oxxxxxxxxxxxdxdllx00
xllodxxxxxxxxxxdxxddooooooooooddxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxdl:;,,,;codxxxxxxxxxxxxddoloO00
Oxllodxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxdxxxxxxxxxxxxxxxxdolok000
0Oxoloddxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxddxxxxxxxxxxxxdddollok0000
000kdollooddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddoolloxO00000
00000OkdoollllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllloodxkO0000000
00000000OOkkxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxkkOO00000000000
0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```

