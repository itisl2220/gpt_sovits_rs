#!/bin/bash

# 配置文件路径
CONFIG_FILE="/root/autodl-tmp/config.toml"

# 默认配置
APP_PATH="/root/autodl-tmp/gpt_sovits_rs"
LOG_FILE="/root/autodl-tmp/gpt_sovits_rs.log"
CACHE_DIR="/root/autodl-tmp/tmp"
RESTART_INTERVAL=3600  # 默认1小时重启一次
PORT=6006
LOG_LEVEL="info"
LIBTORCH_PATH="/root/libtorch"

# 如果存在配置文件，则读取配置
if [ -f "$CONFIG_FILE" ]; then
    echo "正在加载配置文件: $CONFIG_FILE"
    
    # 使用 Python 解析 TOML 文件
    # 这需要安装 toml 包: pip install toml
    CONFIG=$(python3 -c "
import toml
import json
import sys
try:
    with open('$CONFIG_FILE', 'r') as f:
        config = toml.load(f)
        print(json.dumps(config))
except Exception as e:
    print('{\"error\": \"' + str(e) + '\"}', file=sys.stderr)
    exit(1)
")
    
    # 检查是否有错误
    if [ $? -ne 0 ]; then
        echo "警告：解析配置文件失败，将使用默认配置"
    else
        # 解析 JSON 并设置变量
        APP_PATH=$(echo $CONFIG | python3 -c "import sys, json; print(json.load(sys.stdin).get('app_path', '$APP_PATH'))")
        LOG_FILE=$(echo $CONFIG | python3 -c "import sys, json; print(json.load(sys.stdin).get('log_file', '$LOG_FILE'))")
        CACHE_DIR=$(echo $CONFIG | python3 -c "import sys, json; print(json.load(sys.stdin).get('cache_dir', '$CACHE_DIR'))")
        RESTART_INTERVAL=$(echo $CONFIG | python3 -c "import sys, json; print(json.load(sys.stdin).get('restart_interval', $RESTART_INTERVAL))")
        PORT=$(echo $CONFIG | python3 -c "import sys, json; print(json.load(sys.stdin).get('port', $PORT))")
        LOG_LEVEL=$(echo $CONFIG | python3 -c "import sys, json; print(json.load(sys.stdin).get('log_level', '$LOG_LEVEL'))")
        LIBTORCH_PATH=$(echo $CONFIG | python3 -c "import sys, json; print(json.load(sys.stdin).get('libtorch_path', '$LIBTORCH_PATH'))")
    fi
else
    echo "警告：找不到配置文件 $CONFIG_FILE，将使用默认配置"
fi

# 设置环境变量
export LIBTORCH=$LIBTORCH_PATH
export LD_LIBRARY_PATH=$LIBTORCH/lib:$LD_LIBRARY_PATH
# 绕过 PyTorch 版本检查
export LIBTORCH_BYPASS_VERSION_CHECK=1
# 设置缓存目录环境变量，供应用程序使用
export GPT_SOVITS_CACHE_DIR="$CACHE_DIR"
# 设置日志级别
export RUST_LOG=$LOG_LEVEL

# 应用程序名称
APP_NAME="gpt_sovits_rs"
PID_FILE="/tmp/gpt_sovits_rs.pid"

# 确保所有必要的目录都存在
mkdir -p $(dirname "$LOG_FILE")
mkdir -p "$APP_PATH"
mkdir -p "$CACHE_DIR"

# 停止应用的函数
stop_app() {
    for pid in $(pgrep -f "target/release/$APP_NAME $PORT"); do
        if [ -n "$pid" ]; then
            echo "正在停止 $APP_NAME (PID: $pid)..."
            kill $pid
            # 等待进程结束，最多等待10秒
            for i in {1..10}; do
                if ! ps -p $pid > /dev/null 2>&1; then
                    break
                fi
                sleep 1
            done
            # 如果进程仍然存在，强制终止
            if ps -p $pid > /dev/null 2>&1; then
                echo "强制终止 $APP_NAME (PID: $pid)..."
                kill -9 $pid
            fi
            echo "$APP_NAME 已停止 (PID: $pid)"
        fi
    done
    
    # 删除PID文件
    if [ -f "$PID_FILE" ]; then
        rm -f "$PID_FILE"
    fi
}

# 启动应用的函数
start_app() {
    # 检查应用程序文件是否存在
    if [ ! -f "$APP_PATH/target/release/$APP_NAME" ]; then
        echo "错误：找不到应用程序文件 $APP_PATH/target/release/$APP_NAME"
        return 1
    fi

    # 确保应用没有在运行
    stop_app
    
    # 启动应用
    echo "正在启动 $APP_NAME 在端口 $PORT..."
    cd "$APP_PATH" && nohup ./target/release/$APP_NAME $PORT > "$LOG_FILE" 2>&1 &
    
    # 保存 PID
    local pid=$!
    echo $pid > "$PID_FILE"
    echo "$APP_NAME 已启动，PID: $pid"
    echo "日志文件: $LOG_FILE"
    
    # 等待几秒确认服务已启动
    sleep 3
    if ! ps -p $pid > /dev/null 2>&1; then
        echo "警告: 服务可能未成功启动，请检查日志文件"
        return 1
    fi
    
    return 0
}

# 处理信号
trap 'stop_app; exit 0' SIGINT SIGTERM

# 首次启动应用
start_app
if [ $? -ne 0 ]; then
    echo "应用启动失败，请检查路径和权限"
    exit 1
fi

# 定时重启循环
echo "设置每 $RESTART_INTERVAL 秒自动重启一次服务"
while true; do
    # 等待指定的时间
    sleep $RESTART_INTERVAL
    
    echo "执行定时重启..."
    start_app
    
    if [ $? -ne 0 ]; then
        echo "重启失败，等待60秒后重试"
        sleep 60
    fi
done 