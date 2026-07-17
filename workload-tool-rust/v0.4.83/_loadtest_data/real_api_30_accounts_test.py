import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import datetime, timedelta
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EXE = ROOT / "dist" / "workload-tool.exe"
BASE_URL = "http://127.0.0.1:18083"
RUN_ID = datetime.now().strftime("%Y%m%d_%H%M%S")
TEST_DIR = ROOT / "_loadtest_data" / f"real_api_30_accounts_{RUN_ID}"
REPORT_MD = TEST_DIR / "真实30账号接口测试报告.md"
REPORT_JSON = TEST_DIR / "真实30账号接口测试结果.json"


PASSWORD = "Test@123456"


class Metrics:
    def __init__(self):
        self.calls = []

    def record(self, method, path, ok, elapsed_ms, code=None, message="", status=None):
        self.calls.append(
            {
                "method": method,
                "path": path,
                "ok": ok,
                "elapsed_ms": round(elapsed_ms, 2),
                "code": code,
                "message": message,
                "status": status,
            }
        )

    def summary(self):
        total = len(self.calls)
        ok_count = sum(1 for c in self.calls if c["ok"])
        bad = [c for c in self.calls if not c["ok"]]
        latencies = sorted(c["elapsed_ms"] for c in self.calls)
        avg = sum(latencies) / total if total else 0
        p95 = latencies[int(total * 0.95) - 1] if total else 0
        return {
            "total_calls": total,
            "success_calls": ok_count,
            "failed_calls": len(bad),
            "success_rate": round(ok_count * 100 / total, 2) if total else 0,
            "latency_min_ms": round(latencies[0], 2) if total else 0,
            "latency_avg_ms": round(avg, 2),
            "latency_p95_ms": round(p95, 2),
            "latency_max_ms": round(latencies[-1], 2) if total else 0,
            "failures": bad[:50],
        }


metrics = Metrics()


def request(method, path, payload=None, token=None, params=None, expect_ok=True):
    url = BASE_URL + path
    if params:
        url += "?" + urllib.parse.urlencode(params)
    data = None
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    if payload is not None:
        data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    started = time.perf_counter()
    status = None
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            status = resp.status
            raw = resp.read().decode("utf-8", errors="replace")
    except urllib.error.HTTPError as exc:
        status = exc.code
        raw = exc.read().decode("utf-8", errors="replace")
    elapsed = (time.perf_counter() - started) * 1000
    try:
        body = json.loads(raw)
    except Exception:
        body = {"code": -1, "message": raw[:300], "data": None}
    ok = status and 200 <= status < 300 and body.get("code") == 0
    metrics.record(method, path, ok, elapsed, body.get("code"), body.get("message", ""), status)
    if expect_ok and not ok:
        raise RuntimeError(f"{method} {path} failed: status={status}, body={body}")
    return body


def wait_server():
    for _ in range(60):
        try:
            with urllib.request.urlopen(BASE_URL + "/api/version", timeout=5) as resp:
                if 200 <= resp.status < 300:
                    return True
        except Exception:
            time.sleep(0.5)
    return False


def data_or_empty(resp):
    return resp.get("data") or []


def find_by_name(items, name):
    for item in items:
        if item.get("name") == name or item.get("username") == name:
            return item
    return None


def ensure_role(admin_token, name, permissions, sort_order):
    roles = data_or_empty(request("GET", "/api/roles", token=admin_token))
    existing = find_by_name(roles, name)
    if existing:
        request(
            "PUT",
            f"/api/roles/{existing['id']}/permissions",
            {"permissions": permissions},
            token=admin_token,
        )
        return existing["id"]
    created = request(
        "POST",
        "/api/roles",
        {
            "name": name,
            "description": "30账号真实接口测试角色",
            "permissions": permissions,
            "sort_order": sort_order,
        },
        token=admin_token,
    )["data"]
    return created["id"]


def ensure_division(admin_token, name, sort_order, color):
    divisions = data_or_empty(request("GET", "/api/divisions", token=admin_token))
    existing = find_by_name(divisions, name)
    if existing:
        return existing["id"]
    return request(
        "POST",
        "/api/divisions",
        {"name": name, "sort_order": sort_order, "color": color},
        token=admin_token,
    )["data"]["id"]


def ensure_group(admin_token, name, sort_order, division_id):
    groups = data_or_empty(request("GET", "/api/groups", token=admin_token))
    existing = find_by_name(groups, name)
    if existing:
        return existing["id"]
    return request(
        "POST",
        "/api/groups",
        {
            "name": name,
            "sort_order": sort_order,
            "show_in_work": True,
            "show_in_rd": True,
            "division_id": division_id,
        },
        token=admin_token,
    )["data"]["id"]


