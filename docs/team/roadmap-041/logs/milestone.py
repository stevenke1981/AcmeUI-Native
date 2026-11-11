import os,subprocess,sys
from pathlib import Path
checks=[
 ('workspace-check',['cargo','check','--locked','--workspace','--all-targets']),
 ('workspace-tests',['cargo','test','--locked','--workspace']),
 ('gallery-build-final',['cargo','build','--locked','-p','acme-gallery']),
 ('gallery-release',['cargo','build','--locked','--release','-p','acme-gallery']),
]
for label,command in checks:
 p=subprocess.run([sys.executable,str(Path(__file__).with_name('check.py')),label,*command],env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1'})
 if p.returncode:sys.exit(p.returncode)
