# 说明文档 - WheelFlat轮扁

*輮以为轮，其曲中规。*

*闭门造车，出门合辙。*

## 项目简述
本项目希望“重复造轮子”，基于web平台实现梦幻主机TIC-80的大部分功能，并在此基础上实现两点突破：
- 实现对中文字体的支持；
- RIIR。

## 项目结构

当前的程序架构大致为如下：

- **底层**：`WheelContext`提供绘图、播放声音（待实现）、捕获输入、读写文件等接口，供`dyn WheelProgram`调用。
- **中层**：`CartContext`提供内存布局及相关的各种功能接口，`SystemContext`提供终端输出、获取时间、退出程序等接口；`WheelWrapper`持有二者的共享所有权，将其接口暴露给`dyn InternalProgram`使用，并实现`trait WheelProgram`。
- **上层**：JsScript实现`trait InternalProgram`，整合Wrapper中的接口到js环境，供内部存储的js脚本调用。

## 使用说明
目前内置编辑器尚未实现，因此用户只能通过上传外部编辑器生成的二进制文件运行自制程序。二进制文件的编码格式大部分与`.tic`文件格式相同。由于web平台对许多rust库支持有限，WheelFlat目前仅支持使用javascript编写脚本。

### 接口

脚本中可用的api参考了TIC-80的格式，下文将列举目前已实现的接口。

#### 回调函数
- `init()`：在程序启动时执行一次，相当于TIC-80的`BOOT`。
- `update()`：每帧执行一次，相当于TIC-80的`TIC`。
- `scanline(i)`：每帧绘制屏幕上的第`i`行像素时执行，相当于TIC-80的`BDR`。
- `overlay()`：每帧绘制`Vram`中的第二个bank前执行，相当于TIC-80的`OVR`。

#### 绘图函数
- `circ(x, y, r, color)`：以`(x,y)`为圆心绘制半径为`r`像素且颜色为`color`的实心圆形。
- `circb(x, y, r, color)`：以`(x,y)`为圆心绘制半径为`r`像素且颜色为`color`的空心圆形。
- `elli(x, y, a, b, color)`：以`(x,y)`为中心，绘制x方向半轴长为`a`像素，y方向半轴长为`b`像素，颜色为`color`的实心圆形。
- `ellib(x, y, a, b, color)`：以`(x,y)`为中心，绘制x方向半轴长为`a`像素，y方向半轴长为`b`像素，颜色为`color`的空心圆形。
- `clip(x, y, w, h)`：将屏幕的可绘制区域限制为左上顶点坐标为`(x,y)`，宽为`w`，高为`h`的矩形区域。
- `clip()`：重置屏幕的可绘制区域。
- `cls(color=0)`：将屏幕用`color`颜色的像素填充。
- `line(x1, y1, x2, y2, color)`：绘制一条连接`(x1,y1)`和`(x2,y2)`两点且颜色为`color`的直线。
- `map(x=0, y=0, w=30, h=17, sx=0, sy=0, trans_color=-1, scale=1 remap)`：在屏幕上以`(x,y)`为左上顶点绘制地图，绘制的地图块为大地图的一个矩形区域，矩形左上顶点为`(sx,sy)`对应的地图块，横向宽`w`个地图块、纵向高`h`个地图块，缩放比例为`scale`，绘制时颜色为`trans_color`的像素保持透明；`remap`为可选的回调函数，接受被绘制的地图块的`id`与在地图上的坐标为传入参数，输出新的地图块`id`和`flip` `rotate`参数，以调整该地图块实际的绘制方式。
- `pix(x, y, color)`：将像素`(x,y)`的颜色设置为`color`。
- `pix(x, y) -> color`：获取像素`(x,y)`的颜色。
- `print(text, x=0, y=0, color=12, fixed=false, scale=1, alt_font=false) -> text_width`：在屏幕上以`color`颜色绘制字符串`text`的文字，绘制出文字的左上坐标为`(x,y)`，缩放比例为`scale`，绘制出文字的总宽度为`text_width`；`fixed`控制文字是否定宽，`alt_font`控制绘制所使用的文字字体。
- `print_ch(text, x=0, y=0, color=12, fixed=false, scale=1, alt_font=false) -> text_width`：与`print`类似，但支持以寒蝉点阵体绘制`text`中的中文字符（暂不支持中文标点）；`alt_font`为`true`时使用寒蝉点阵体的7px字体，否则使用16px字体。
- `rect(x, y, w, h, color)`：绘制左上顶点为`(x,y)`，宽`w`像素，高`h`像素，颜色为`color`的实心矩形。
- `rectb(x, y, w, h, color)`：绘制左上顶点为`(x,y)`，宽`w`像素，高`h`像素，颜色为`color`的空心矩形。
- `spr(id, x, y, trans_color=-1, scale=1, flip=0, rotate=0, w=1, h=1)`：绘制矩形精灵图，左上顶点为`(x,y)`，缩放比例为`scale`，被绘制的精灵在画布上左上角精灵单元id为`id`，宽`w`个精灵单元，高`h`个精灵单元；颜色为`trans_color`的像素绘制为透明；`flip`与`rotate`控制精灵的旋转与对称，旋转优先于对称，具体参见TIC-80文档。
- `tri(x1, y1, x2, y2, x3, y3, color)`：绘制顶点坐标为`(x1,y1)` `(x2,y2)` `(x3,y3)`，颜色为`color`的实心三角形。
- `trib(x1, y1, x2, y2, x3, y3, color)`：绘制顶点坐标为`(x1,y1)` `(x2,y2)` `(x3,y3)`，颜色为`color`的空心三角形。
- `textri(x1, y1, x2, y2, x3, y3, u1, v1, u2, v2, u3, v3, use_map=false, trans_color=-1)`：将纹理上顶点坐标为`(u1,v1)` `(u2,v2)` `(u3,v3)`的三角形区域映射到屏幕上顶点坐标为`(x1,y1)` `(x2,y2)` `(x3,y3)`的三角形区域，`use_map`为`true`时纹理图为地图，否则为精灵画布；颜色为`trans_color`的像素绘制为透明。

