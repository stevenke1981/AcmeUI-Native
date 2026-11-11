import hashlib, json, platform, subprocess, sys
from pathlib import Path
root=Path('/workspace/AcmeUI-Native')
sys.path.insert(0,str(root/'scripts'))
from verify_sdk import run_check, probe
label,*command=sys.argv[1:]
evidence=root/'docs/team/roadmap-044/evidence.json'
out=evidence.parent/'logs'
record={'label':label,'source_sha':probe(['git','rev-parse','HEAD'],root),'git_status':probe(['git','status','--porcelain'],root),'os':platform.platform(),'rustc':probe(['rustc','--version'],root),'cargo':probe(['cargo','--version'],root)}
record.update(run_check(command,root,out/f'{label}.txt',2400))
record['log']=f'logs/{label}.txt'
data=json.loads(evidence.read_text());data['checks'].append(record)
evidence.write_text(json.dumps(data,indent=2,ensure_ascii=False)+'\n')
print(label,record['status'],flush=True)
print((out/f'{label}.txt').read_text()[-4000:])
sys.exit(0 if record['status']=='PASS' else 1)
