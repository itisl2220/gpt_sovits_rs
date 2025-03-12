#!/bin/bash

# 设置环境变量
export LIBTORCH=/path/to/libtorch
export LD_LIBRARY_PATH=$LIBTORCH/lib:$LD_LIBRARY_PATH
export RUST_LOG=info

# 应用程序路径
APP_PATH="/path/to/gpt_sovits_rs"
APP_NAME="gpt_sovits_rs"
LOG_FILE="/var/log/gpt_sovits_rs.log"
PID_FILE="/var/run/gpt_sovits_rs.pid"

# 确保日志目录存在
mkdir -p $(dirname "$LOG_FILE")

# 检查应用是否已经在运行
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p $PID > /dev/null; then
        echo "$APP_NAME 已经在运行，PID: $PID"
        exit 1
    else
        echo "移除过期的 PID 文件"
        rm "$PID_FILE"
    fi
fi

# 启动应用
echo "正在启动 $APP_NAME..."
cd "$APP_PATH" && nohup ./target/release/$APP_NAME > "$LOG_FILE" 2>&1 &

# 保存 PID
echo $! > "$PID_FILE"
echo "$APP_NAME 已启动，PID: $!"
echo "日志文件: $LOG_FILE"

# 在容器中，保持前台进程运行
tail -f "$LOG_FILE" 