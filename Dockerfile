# 多阶段构建 Dockerfile for GPT-SoVITS Rust 应用程序

# 第一阶段：构建阶段
FROM swr.cn-north-4.myhuaweicloud.com/ddn-k8s/docker.io/pytorch/pytorch:2.5.0-cuda12.4-cudnn9-devel

# 设置时区
ENV TZ=Asia/Shanghai
ENV TimeZone=Asia/Shanghai

# 设置工作目录
WORKDIR /app

# 安装 Rust 和必要的系统依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# 设置 Rust 镜像环境变量
ENV RUSTUP_DIST_SERVER=https://rsproxy.cn
ENV RUSTUP_UPDATE_ROOT=https://rsproxy.cn/rustup

# 安装 Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# 配置 crates.io 镜像源
RUN mkdir -p /root/.cargo && \
    echo '[build]' > /root/.cargo/config.toml && \
    echo 'rustflags = ["-C", "link-args=-Wl,--no-as-needed -ldl -lpthread"]' >> /root/.cargo/config.toml && \
    echo '[source.crates-io]' >> /root/.cargo/config.toml && \
    echo 'replace-with = "rsproxy-sparse"' >> /root/.cargo/config.toml && \
    echo '[source.rsproxy-sparse]' >> /root/.cargo/config.toml && \
    echo 'registry = "sparse+https://rsproxy.cn/index/"' >> /root/.cargo/config.toml && \
    echo '[net]' >> /root/.cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> /root/.cargo/config.toml


# 复制 LibTorch 到镜像中
COPY libtorch/ /libtorch/

# 设置 LibTorch 环境变量
ENV LIBTORCH=/libtorch
ENV PATH=/usr/local/cuda-12.4/bin:$PATH
ENV LD_LIBRARY_PATH=/usr/local/cuda-12.4/lib64:/libtorch/lib:$LD_LIBRARY_PATH

# 复制 Cargo 文件
COPY Cargo.toml ./
COPY Cargo.lock ./

# 复制源代码和资源文件
COPY src/ ./src/
COPY build.rs ./
COPY resource/ ./resource/

# 构建应用程序
RUN cargo build --release

ENV RUST_LOG=info

# 复制配置文件
COPY config.toml /app/
COPY entrypoint.sh /app/

# 创建必要的目录
RUN mkdir -p /app/logs /app/voices /app/tmp

# 设置执行权限
RUN chmod +x /app/entrypoint.sh /app/target/release/gpt_sovits_rs

# 暴露端口
EXPOSE 6006

# # 设置入口点
# ENTRYPOINT ["/app/entrypoint.sh"]

# 默认命令
CMD ["/app/target/release/gpt_sovits_rs"]