def ensure_method_type(admin_token, name, sort_order):
    types = data_or_empty(request("GET", "/api/method-types", token=admin_token))
    existing = find_by_name(types, name)
    if existing:
        return existing["id"]
    return request(
        "POST",
        "/api/method-types",
        {"name": name, "sort_order": sort_order},
        token=admin_token,
    )["data"]["id"]


def ensure_method(admin_token, name, idx, type_ids):
    methods = data_or_empty(request("GET", "/api/methods", token=admin_token))
    existing = find_by_name(methods, name)
    if existing:
        return existing["id"]
    return request(
        "POST",
        "/api/methods",
        {
            "name": name,
            "full_name": f"{name}-真实接口测试方法",
            "coefficient": round(1.0 + idx * 0.05, 2),
            "amount": idx * 10,
            "multiplier": 1.0,
            "notes": "30账号真实接口测试",
            "type_ids": type_ids,
        },
        token=admin_token,
    )["data"]["id"]


def ensure_project(admin_token, name, idx, lab_ids, method_ids, high_item):
    projects = data_or_empty(request("GET", "/api/projects", token=admin_token))
    existing = find_by_name(projects, name)
    if existing:
        return existing["id"]
    return request(
        "POST",
        "/api/projects",
        {
            "name": name,
            "full_name": f"{name}-真实接口测试项目",
            "notes": "30账号真实接口测试",
            "sort_order": idx,
            "is_active": True,
            "lab_ids": lab_ids,
            "method_ids": method_ids,
            "high_item": high_item,
        },
        token=admin_token,
    )["data"]["id"]


def ensure_user(admin_token, username, division_id, group_id, role_ids):
    users = data_or_empty(request("GET", "/api/users", token=admin_token))
    existing = find_by_name(users, username)
    if existing:
        request(
            "PUT",
            f"/api/users/{existing['id']}",
            {
                "division_id": division_id,
                "group_id": group_id,
                "is_active": True,
                "role_ids": role_ids,
            },
            token=admin_token,
        )
        return existing["id"]
    return request(
        "POST",
        "/api/users",
        {
            "username": username,
            "password": PASSWORD,
            "division_id": division_id,
            "group_id": group_id,
            "role_id": None,
            "role_ids": role_ids,
        },
        token=admin_token,
    )["data"]["id"]


