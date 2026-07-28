use anyhow::Result;
use chrono::Utc;
use std::io::Write;

fn ts() -> String {
    Utc::now().to_rfc3339()
}
fn load(p: &str) -> Result<serde_json::Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(p)?)?)
}
fn save(p: &str, d: &serde_json::Value) -> Result<()> {
    let tmp = format!("{p}.tmp");
    let mut f = std::fs::File::create(&tmp)?;
    f.write_all(serde_json::to_string_pretty(d)?.as_bytes())?;
    f.write_all(b"\n")?;
    f.flush()?;
    drop(f);
    std::fs::rename(&tmp, p)?;
    Ok(())
}
fn arg(a: &[String], n: &str) -> String {
    a.iter()
        .position(|x| x == n)
        .and_then(|i| a.get(i + 1))
        .cloned()
        .unwrap_or_default()
}

pub fn cmd_init(a: &[String]) -> Result<()> {
    let (ns, yml, sf) = (
        arg(a, "--namespace"),
        arg(a, "--pipeline-yml"),
        arg(a, "--state-file"),
    );
    if ns.is_empty() || yml.is_empty() || sf.is_empty() {
        eprintln!("Usage: init --namespace <ns> --pipeline-yml <p> --state-file <p>");
        std::process::exit(1);
    }
    let ids: Vec<String> = std::fs::read_to_string(&yml)?
        .lines()
        .filter(|l| l.trim().starts_with("- id:"))
        .filter_map(|l| l.split(':').nth(1).map(|s| s.trim().to_string()))
        .collect();
    let mut st = serde_json::Map::new();
    for id in &ids {
        st.insert(id.clone(), serde_json::json!({"status":"pending","gates":[],"pending_asks":[],"artifacts":[],"started_at":null,"completed_at":null}));
    }
    save(
        &sf,
        &serde_json::json!({"pipeline_name":"pipeline","namespace":ns,"created_at":ts(),"updated_at":ts(),"stages":st}),
    )?;
    println!("state_file: {sf}");
    Ok(())
}

pub fn cmd_consume(a: &[String]) -> Result<()> {
    let (m, yml, sf) = (
        arg(a, "--manifest"),
        arg(a, "--pipeline-yml"),
        arg(a, "--state-file"),
    );
    if m.is_empty() || yml.is_empty() || sf.is_empty() {
        eprintln!("Usage: consume --manifest <p> --pipeline-yml <p> --state-file <p>");
        std::process::exit(1);
    }
    let man = load(&m)?;
    let (ns, code) = (
        man["namespace"].as_str().unwrap_or("").to_string(),
        man["app_code"].as_str().unwrap_or("").to_string(),
    );
    if ns.is_empty() {
        eprintln!("Invalid manifest");
        std::process::exit(1);
    }
    let content = std::fs::read_to_string(&yml)?;
    let mut br: Vec<String> = vec![];
    let mut ids: Vec<String> = vec![];
    let mut cur;
    for line in content.lines() {
        let s = line.trim();
        if s.starts_with("- id:") {
            cur = s.split(':').nth(1).unwrap_or("").trim().to_string();
            ids.push(cur.clone());
        } else if s.starts_with("# @bypass-refs:") {
            br = s
                .split(':')
                .nth(1)
                .unwrap_or("")
                .split_whitespace()
                .map(|x| x.to_string())
                .collect();
        }
    }
    let mut st = serde_json::Map::new();
    for id in &ids {
        let bypass = br.contains(id) || id == "appagent_ready";
        st.insert(id.clone(), serde_json::json!({"status": if bypass {"completed"} else {"pending"}, "via": if bypass {serde_json::json!("appagent")} else {serde_json::Value::Null}, "gates":[],"pending_asks":[],"artifacts":[],"started_at":null,"completed_at":null}));
    }
    let fp = ids
        .iter()
        .find(|id| st[*id]["status"] == "pending")
        .cloned()
        .unwrap_or_else(|| "factor_dev".into());
    save(
        &sf,
        &serde_json::json!({"pipeline_name":"pipeline","namespace":ns,"app_code":code,"created_at":ts(),"updated_at":ts(),"current_stage":fp,"stages":st}),
    )?;
    println!("state_file: {sf}");
    Ok(())
}

