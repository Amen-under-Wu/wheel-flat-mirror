# WheelFlat - 轮扁
*輮以为轮，其曲中规。*

*闭门造车，出门合辙。*

## 项目简述
本项目希望“重复造轮子”，基于web平台实现梦幻主机TIC-80的大部分功能，并在此基础上实现两点突破：
- 实现对中文字体的支持；
- RIIR。

详细文档见`doc`文件夹下的`doc.pdf`。

## 构建方法
当前项目已配置了github actions，最新版本将自动部署在[此](https://amen-under-wu.github.io/wheel-flat-mirror/)。如希望手动部署，可参考下列步骤。

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

## TODO

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
  - [x] 内存操作函数
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
