# Autogram

# 开发

编译安装 `tdlib` 依赖
```bash
sudo apt update && sudo apt upgrade
sudo apt-get install -y gcc pkg-config cmake g++ gperf libssl-dev zlib1g-dev

sudo cd /usr/src/
sudo git clone https://github.com/tdlib/td.git
cd td/
git checkout 2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09         # tdlib = "0.10.0" 对应此版本
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