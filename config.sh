#!/bin/bash

# GPT-SoVITS 配置文件

# 应用程序路径
APP_PATH="/root/autodl-tmp/gpt_sovits_rs"

# 日志文件路径
LOG_FILE="/root/autodl-tmp/gpt_sovits_rs.log"

# 缓存目录
CACHE_DIR="/root/autodl-tmp/tmp"

# 自动重启间隔（秒）
# 3600 = 1小时
RESTART_INTERVAL=3600

# 服务端口
PORT=6006

# 日志级别 (trace, debug, info, warn, error)
LOG_LEVEL=info 