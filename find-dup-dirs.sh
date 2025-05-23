#!/bin/bash

# 用法说明
if [ $# -ne 1 ]; then
    echo "用法: $0 <搜索目录>"
    exit 1
fi

SEARCH_DIR="$1"

# 检查目录是否存在
if [ ! -d "$SEARCH_DIR" ]; then
    echo "错误: '$SEARCH_DIR' 不是有效的目录。"
    exit 2
fi

# 查找重复命名的目录并打印完整路径
find "$SEARCH_DIR" -type d -printf "%f\t%p\n" 2>/dev/null | \
    sort | awk -F'\t' '
{
    count[$1]++;
    paths[$1] = paths[$1] ? paths[$1] ORS $2 : $2;
}
END {
    for (name in count) {
        if (count[name] > 1) {
            print "=== 重名目录: " name " ===";
            print paths[name];
            print "";
        }
    }
}'
