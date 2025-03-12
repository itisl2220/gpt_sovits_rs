#!/bin/bash

PID_FILE="/var/run/gpt_sovits_rs.pid"
APP_NAME="gpt_sovits_rs"
LOG_FILE="/var/log/gpt_sovits_rs.log"

# 检查 PID 文件是否存在
if [ ! -f "$PID_FILE" ]; then
    echo "$APP_NAME 未运行"
    exit 1
fi

# 获取 PID 并检查进程
PID=$(cat "$PID_FILE")
if ps -p $PID > /dev/null; then
    echo "$APP_NAME 正在运行 (PID: $PID)"
    echo "最近的日志:"
    tail -n 10 "$LOG_FILE"
    exit 0
else
    echo "$APP_NAME 已崩溃或被意外终止"
    echo "最近的错误日志:"
    tail -n 20 "$LOG_FILE"
    echo "删除过期的 PID 文件"
    rm -f "$PID_FILE"
    exit 1
fi 