pub fn cmd_print_summary(a: &[String]) -> Result<()> {
    let sf = arg(a, "--state-file");
    if sf.is_empty() {
        eprintln!("Usage: print-summary --state-file <p>");
        std::process::exit(1);
    }
    let s = load(&sf)?;
    let b: Vec<String> = s["stages"]
        .as_object()
        .map(|m| {
            m.keys()
                .filter(|k| m[*k]["via"] == "appagent" && *k != "appagent_ready")
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let c = s["stages"]
        .as_object()
        .map(|m| m.values().filter(|v| v["status"] == "completed").count())
        .unwrap_or(0);
    let p = s["stages"]
        .as_object()
        .map(|m| m.values().filter(|v| v["status"] == "pending").count())
        .unwrap_or(0);
    println!("namespace: {}", s["namespace"].as_str().unwrap_or("?"));
    if let Some(a) = s["app_code"].as_str().filter(|x| !x.is_empty()) {
        println!("app_code: {a}");
    }
    println!(
        "current_stage: {}",
        s["current_stage"].as_str().unwrap_or("?")
    );
    if !b.is_empty() {
        println!("bypassed: {}", b.join(" "));
    }
    println!("completed: {c}");
    println!("pending: {p}");
    Ok(())
}

pub fn cmd_set_stage(a: &[String]) -> Result<()> {
    let (st, sts, sf) = (
        arg(a, "--stage"),
        arg(a, "--status"),
        arg(a, "--state-file"),
    );
    if st.is_empty() || sts.is_empty() || sf.is_empty() {
        eprintln!("Usage: set-stage ...");
        std::process::exit(1);
    }
    let mut s = load(&sf)?;
    s["stages"][&st]["status"] = serde_json::json!(&sts);
    s["updated_at"] = serde_json::json!(ts());
    save(&sf, &s)?;
    println!("stage: {st} status: {sts}");
    Ok(())
}

pub fn cmd_check_requires(a: &[String]) -> Result<()> {
    let (sid, yml, sf) = (
        arg(a, "--stage"),
        arg(a, "--pipeline-yml"),
        arg(a, "--state-file"),
    );
    if sid.is_empty() || yml.is_empty() || sf.is_empty() {
        eprintln!("Usage: check-requires ...");
        std::process::exit(1);
    }
    let state = load(&sf)?;
    let mut reqs: Vec<String> = vec![];
    let mut ins = false;
    for line in std::fs::read_to_string(&yml)?.lines() {
        let s = line.trim();
        if s.starts_with("- id:") {
            ins = s.split(':').nth(1).unwrap_or("").trim() == sid;
        } else if ins && s.starts_with("requires:") {
            reqs = s
                .split(':')
                .nth(1)
                .unwrap_or("")
                .split(',')
                .map(|x| x.trim().trim_matches('"').to_string())
                .collect();
        }
    }
    for r in &reqs {
        if state["stages"][r]["status"].as_str().unwrap_or("") != "completed" {
            eprintln!("REQUIRE_FAIL: {r}");
            std::process::exit(1);
        }
    }
    println!("REQUIRES_PASS: {sid}");
    Ok(())
}

pub fn cmd_complete_stage(a: &[String]) -> Result<()> {
    let (sid, sf) = (arg(a, "--stage"), arg(a, "--state-file"));
    if sid.is_empty() || sf.is_empty() {
        eprintln!("Usage: complete-stage ...");
        std::process::exit(1);
    }
    let mut s = load(&sf)?;
    let t = ts();
    s["stages"][&sid]["status"] = serde_json::json!("completed");
    s["stages"][&sid]["completed_at"] = serde_json::json!(&t);
    s["current_stage"] = serde_json::json!(&sid);
    s["updated_at"] = serde_json::json!(&t);
    save(&sf, &s)?;
    println!("COMPLETED: {sid}");
    Ok(())
}

pub fn cmd_resolve_gate(a: &[String]) -> Result<()> {
    let (sid, gid, sf) = (arg(a, "--stage"), arg(a, "--gate"), arg(a, "--state-file"));
    if sid.is_empty() || gid.is_empty() || sf.is_empty() {
        eprintln!("Usage: resolve-gate ...");
        std::process::exit(1);
    }
    let mut s = load(&sf)?;
    s["stages"][&sid]["resolved_gates"]
        .as_array_mut()
        .unwrap_or(&mut vec![])
        .push(serde_json::json!({"id":gid,"resolved_at":ts()}));
    save(&sf, &s)?;
    println!("RESOLVED: {gid}");
    Ok(())
}