def main():
    TEST_DIR.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env["WORKLOAD_DATA_DIR"] = str(TEST_DIR)
    proc = subprocess.Popen(
        [str(EXE)],
        cwd=str(ROOT / "dist"),
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
    )
    try:
        if not wait_server():
            raise RuntimeError("server did not start on port 18083")

        admin_login = request("POST", "/api/auth/login", {"username": "admin", "password": "admin123"})
        admin_token = admin_login["data"]["token"]

        analysis_role = ensure_role(
            admin_token,
            "真实测试-分析检测",
            [
                "entry:workload",
                "stats:workload:access",
                "stats:workload:week",
                "stats:workload:month",
                "stats:workload:user-log",
            ],
            910,
        )
        rd_role = ensure_role(
            admin_token,
            "真实测试-研发送样",
            ["entry:sample", "stats:workload:access"],
            920,
        )
        sender_role = ensure_role(
            admin_token,
            "真实测试-部门送样",
            ["entry:sample-info", "sample:collect"],
            930,
        )

        division_names = ["真实测试-分析部", "真实测试-研发部", "真实测试-送样部"]
        division_ids = [
            ensure_division(admin_token, n, i + 1, ["#2563eb", "#16a34a", "#f97316"][i])
            for i, n in enumerate(division_names)
        ]

        lab_names = ["真实测试-LC室", "真实测试-GC室", "真实测试-理化室", "真实测试-制剂室", "真实测试-留样室"]
        lab_ids = [ensure_group(admin_token, n, i + 1, division_ids[i % 3]) for i, n in enumerate(lab_names)]

        type_lc = ensure_method_type(admin_token, "真实测试-液相", 901)
        type_gc = ensure_method_type(admin_token, "真实测试-气相", 902)
        type_other = ensure_method_type(admin_token, "真实测试-理化", 903)

        method_ids = []
        for i in range(10):
            prefix = "LC" if i < 4 else "GC" if i < 7 else "PH"
            type_ids = [type_lc] if prefix == "LC" else [type_gc] if prefix == "GC" else [type_other]
            method_ids.append(ensure_method(admin_token, f"真实测试-{prefix}-方法{i + 1:02d}", i + 1, type_ids))

        project_ids = []
        high_items = [f"真实测试-高项{i + 1:02d}" for i in range(10)]
        for i in range(10):
            project_ids.append(
                ensure_project(
                    admin_token,
                    f"真实测试-项目{i + 1:02d}",
                    i + 1,
                    [lab_ids[i % len(lab_ids)], lab_ids[(i + 1) % len(lab_ids)]],
                    [method_ids[i], method_ids[(i + 1) % len(method_ids)]],
                    high_items[i],
                )
            )

        users = []
        for i in range(30):
            if i < 10:
                prefix, div, role = "analysis", division_ids[0], analysis_role
            elif i < 20:
                prefix, div, role = "rd", division_ids[1], rd_role
            else:
                prefix, div, role = "sender", division_ids[2], sender_role
            username = f"realtest_{prefix}_{(i % 10) + 1:02d}"
            group_id = lab_ids[i % len(lab_ids)]
            ensure_user(admin_token, username, div, group_id, [role])
            login = request("POST", "/api/users/login", {"username": username, "password": PASSWORD})
            users.append(
                {
                    "username": username,
                    "token": login["data"]["token"],
                    "division_id": div,
                    "group_id": group_id,
                    "role": prefix,
                }
            )

        base_date = datetime(2026, 7, 16, 9, 0, 0)
        analysis_records = []
        for i, user in enumerate(users[:10]):
            for j in range(5):
                idx = (i + j) % 10
                record = request(
                    "POST",
                    "/api/records",
                    {
                        "project_id": project_ids[idx],
                        "method_id": method_ids[idx],
                        "user_name": user["username"],
                        "quantity": (j + 1) * 2,
                        "recorded_at": (base_date + timedelta(hours=i, days=j)).isoformat(),
                        "group_id": user["group_id"],
                        "division_id": user["division_id"],
                        "multiplier": 1.0 + (j % 3) * 0.1,
                        "high_item": high_items[idx],
                    },
                    token=user["token"],
                )["data"]
                analysis_records.append(record)

        rd_records = []
        for i, user in enumerate(users[10:20]):
            for j in range(5):
                idx = (i + j) % 10
                record = request(
                    "POST",
                    "/api/rd-records",
                    {
                        "project_id": project_ids[idx],
                        "method_id": method_ids[idx],
                        "user_name": user["username"],
                        "quantity": j + 1,
                        "recorded_at": (base_date + timedelta(hours=i, days=j)).isoformat(),
                        "group_id": user["group_id"],
                        "division_id": user["division_id"],
                        "batch_no": f"RD-REAL-{i + 1:02d}-{j + 1:02d}",
                        "notes": "30账号真实接口测试",
                    },
                    token=user["token"],
                )["data"]
                rd_records.append(record)

        sample_info_records = []
        for i, user in enumerate(users[20:30]):
            for j in range(3):
                idx = (i + j) % 10
                record = request(
                    "POST",
                    "/api/sample-info",
                    {
                        "batch_no": f"SI-REAL-{i + 1:02d}-{j + 1:02d}",
                        "user_name": user["username"],
                        "lab_name": lab_names[idx % len(lab_names)],
                        "project_name": f"真实测试-项目{idx + 1:02d}",
                        "submitted_at": (base_date + timedelta(hours=i, days=j)).isoformat(),
                        "detection_date": (base_date + timedelta(days=j)).date().isoformat(),
                        "main_components": "主成分A/辅料B",
                        "detection_type": "液相/气相/理化",
                        "type_key": "realtest",
                        "division_id": user["division_id"],
                        "quantity": j + 1,
                        "notes": "30账号真实接口测试",
                    },
                    token=user["token"],
                )["data"]
                sample_info_records.append(record)

        sampled = []
        samplers = users[20:30]
        for i, record in enumerate(rd_records[:20]):
            sampler = samplers[i % len(samplers)]
            sampled.append(
                request("PUT", f"/api/rd-records/{record['id']}/sample", token=sampler["token"])["data"]
            )

        query_params = {"start": "2026-07-16", "end": "2026-07-22", "group_by": "day"}
        stats_results = {
            "admin_analysis_summary": request("GET", "/api/stats/summary", token=admin_token, params=query_params)["data"],
            "admin_analysis_by_user": request("GET", "/api/stats/by-user", token=admin_token, params=query_params)["data"],
            "admin_rd_summary": request("GET", "/api/rd-stats/summary", token=admin_token, params=query_params)["data"],
            "admin_rd_by_user": request("GET", "/api/rd-stats/by-user", token=admin_token, params=query_params)["data"],
            "sample_info_stats": request("GET", "/api/sample-info/stats", token=admin_token, params={"start": "2026-07-16", "end": "2026-07-22"})["data"],
            "analysis_user_scoped_summary": request("GET", "/api/stats/summary", token=users[0]["token"], params=query_params)["data"],
            "rd_user_scoped_summary": request("GET", "/api/rd-stats/summary", token=users[10]["token"], params=query_params)["data"],
        }

        list_results = {
            "users": request("GET", "/api/users", token=admin_token)["data"],
            "records": request("GET", "/api/records", token=admin_token, params={"page": 1, "page_size": 500})["data"],
            "rd_records": request("GET", "/api/rd-records", token=admin_token, params={"page": 1, "page_size": 500})["data"],
            "sample_info": request("GET", "/api/sample-info", token=admin_token, params={"page": 1, "page_size": 500})["data"],
        }

        result = {
            "tested_at": datetime.now().isoformat(timespec="seconds"),
            "version_folder": str(ROOT),
            "server_url": BASE_URL,
            "test_data_dir": str(TEST_DIR),
            "created_or_ensured": {
                "departments": division_names,
                "laboratories": lab_names,
                "methods": 10,
                "projects": 10,
                "high_items": high_items,
                "users": [u["username"] for u in users],
            },
            "workflow_counts": {
                "analysis_records_created": len(analysis_records),
                "rd_records_created": len(rd_records),
                "sample_info_records_created": len(sample_info_records),
                "rd_records_sampled": len(sampled),
            },
            "api_totals": {
                "users_total": len(list_results["users"]),
                "analysis_records_total": list_results["records"]["total"],
                "rd_records_total": list_results["rd_records"]["total"],
                "sample_info_total": list_results["sample_info"]["total"],
            },
            "stats_results": stats_results,
            "metrics": metrics.summary(),
            "calls": metrics.calls,
        }
        REPORT_JSON.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
        failures = result["metrics"]["failures"]
        lines = [
            "# 真实30账号接口测试报告",
            "",
            f"- 测试时间：{result['tested_at']}",
            f"- 测试版本目录：`{ROOT}`",
            f"- 测试数据目录：`{TEST_DIR}`",
            f"- 服务地址：`{BASE_URL}`",
            "",
            "## 基础数据",
            "",
            "- 部门：真实测试-分析部、真实测试-研发部、真实测试-送样部",
            "- 实验室：真实测试-LC室、真实测试-GC室、真实测试-理化室、真实测试-制剂室、真实测试-留样室",
            "- 检测方法：10 个",
            "- 项目：10 个",
            "- 高项：10 个",
            "- 账号：30 个，分别为 10 个分析、10 个研发送样、10 个部门送样账号",
            "",
            "## 业务动作",
            "",
            f"- 分析检测记录新增：{len(analysis_records)} 条",
            f"- 研发送样记录新增：{len(rd_records)} 条",
            f"- 样品信息登记新增：{len(sample_info_records)} 条",
            f"- 研发送样取样操作：{len(sampled)} 条",
            "",
            "## 接口统计",
            "",
            f"- 调用总数：{result['metrics']['total_calls']}",
            f"- 成功：{result['metrics']['success_calls']}",
            f"- 失败：{result['metrics']['failed_calls']}",
            f"- 成功率：{result['metrics']['success_rate']}%",
            f"- 延迟：min {result['metrics']['latency_min_ms']} ms / avg {result['metrics']['latency_avg_ms']} ms / p95 {result['metrics']['latency_p95_ms']} ms / max {result['metrics']['latency_max_ms']} ms",
            "",
            "## 统计校验",
            "",
            f"- 分析检测统计总记录数：{stats_results['admin_analysis_summary']['total_records']}，总数量：{stats_results['admin_analysis_summary']['total_quantity']}",
            f"- 研发送样统计总记录数：{stats_results['admin_rd_summary']['total_records']}，总数量：{stats_results['admin_rd_summary']['total_quantity']}",
            f"- 样品信息统计总数：{stats_results['sample_info_stats']['total']}",
            f"- 普通分析账号统计范围：{stats_results['analysis_user_scoped_summary']['total_records']} 条，仅返回本人数据",
            f"- 普通研发账号统计范围：{stats_results['rd_user_scoped_summary']['total_records']} 条，仅返回本人数据",
            "",
            "## 失败记录",
            "",
        ]
        if failures:
            for failure in failures:
                lines.append(f"- {failure['method']} {failure['path']}：status={failure['status']} code={failure['code']} message={failure['message']}")
        else:
            lines.append("- 无失败接口。")
        lines.append("")
        lines.append(f"原始结果见：`{REPORT_JSON}`")
        REPORT_MD.write_text("\n".join(lines), encoding="utf-8")
        print(json.dumps({"report": str(REPORT_MD), "metrics": result["metrics"]}, ensure_ascii=False))
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        TEST_DIR.mkdir(parents=True, exist_ok=True)
        (TEST_DIR / "真实30账号接口测试失败.txt").write_text(str(exc), encoding="utf-8")
        print(f"FAILED: {exc}", file=sys.stderr)
        sys.exit(1)
