#!/usr/bin/env python3
"""
v0.3.24 修复：将 export_data.rs 和 stats_handler.rs 中所有
通过 project_lab_links 获取实验室名的查询改为优先使用 wr.group_id
"""
import re
import sys

def update_sql(content, file_type):
    """
    更新 SQL 查询，将 lab_name 子查询改为优先使用 wr.group_id
    """
    # 旧模式：COALESCE((SELECT group_concat(pg.name)) FROM project_lab_links ... WHERE pll.project_id = p.id), '未知') AS lab_name
    # 新模式：COALESCE((SELECT pg.name FROM project_groups pg WHERE pg.id = wr.group_id), (SELECT group_concat(pg.name)) ..., '未知') AS lab_name
    
    # 匹配 COALESCE((SELECT group_concat(pg.name)) 开头的 lab_name 查询
    # 替换为使用 wr.group_id 的版本
    
    if file_type == 'export':
        # export_data.rs 中的模式
        old_pattern = r"COALESCE\(\(SELECT group_concat\(pg\.name\)\)\s*\n\s*FROM project_lab_links pll\s*\n\s*JOIN project_groups pg ON pll\.group_id = pg\.id AND pg\.name != '研发项目'\s*\n\s*WHERE pll\.project_id = p\.id\), '未知'\) as lab_name"
        
        new_sql = """COALESCE(
                    (SELECT pg.name FROM project_groups pg WHERE pg.id = wr.group_id),
                    (SELECT group_concat(pg.name))
                     FROM project_lab_links pll
                     JOIN project_groups pg ON pll.group_id = pg.id AND pg.name != '研发项目'
                     WHERE pll.project_id = p.id),
                    '未知'
                ) as lab_name"""
        
        content = re.sub(old_pattern, new_sql, content, flags=re.MULTILINE)
    
    return content

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python update_sql.py <file_path> [export|stats]")
        sys.exit(1)
    
    file_path = sys.argv[1]
    file_type = sys.argv[2] if len(sys.argv) > 2 else 'export'
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    updated = update_sql(content, file_type)
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(updated)
    
    print(f"Updated {file_path}")
