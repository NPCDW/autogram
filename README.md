# Autogram

# 功能

使用任何功能都需要先去 [telegram api](https://my.telegram.org/apps) 网站申请一个客户端，需要用家宽申请，使用代理会报错 ERROR。
然后在环境变量中配置 `API_ID` 和 `API_HASH`

当前支持
- akile 自动签到，需要配置 `AKILE_CHAT_ID` 如果不知道此ID，可以先运行程序，会打印前二十个聊天的ID，找到之后配置

# 开发

编译安装 `tdlib` 依赖
```bash
sudo apt update && sudo apt upgrade
sudo apt-get install -y gcc pkg-config cmake g++ gperf libssl-dev zlib1g-dev

sudo cd /usr/src/
sudo git clone https://github.com/tdlib/td.git
cd td/
git checkout 2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09         # tdlib = "0.10.0" 对应此版本，如果之前编译过其他版本，需要删除本项目目录下的 db 数据缓存
sudo mkdir build && cd build
sudo cmake -DCMAKE_BUILD_TYPE=Release ..
sudo cmake --build .
sudo cp pkgconfig/* /usr/lib/pkgconfig/
sudo cp libtdjson.so* /usr/local/lib/
sudo ldconfig
```

添加环境变量，编辑 `/etc/profile` 文件
```
export API_ID=12345678
export API_HASH=1234567890abcdef1234567890abcdef
```
使其生效
```bash
source /etc/profile
```