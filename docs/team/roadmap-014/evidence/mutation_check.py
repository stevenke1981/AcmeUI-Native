"""Compile in-memory coverage mutants; never replace production source files."""

import hashlib
import json
from pathlib import Path
import subprocess


root = Path(__file__).resolve().parents[4]
source_path = root / "crates/acme-render-cpu/src/coverage.rs"
current = source_path.read_text(encoding="utf-8")
tests = (source_path.parent / "coverage/analytic_tests.rs").read_text(encoding="utf-8")
baseline = subprocess.run(
    ["git", "show", "e0e44892997eda7c9ab42b18c303463c750cc602:crates/acme-render-cpu/src/coverage.rs"],
    cwd=root,
    check=True,
    capture_output=True,
).stdout.decode("utf-8")
mutants = {
    "control": current,
    "old_grid_8x8": baseline.replace(
        "use crate::Bounds;", "use crate::Bounds;\n#[cfg(test)]\nmod analytic_tests;", 1
    ),
    "sharp_only": current.replace("rounded_coverage(bounds, radius, cx, cy, base)", "base", 1),
    "incorrect_segment": current.replace("+ 0.5 * r2 * segment;", "+ 0.49 * r2 * segment;", 1),
    "missing_bottom_right": current.replace(
        "(right - radius, bottom - radius, 1.0, 1.0),",
        "(left + radius, top + radius, -1.0, -1.0),",
        1,
    ),
    "no_radius_clamp": current.replace(
        "radius.max(0.0).min(half_x.min(half_y))", "radius.max(0.0)", 1
    ),
}
start = current.index("    let start_y =")
end = current.index("    covered.clamp(0.0, area)", start)
unstable = """    let integral = |x: f64| {
        let y = ((r - x) * (r + x)).max(0.0).sqrt();
        0.5 * (x * y + r2 * (x / r).asin())
    };
    let covered = (full_end - x0) * (y1 - y0)
        + integral(end) - integral(full_end) - y0 * (end - full_end);
"""
mutants["unstable_antiderivative"] = current[:start] + unstable + current[end:]
target = root / "target/coverage-mutations"
target.mkdir(exist_ok=True, parents=True)
results = []
for name, source in mutants.items():
    if name != "control" and source == current:
        raise RuntimeError(f"mutation {name} did not change the source")
    source = source.replace("mod analytic_tests;", "mod analytic_tests {\n" + tests + "\n}", 1)
    harness = (
        "#[derive(Clone, Copy)]\n"
        "struct Bounds { left: f32, top: f32, right: f32, bottom: f32 }\n"
        "mod coverage {\n" + source + "\n}\n"
    )
    binary = target / (name + ".exe")
    compiled = subprocess.run(
        ["rustc", "--edition=2024", "--test", "-O", "-", "--crate-name", "coverage_mutation", "-o", str(binary)],
        input=harness.encode("utf-8"),
        cwd=root,
        capture_output=True,
    )
    if compiled.returncode:
        raise RuntimeError(compiled.stderr.decode("utf-8", errors="replace"))
    run = subprocess.run([str(binary), "analytic_coverage", "--nocapture"], capture_output=True, cwd=root)
    output = (run.stdout + run.stderr).decode("utf-8", errors="replace").replace(str(root), ".")
    result = {"name": name, "compile_exit": compiled.returncode, "test_exit": run.returncode, "output": output}
    results.append(result)
    print(name, "exit", run.returncode)
    assert (run.returncode == 0) == (name == "control"), output

evidence = {
    "base": "e0e44892997eda7c9ab42b18c303463c750cc602",
    "source_sha256": hashlib.sha256(current.encode()).hexdigest(),
    "tests_sha256": hashlib.sha256(tests.encode()).hexdigest(),
    "results": results,
}
(Path(__file__).parent / "mutation.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
