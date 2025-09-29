#!/bin/bash

# GPT-SoVITS Rust Docker 构建和运行脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_message() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查 Docker 是否安装
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker 未安装，请先安装 Docker"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        print_warning "docker-compose 未安装，将使用 docker compose"
    fi
}

# 构建镜像
build_image() {
    print_message "开始构建 GPT-SoVITS Rust Docker 镜像..."
    docker build -t gpt-sovits-rs:latest .
    print_message "镜像构建完成"
}

# 使用 Podman 运行容器（基于您提供的命令）
run_with_podman() {
    print_message "使用 Podman 运行容器..."
    
    # 停止并删除现有容器（如果存在）
    if podman ps -a --format "{{.Names}}" | grep -q "^gpt_sovits_rs$"; then
        print_warning "停止并删除现有容器..."
        podman stop gpt_sovits_rs || true
        podman rm gpt_sovits_rs || true
    fi
    
    # 创建必要的目录
    mkdir -p ./voices ./logs ./tmp
    
    # 运行容器
    podman run -itd \
        --restart=always \
        -p 8080:6006 \
        -e TimeZone='Asia/Shanghai' \
        -e TZ='Asia/Shanghai' \
        -v /etc/localtime:/etc/localtime:ro \
        -v ./voices:/app/voices \
        -v ./logs:/app/logs \
        -v ./tmp:/app/tmp \
        --name gpt_sovits_rs \
        gpt-sovits-rs:latest
    
    print_message "容器已启动，访问地址: http://localhost:8080"
}

# 使用 Docker 运行容器
run_with_docker() {
    print_message "使用 Docker 运行容器..."
    
    # 停止并删除现有容器（如果存在）
    if docker ps -a --format "{{.Names}}" | grep -q "^gpt_sovits_rs$"; then
        print_warning "停止并删除现有容器..."
        docker stop gpt_sovits_rs || true
        docker rm gpt_sovits_rs || true
    fi
    
    # 创建必要的目录
    mkdir -p ./voices ./logs ./tmp
    
    # 运行容器
    docker run -itd \
        --restart=always \
        -p 8080:6006 \
        -e TimeZone='Asia/Shanghai' \
        -e TZ='Asia/Shanghai' \
        -v /etc/localtime:/etc/localtime:ro \
        -v ./voices:/app/voices \
        -v ./logs:/app/logs \
        -v ./tmp:/app/tmp \
        --name gpt_sovits_rs \
        gpt-sovits-rs:latest
    
    print_message "容器已启动，访问地址: http://localhost:8080"
}

# 使用 Docker Compose 运行
run_with_compose() {
    print_message "使用 Docker Compose 运行..."
    
    # 创建必要的目录
    mkdir -p ./voices ./logs ./tmp
    
    if command -v docker-compose &> /dev/null; then
        docker-compose up -d
    else
        docker compose up -d
    fi
    
    print_message "服务已启动，访问地址: http://localhost:8080"
}

# 显示日志
show_logs() {
    print_message "显示容器日志..."
    if command -v podman &> /dev/null && podman ps --format "{{.Names}}" | grep -q "^gpt_sovits_rs$"; then
        podman logs -f gpt_sovits_rs
    elif docker ps --format "{{.Names}}" | grep -q "^gpt_sovits_rs$"; then
        docker logs -f gpt_sovits_rs
    else
        print_error "未找到运行中的容器"
    fi
}

# 停止容器
stop_container() {
    print_message "停止容器..."
    if command -v podman &> /dev/null && podman ps --format "{{.Names}}" | grep -q "^gpt_sovits_rs$"; then
        podman stop gpt_sovits_rs
    elif docker ps --format "{{.Names}}" | grep -q "^gpt_sovits_rs$"; then
        docker stop gpt_sovits_rs
    else
        print_warning "未找到运行中的容器"
    fi
}

# 显示帮助信息
show_help() {
    echo "GPT-SoVITS Rust Docker 管理脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  build           构建 Docker 镜像"
    echo "  run-podman      使用 Podman 运行容器"
    echo "  run-docker      使用 Docker 运行容器"
    echo "  run-compose     使用 Docker Compose 运行"
    echo "  logs            显示容器日志"
    echo "  stop            停止容器"
    echo "  help            显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 build && $0 run-podman"
    echo "  $0 run-compose"
    echo "  $0 logs"
}

# 主函数
main() {
    case "${1:-help}" in
        build)
            check_docker
            build_image
            ;;
        run-podman)
            build_image
            run_with_podman
            ;;
        run-docker)
            check_docker
            build_image
            run_with_docker
            ;;
        run-compose)
            check_docker
            build_image
            run_with_compose
            ;;
        logs)
            show_logs
            ;;
        stop)
            stop_container
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "未知选项: $1"
            show_help
            exit 1
            ;;
    esac
}

# 执行主函数
main "$@"