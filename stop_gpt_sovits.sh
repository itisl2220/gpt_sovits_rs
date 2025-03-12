#!/bin/bash

PID_FILE="/var/run/gpt_sovits_rs.pid"
APP_NAME="gpt_sovits_rs"

# 检查 PID 文件是否存在
if [ ! -f "$PID_FILE" ]; then
    echo "$APP_NAME 未运行"
    exit 0
fi

# 获取 PID 并终止进程
PID=$(cat "$PID_FILE")
if ps -p $PID > /dev/null; then
    echo "正在停止 $APP_NAME (PID: $PID)..."
    kill $PID
    
    # 等待进程终止
    for i in {1..10}; do
        if ! ps -p $PID > /dev/null; then
            break
        fi
        echo "等待进程终止..."
        sleep 1
    done
    
    # 如果进程仍在运行，强制终止
    if ps -p $PID > /dev/null; then
        echo "强制终止进程..."
        kill -9 $PID
    fi
    
    echo "$APP_NAME 已停止"
else
    echo "进程不存在，可能已经停止"
fi

# 删除 PID 文件
rm -f "$PID_FILE" 