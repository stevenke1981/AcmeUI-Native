from pathlib import Path
import subprocess,sys,json,hashlib
root=Path.cwd();p=root/'crates/acme-ui/src/layout/paper.rs';saved=p.read_bytes();base='2ad3e8841fff1c591433f92fd7590542cddc5f9f';baseline=subprocess.check_output(['git','show',base+':crates/acme-ui/src/layout/paper.rs'])
try:
 p.write_bytes(baseline)
 subprocess.run([sys.executable,'/tmp/acme-evidence.py','036','layout-baseline','cargo','test','--locked','-p','acme-ui','--no-default-features','--features','layout'],check=False)
 Path('target/roadmap-036/baseline-sources.txt').write_text('Base '+base+'; restored paper.rs from base; new paper_tests.rs unreferenced. All other acme-ui tracked files equal base.\n'+hashlib.sha256(baseline).hexdigest()+'  crates/acme-ui/src/layout/paper.rs\n')
finally:
 p.write_bytes(saved)
assert p.read_bytes()==saved
