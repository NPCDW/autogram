准备安装依赖
```bash
apt update && apt upgrade
apt-get install -y gcc
apt install -y pkg-config
apt install -y cmake
apt-get install -y g++
apt-get install -y gperf
apt-get install -y libssl-dev
apt install -y zlib1g-dev

cd /usr/src/
git clone https://github.com/tdlib/td.git
cd td/
mkdir build
cd build
cmake -DCMAKE_BUILD_TYPE=Release ..
cmake --build .
cp pkgconfig/* /usr/lib/pkgconfig/
cp libtdjson.so* /usr/local/lib/
ldconfig
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