import os,subprocess,sys
from pathlib import Path
checks=[
 ('targeted-final',['cargo','test','--locked','-p','acme-ui','--all-features','-p','acme-ui-gallery']),
 ('fmt',['cargo','fmt','-p','acme-ui','-p','acme-ui-gallery','--','--check']),
 ('clippy',['cargo','clippy','--locked','-p','acme-ui','-p','acme-ui-gallery','--all-features','--all-targets','--no-deps','--','-D','warnings']),
 ('workspace-check',['cargo','check','--locked','--workspace','--all-targets']),
 ('workspace-tests',['cargo','test','--locked','--workspace']),
 ('gallery-build',['cargo','build','--locked','-p','acme-ui-gallery']),
 ('gallery-release',['cargo','build','--locked','--release','-p','acme-ui-gallery']),
]
for label,command in checks:
 p=subprocess.run([sys.executable,str(Path(__file__).with_name('check.py')),label,*command],env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1'})
 if p.returncode:sys.exit(p.returncode)
p=subprocess.run([sys.executable,'scripts/verify_sdk.py','--profile','cpu','--output','docs/team/roadmap-042/logs/cpu'])
sys.exit(p.returncode)