#### 输入函数
- `btn(id) -> bool`：在`id`对应的键已被按下时返回真。
- `btnp(id) -> bool`：在`id`对应的键被按下且该键在前一帧中未被按下时返回真。
- `btnp(id, hold, period=1) -> bool`：在`id`对应的键被按下时长为0帧或时长减去`hold`值不小于0且能被`period`整除时返回真。
- `key() -> bool`：在键盘有输入时返回真。
- `key(code) -> bool`：在键盘上`code`对应的键有输入时返回真。
- `keyp() -> bool`：在键盘有某键被按下且该键在前一帧中未被按下时返回真。
- `keyp(code) -> bool`：在键盘上`code`对应的键被按下且该键在前一帧中未被按下时返回真。
- `keyp(code, hold, period=1) -> bool`：在`code`对应的键被按下时长为0帧或时长减去`hold`值不小于0且能被`period`整除时返回真。
- `mouse() -> [x, y, left, middle, right, scroll_x, scroll_y]`：返回鼠标指针的坐标、各按键状态和滚轮状态。

#### 内存操作函数
- `memcpy(to, from, len)`：将`Ram`中首地址为`from`，长度为`len`的一段数据复制到首地址为`to`的区域中。
- `memset(addr, val, len)`：将`Ram`中首地址为`addr`，长度为`len`的区域内数值均设为`val`。
- `peek(addr) -> val`：查看`Ram`中地址为`addr`的数值。
- `peek(addr, bits) -> val`：将`Ram`以小端序展开为bitmap，每`bits`位截断，查看第`addr`个bits对应的数值；`bits`只能取8的因数。
- `peek1(addr1) -> val1`：相当于`peek(addr1, 1)`。
- `peek2(addr2) -> val2`：相当于`peek(addr2, 2)`。
- `peek4(addr4) -> val4`：相当于`peek(addr4, 4)`。
- `poke(addr, val)`：将`Ram`中地址为`addr`的数值设为`val`。
- `poke(addr, val, bits)`：将`Ram`以小端序展开为bitmap，每`bits`位截断，将第`addr`个bits对应的数值设置为`val`；`bits`只能取8的因数。
- `poke1(addr1, val1)`：相当于`poke(addr1, val1, 1)`。
- `poke2(addr2, val2)`：相当于`poke(addr2, val2, 2)`。
- `poke4(addr4, val4)`：相当于`poke(addr4, val4, 4)`。
- `sync(mask, bank, to_cart=false)`：将`Ram`中的信息存储进卡带，或将卡带中的信息读取到`Ram`。
- `vbank(bank)`：将当前`Vram`使用的bank切换为编号`bank`的bank。

#### 实用函数
- `fget(id, flag) -> val`：获取`id`对应`sprite`的第`flag`个flag值。
- `fset(id, flag, val)`：设置`id`对应`sprite`的第`flag`个flag值。
- `mget(x, y) -> id`：获取大地图上坐标为`(x,y)`的地图块的`id`。
- `mset(x, y) -> id`：设置大地图上坐标为`(x,y)`的地图块的`id`。

#### 系统函数
- `exit()`：退出到命令行界面。
- `time()`：获取程序启动以来的毫秒数。
- `tstamp()`：获取当前的秒级UNIX时间戳。
- `trace(text, color=15)`：向命令行的“标准输出”写入字符串`text`，以颜色`color`显示。
