import subprocess, time, os, json, hashlib
from pathlib import Path
from datetime import datetime, timezone
from PIL import ImageGrab
root=Path('/workspace/AcmeUI-Native'); out=root/'target/roadmap-024'; cwd=out/'native-cwd'; cwd.mkdir(exist_ok=True)
now=lambda:datetime.now(timezone.utc).isoformat()
r={'started_at':now(),'source_sha':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'git_status':subprocess.check_output(['git','status','--porcelain'],cwd=root,text=True),'manual_validation':'NOT_RUN','scenario':'Automated X11 CPU Gallery window on isolated Xvfb; default light theme; Mobile page','commands':[]}
app=None; server=None
with (out/'native-window.log').open('w') as log:
    try:
        args=['Xvfb','-displayfd','1','-screen','0','1100x1100x24','-nolisten','tcp']; r['commands'].append(args)
        server=subprocess.Popen(args,stdout=subprocess.PIPE,stderr=log,text=True)
        display=':'+server.stdout.readline().strip()
        env={**os.environ,'DISPLAY':display,'WINIT_UNIX_BACKEND':'x11','ACME_RENDER_BACKEND':'cpu'}
        r['environment']={k:env[k] for k in ['DISPLAY','WINIT_UNIX_BACKEND','ACME_RENDER_BACKEND']}
        args=[str(root/'target/debug/acme-ui-gallery')]; r['commands'].append(args)
        app=subprocess.Popen(args,cwd=cwd,env=env,stdout=log,stderr=log)
        wid=None
        for _ in range(60):
            p=subprocess.run(['xdotool','search','--name','^AcmeUI Gallery$'],env=env,capture_output=True,text=True)
            if p.returncode==0:
                wid=p.stdout.splitlines()[0]; break
            if app.poll() is not None: raise RuntimeError('Gallery exited')
            time.sleep(0.1)
        if not wid: raise RuntimeError('Gallery window did not appear')
        for args in [['xdotool','windowsize','--sync',wid,'1100','1100'],['xdotool','getwindowgeometry','--shell',wid]]:
            r['commands'].append(args)
            p=subprocess.run(args,env=env,capture_output=True,text=True,check=True)
            if 'getwindowgeometry' in args:r['window_geometry']=p.stdout
        time.sleep(1.5)
        path=root/'docs/team/roadmap-024/gallery-mobile-avatars.png'
        image=ImageGrab.grab(xdisplay=display); image.save(path)
        r.update(status='PASS',dimensions=list(image.size),screenshot=str(path.relative_to(root)),screenshot_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),capture='PIL.ImageGrab.grab(xdisplay=display).save(path)',cwd=str(cwd))
    finally:
        for p in [app,server]:
            if p is not None:
                p.terminate()
                try:p.wait(timeout=5)
                except subprocess.TimeoutExpired:p.kill();p.wait()
r.update(finished_at=now(),log=str(out/'native-window.log'),log_sha256=hashlib.sha256((out/'native-window.log').read_bytes()).hexdigest())
(out/'native-window.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps(r,indent=2))
