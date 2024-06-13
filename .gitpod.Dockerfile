FROM gitpod/workspace-full

WORKDIR /usr/src/

RUN sudo apt-get update -y
RUN sudo apt-get install -y gcc pkg-config cmake g++ gperf libssl-dev zlib1g-dev git openssl ca-certificates
RUN sudo git clone https://github.com/tdlib/td.git
WORKDIR /usr/src/td/
RUN sudo git checkout 2589c3fd46925f5d57e4ec79233cd1bd0f5d0c09
RUN sudo mkdir build
WORKDIR /usr/src/td/build/
RUN sudo cmake -DCMAKE_BUILD_TYPE=Release ..
RUN sudo cmake --build .
RUN sudo cp -r /usr/src/td/build/pkgconfig/* /usr/lib/pkgconfig/
RUN sudo cp -r /usr/src/td/build/libtdjson.so* /usr/local/lib/
RUN sudo ldconfig
