#!/bin/bash

# 设置环境变量
export LIBTORCH=/libtorch
export LD_LIBRARY_PATH=$LIBTORCH/lib:$LD_LIBRARY_PATH
export RUST_LOG=info

# 应用程序路径
APP_PATH="/app"
APP_NAME="gpt_sovits_rs"
LOG_FILE="/app/logs/gpt_sovits_rs.log"

# 确保日志目录存在
mkdir -p /app/logs

# 如果命令是 start，则启动应用
if [ "$1" = "start" ]; then
    cd "$APP_PATH" && exec ./$APP_NAME
# 如果命令是 shell，则提供一个 shell
elif [ "$1" = "shell" ]; then
    exec /bin/bash
# 否则，执行传入的命令
else
    exec "$@"
fi 