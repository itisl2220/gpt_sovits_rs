#!/bin/bash

# 设置环境变量
export LIBTORCH=/root/libtorch
export LD_LIBRARY_PATH=$LIBTORCH/lib:$LD_LIBRARY_PATH
# 绕过 PyTorch 版本检查
export LIBTORCH_BYPASS_VERSION_CHECK=1

# 应用程序路径
APP_PATH="./gpt_sovits_rs"
APP_NAME="gpt_sovits_rs"
# 修改日志文件路径到 ~/autodl-tmp/
LOG_FILE="$HOME/autodl-tmp/gpt_sovits_rs.log"
PID_FILE="/var/run/gpt_sovits_rs.pid"

# 获取端口参数，默认为6006
PORT=${1:-6006}

# 确保日志目录存在
mkdir -p $(dirname "$LOG_FILE")

# 自动重启间隔（秒）
RESTART_INTERVAL=3600  # 1小时 = 3600秒

# 启动应用的函数
start_app() {
    # 检查应用是否已经在运行
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p $PID > /dev/null; then
            echo "$APP_NAME 已经在运行，PID: $PID"
            return 1
        else
            echo "移除过期的 PID 文件"
            rm "$PID_FILE"
        fi
    fi

    # 启动应用
    echo "正在启动 $APP_NAME 在端口 $PORT..."
    cd "$APP_PATH" && nohup ./target/release/$APP_NAME $PORT > "$LOG_FILE" 2>&1 &

    # 保存 PID
    echo $! > "$PID_FILE"
    echo "$APP_NAME 已启动，PID: $!"
    echo "日志文件: $LOG_FILE"
    
    return 0
}

# 停止应用的函数
stop_app() {
    if [ -f "$PID_FILE" ]; then
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
        fi
        rm -f "$PID_FILE"
    fi
}

# 首次启动应用
start_app

# 在后台运行定时重启任务
(
    while true; do
        sleep $RESTART_INTERVAL
        echo "执行定时重启..."
        stop_app
        sleep 5  # 等待5秒确保完全停止
        start_app
    done
) &

# 保存后台任务的PID，以便在需要时可以终止
echo $! > "/var/run/gpt_sovits_rs_restart.pid"

# 在容器中，保持前台进程运行
tail -f "$LOG_FILE" 