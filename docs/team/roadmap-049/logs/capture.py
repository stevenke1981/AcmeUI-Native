"""Automated real X11/CPU Gallery pointer evidence, not human acceptance."""
import hashlib, json, os, platform, subprocess, sys, time
from datetime import datetime, timezone
from pathlib import Path
from PIL import ImageGrab
root=Path(__file__).resolve().parents[4]
out=root/'docs/team/roadmap-049'; work=root/'target/roadmap-049/native-cwd';work.mkdir(parents=True,exist_ok=True)
(work/'acme-gallery-theme.conf').write_text('pack=default\ndark=false\n')
now=lambda:datetime.now(timezone.utc).isoformat()
r={'started_at':now(),'source_sha':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'git_status':subprocess.check_output(['git','status','--porcelain'],cwd=root,text=True),'os':platform.platform(),'manual_validation':'NOT_RUN','scenario':'Layout Anchor: actual pointer API then GET then Introduction; controlled messages only, no automatic scrolling or human sign-off','commands':[],'captures':[]}
r['binary_sha256']=hashlib.sha256((root/'target/debug/acme-ui-gallery').read_bytes()).hexdigest()
server=app=None
with (out/'logs/native-window.txt').open('w') as log:
    try:
        args=['Xvfb','-displayfd','1','-screen','0','1100x1000x24','-nolisten','tcp'];r['commands'].append(args)
        server=subprocess.Popen(args,stdout=subprocess.PIPE,stderr=log,text=True)
        display=':'+server.stdout.readline().strip()
        env={**os.environ,'DISPLAY':display,'WINIT_UNIX_BACKEND':'x11','ACME_RENDER_BACKEND':'cpu'}
        r['environment']={key:env[key] for key in ['DISPLAY','WINIT_UNIX_BACKEND','ACME_RENDER_BACKEND']}
        args=[str(root/'target/debug/acme-ui-gallery')];r['commands'].append(args)
        app=subprocess.Popen(args,cwd=work,env=env,stdout=log,stderr=log)
        wid=None
        for _ in range(100):
            p=subprocess.run(['xdotool','search','--name','^AcmeUI Gallery$'],env=env,capture_output=True,text=True)
            if p.returncode==0:wid=p.stdout.splitlines()[0];break
            if app.poll() is not None:raise RuntimeError('Gallery exited')
            time.sleep(.1)
        if not wid:raise RuntimeError('No Gallery window')
        def run(*args):
            r['commands'].append(list(args))
            return subprocess.run(args,env=env,check=True,capture_output=True,text=True,timeout=15).stdout
        run('xdotool','windowsize','--sync',wid,'1100','900')
        r['geometry']=run('xdotool','getwindowgeometry','--shell',wid)
        run('xdotool','windowfocus','--sync',wid)
        time.sleep(1)
        run('xdotool','mousemove','--window',wid,'80','198','click','1')
        time.sleep(.5)
        def capture(name):
            path=out/f'gallery-anchor-{name}.png';im=ImageGrab.grab(xdisplay=display);im.save(path)
            r['captures'].append({'captured_at':now(),'path':str(path.relative_to(root)),'dimensions':list(im.size),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
        run('xdotool','mousemove','--window',wid,'1000','750')
        # The Anchor demo is visible at the top of Layout; no scrolling.
        time.sleep(.6)
        capture('before')
        print('Anchor capture ready; awaiting API/GET/Introduction coordinates.', flush=True)
        coordinates = sys.argv[1:] or input().split()
        if len(coordinates) != 6: raise ValueError('Expected six click coordinates')
        r['click_coordinates'] = [int(value) for value in coordinates]
        for index in range(0, 6, 2):
            run('xdotool','mousemove','--window',wid,coordinates[index],coordinates[index+1],'click','1')
            time.sleep(.6)
        time.sleep(.6)
        run('xdotool','mousemove','--window',wid,'1000','750')
        capture('after')
        r['status']='PASS'
    except Exception as error:
        r.update(status='FAIL',reason=str(error));raise
    finally:
        for p in [app,server]:
            if p is not None:
                p.terminate()
                try:p.wait(timeout=5)
                except subprocess.TimeoutExpired:p.kill();p.wait()
        r.update(finished_at=now(),log='logs/native-window.txt',log_sha256=hashlib.sha256((out/'logs/native-window.txt').read_bytes()).hexdigest())
        (out/'logs/native-window.json').write_text(json.dumps(r,indent=2)+'\n')
print(r['status'])
