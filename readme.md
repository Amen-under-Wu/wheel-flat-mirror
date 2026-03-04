# WheelFlat - 轮扁
*輮以为轮，其曲中规。*

*闭门造车，出门合辙。*

## 项目简述
本项目希望“重复造轮子”，基于web平台实现梦幻主机TIC-80的大部分功能，并在此基础上实现两点突破：
- 实现对中文字体的支持；
- RIIR。

## 构建方法
### 编译Rust代码
运行 `cargo install wasm-pack` 安装wasm-pack等工具，在根目录下运行
```
wasm-pack build --target web
```
如中途卡住，可能是由于相关工具无法自动下载，可尝试手动安装 `wasm-bindgen-cli`, `wasm-opt`等工具。

### 部署网页
在根目录下运行
```
http-server
```
之后访问 http://127.0.0.1:8080 即可。你也可以使用自己喜欢的其他方式部署网页。

## 设计结构
作为“梦幻主机”，该程序应具有与真实主机相似的行为，因此其基本结构也可以参考计算机组成原理的分层结构。具体而言，可粗略分为四层：

1. “硬件”层，实现虚拟的内存布局、基础的“处理器”指令，整合输入输出设备web API；
2. 系统层，实现应用程序的各种接口（参考TIC-80 wiki）和shell；
3. 系统软件层，实现lua（或其他脚本语言）解释器；
4. 应用层，用脚本语言实现代码编辑器、精灵编辑器、地图编辑器、音效编辑器等。

终审时应至少完成至第三层。

当前的程序架构大致为如下：

- **底层**：`WheelContext`提供绘图、播放声音（待实现）、捕获输入、读写文件等接口，供`dyn WheelProgram`调用。
- **中层**：`CartContext`提供内存布局及相关的各种功能接口，`SystemContext`提供终端输出、获取时间、退出程序等接口；`WheelWrapper`持有二者的共享所有权，将其接口暴露给`dyn InternalProgram`使用，并实现`trait WheelProgram`。
- **上层**：JsScript实现`trait InternalProgram`，整合Wrapper中的接口到js环境，供内部存储的js脚本调用。

## TODO

- [ ] 写文档😫

- [ ] 用Rust调用web api
  - [x] 图像api
  - [ ] 音频api
  - [x] 键盘api
  - [x] 鼠标api
- [x] 封装底层接口，实现系统内核
- [ ] 提供[“系统”api](https://github.com/nesbox/TIC-80/wiki/API)
  - [x] 回调函数
  - [ ] 绘图函数
    - [x] ...
    - [ ] font
  - [x] 输入函数
  - [ ] 音频函数
    - [ ] sfx
    - [ ] music
  - [ ] 内存操作函数
    - [x] ...
    - [x] sync
  - [x] 实用函数
  - [ ] 系统函数
    - [x] ...
    - [ ] reset
    - [x] trace (with color)
- [ ] 实现系统console
  - [x] 滚动功能
  - [ ] 选中功能
  - [ ] 复制粘贴功能
- [ ] 实现系统菜单
- [ ] 实现中文输入法
