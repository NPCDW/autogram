# Autogram

> 特别提醒
> 能用机器人做到的事就不要用此程序，此程序相当于真人操作

# 功能

当前支持
- akile 自动签到

# 使用

使用任何功能都需要先去 [telegram api](https://my.telegram.org/apps) 网站申请一个客户端，需要用家宽申请，使用代理会报错 ERROR。

任意位置新建 `autogram` 文件夹，将 [docker-compose.yml](./docker-compose.yml) 文件复制到此文件夹，并修改环境变量字段，必须配置 `API_ID` 和 `API_HASH` ，其他环境变量可以在登录后选择配置，执行命令
```bash
docker compose pull
docker compose run --rm -it autogram login            # 登录你的账户，API_ID 相当于你申请的网站，login 相当于在你的网站上登录你的账户，需要输入手机号和验证码登录，使用其他命令前必须先登录
docker compose run --rm -it autogram chats            # 查看前几个聊天组的ID和标题，用于配置自动化，默认前20，可以使用 --top 50 参数指定
docker compose run --rm -it autogram chat             # 指定一个聊天ID和消息内容，发送消息，示例： docker compose run --rm -it autogram chat --chat-id='-1234567890123' -m '/checkin'
docker compose run --rm -it autogram chat             # 监听一个聊天，示例： docker compose run --rm -it autogram listen --chat-id='-1234567890123'
docker compose run --rm -it autogram start            # 默认命令，使用 docker compose up 启动时会执行此命令
```

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

添加环境变量，编辑 `~/.bashrc` 文件
```
export API_ID=12345678
export API_HASH=1234567890abcdef1234567890abcdef
export AKILE_CHAT_ID=-1234567890123         # 选填
```
把终端关闭，重新打开即可使其生效