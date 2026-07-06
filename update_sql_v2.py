#!/usr/bin/env python3
"""
v0.3.24: 批量更新 export_data.rs 和 stats_handler.rs
将所有通过 project_lab_links 获取实验室名的查询
改为使用 wr.group_id + LEFT JOIN project_groups
"""
import re
import sys

def update_export_data(content):
    """
    更新 export_data.rs:
    1. 将所有 COALESCE((SELECT group_concat(pg.name)) ... '未知') as lab_name,
       替换为 COALESCE(pg.name, '未知') as lab_name,
    2. 在 FROM work_records wr 后添加 LEFT JOIN project_groups pg ON pg.id = wr.group_id
    """
    # 替换 lab_name 子查询为简单 COALESCE(pg.name, '未知')
    # 匹配多行模式
    pattern1 = r"COALESCE\(\(SELECT group_concat\(pg\.name\)\)\s*\n\s*FROM project_lab_links pll\s*\n\s*JOIN project_groups pg ON pll\.group_id = pg\.id AND pg\.name != '研发项目'\s*\n\s*WHERE pll\.project_id = p\.id\), '未知'\) as lab_name"
    
    replacement1 = "COALESCE(pg.name, '未知') as lab_name"
    
    content = re.sub(pattern1, replacement1, content, flags=re.MULTILINE)
    
    # 在 FROM work_records wr 后添加 LEFT JOIN project_groups pg ON pg.id = wr.group_id
    # 但要注意：有些查询可能没有 work_records wr，需要检查
    # 先处理有明确 FROM work_records wr 的情况
    pattern2 = r"(FROM work_records wr\s*\n\s*)JOIN projects p ON wr\.project_id = p\.id"
    replacement2 = r"\1LEFT JOIN project_groups pg ON pg.id = wr.group_id\n         JOIN projects p ON wr.project_id = p.id"
    
    content = re.sub(pattern2, replacement2, content, flags=re.MULTILINE)
    
    return content

def update_stats_handler(content):
    """
    更新 stats_handler.rs:
    类似地，将实验室名查询改为使用 wr.group_id
    """
    # 替换 lab_name 子查询
    pattern1 = r"COALESCE\(\(SELECT group_concat\(pg\.name\)\)\s*\n\s*FROM project_lab_links pll\s*\n\s*JOIN project_groups pg ON pll\.group_id = pg\.id AND pg\.name != '研发项目'\s*\n\s*WHERE pll\.project_id = p\.id\), '未分组'\) as group_name"
    
    replacement1 = "COALESCE(pg.name, '未分组') as group_name"
    
    content = re.sub(pattern1, replacement1, content, flags=re.MULTILINE)
    
    # 添加 JOIN
    pattern2 = r"(FROM work_records wr\s*\n\s*)JOIN projects p ON wr\.project_id = p\.id"
    replacement2 = r"\1LEFT JOIN project_groups pg ON pg.id = wr.group_id\n         JOIN projects p ON wr.project_id = p.id"
    
    content = re.sub(pattern2, replacement2, content, flags=re.MULTILINE)
    
    return content

if __name__ == '__main__':
    files = [
        ('D:/桌面/工作量统计工具项目/workload-tool-rust/v0.3.24/src/api/export_data.rs', 'export'),
        ('D:/桌面/工作量统计工具项目/workload-tool-rust/v0.3.24/src/api/stats_handler.rs', 'stats'),
    ]
    
    for file_path, file_type in files:
        print(f"Processing {file_path}...")
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        if file_type == 'export':
            updated = update_export_data(content)
        else:
            updated = update_stats_handler(content)
        
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(updated)
        
        print(f"  Updated {file_path}")
    
    print("Done!")
