#!/bin/bash
set -euo pipefail

# decrypt-env.sh — 被 mise _.source 调用
# 自动解密当前目录 .env / .env.local 中的 enc: 前缀敏感值
#
# 依赖环境变量:
#   ENCRYPTION_KEY_PATH — 加密密钥文件路径（由根目录 .mise.toml 提供）

ENV_FILE=""
if [[ -f ".env.local" ]]; then
    ENV_FILE=".env.local"
elif [[ -f ".env" ]]; then
    ENV_FILE=".env"
else
    return 0
fi

# 向上查找项目根目录（含 .git 的目录）
find_root_dir() {
    local dir="$PWD"
    while [[ "$dir" != "/" ]]; do
        if [[ -d "$dir/.git" ]]; then
            echo "$dir"
            return
        fi
        dir="$(dirname "$dir")"
    done
}

# 向上查找 .encryption_key，但优先使用项目根目录的 key
find_key_file() {
    local dir="$PWD"
    local root_dir=""
    local nearest_key=""
    while [[ "$dir" != "/" ]]; do
        if [[ -f "$dir/.encryption_key" ]]; then
            nearest_key="$dir/.encryption_key"
        fi
        if [[ -d "$dir/.git" ]]; then
            root_dir="$dir"
            break
        fi
        dir="$(dirname "$dir")"
    done
    # 如果根目录存在且根目录有 key，优先使用根目录 key
    if [[ -n "$root_dir" ]] && [[ -f "$root_dir/.encryption_key" ]]; then
        echo "$root_dir/.encryption_key"
        return
    fi
    # 否则回退到最近找到的 key
    echo "$nearest_key"
}

KEY_FILE="${ENCRYPTION_KEY_PATH:-}"
if [[ -z "$KEY_FILE" ]] || [[ ! -f "$KEY_FILE" ]]; then
    KEY_FILE=$(find_key_file)
fi

if [[ -z "$KEY_FILE" ]] || [[ ! -f "$KEY_FILE" ]]; then
    # 无密钥时跳过解密，保留 _.file 加载的原始值
    return 0
fi

KEY_HEX=$(openssl enc -base64 -d -in "$KEY_FILE" | xxd -p -c 64)

while IFS= read -r line || [[ -n "$line" ]]; do
    # 跳过注释和空行
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ -z "$line" ]] && continue

    # 只处理 KEY=VALUE 格式
    if [[ "$line" == *=* ]]; then
        key="${line%%=*}"
        value="${line#*=}"

        # 移除可能的引号
        value="${value#\"}"
        value="${value%\"}"
        value="${value#\'}"
        value="${value%\'}"

        if [[ "$value" =~ ^enc: ]]; then
            iv="${value#enc:}"
            iv="${iv%%:*}"
            ciphertext="${value#enc:*:}"

            plaintext=$(openssl enc -aes-256-cbc -d -a -A -nosalt -K "$KEY_HEX" -iv "$iv" <<< "$ciphertext" 2>/dev/null || true)

            if [[ -n "$plaintext" ]]; then
                printf -v "$key" "%s" "$plaintext"
                export "$key"
            fi
        fi
    fi
done < "$ENV_FILE"
