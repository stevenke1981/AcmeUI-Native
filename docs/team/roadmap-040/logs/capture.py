import subprocess,time,os,json,hashlib,sys,platform
from pathlib import Path
from datetime import datetime,timezone
from PIL import ImageGrab, Image, ImageChops
batch,slug,*clicks=sys.argv[1:]
root=Path('/workspace/AcmeUI-Native');out=root/f'target/roadmap-{batch}';cwd=out/'native-cwd';cwd.mkdir(exist_ok=True)
now=lambda:datetime.now(timezone.utc).isoformat()
r={'script_argv':sys.argv,'started_at':now(),'source_sha':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'git_status':subprocess.check_output(['git','status','--porcelain'],cwd=root,text=True),'os':platform.platform(),'manual_validation':'NOT_RUN','scenario':'Automated X11 CPU Gallery window on isolated Xvfb; default light theme; Spinner normal and reduced-motion poses; actual scroll and pointer toggle','commands':[],'captures':[]}
app=None;server=None
with (out/'native-window.txt').open('w') as log:
    try:
        args=['Xvfb','-displayfd','1','-screen','0','1100x1100x24','-nolisten','tcp'];r['commands'].append(args)
        server=subprocess.Popen(args,stdout=subprocess.PIPE,stderr=log,text=True)
        display=':'+server.stdout.readline().strip()
        env={**os.environ,'DISPLAY':display,'WINIT_UNIX_BACKEND':'x11','ACME_RENDER_BACKEND':'cpu'}
        r['environment']={k:env[k] for k in ['DISPLAY','WINIT_UNIX_BACKEND','ACME_RENDER_BACKEND']}
        args=[str(root/'target/debug/acme-ui-gallery')];r['commands'].append(args)
        app=subprocess.Popen(args,cwd=cwd,env=env,stdout=log,stderr=log)
        wid=None
        for _ in range(100):
            p=subprocess.run(['xdotool','search','--name','^AcmeUI Gallery$'],env=env,capture_output=True,text=True)
            if p.returncode==0:wid=p.stdout.splitlines()[0];break
            if app.poll() is not None:raise RuntimeError('Gallery exited')
            time.sleep(.1)
        if not wid:raise RuntimeError('No Gallery window')
        for args in [['xdotool','windowsize','--sync',wid,'1100','1100'],['xdotool','getwindowgeometry','--shell',wid]]:
            r['commands'].append(args);p=subprocess.run(args,env=env,capture_output=True,text=True,check=True)
            if 'getwindowgeometry' in args:r['window_geometry']=p.stdout
        time.sleep(1.5)
        def capture(name):
            path=root/f'docs/team/roadmap-{batch}/gallery-{name}.png';image=ImageGrab.grab(xdisplay=display);image.save(path)
            r['captures'].append({'captured_at':now(),'path':str(path.relative_to(root)),'dimensions':list(image.size),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
        args=['xdotool','mousemove','--window',wid,'700','500','click','--repeat','50','--delay','25','5'];r['commands'].append(args)
        subprocess.run(args,env=env,check=True,stdout=log,stderr=log,timeout=15)
        time.sleep(.8)
        if slug != "_":capture(slug)
        time.sleep(.23)
        capture("spinner-next")
        for click in clicks:
            name,x,y,*delay=click.split(':')
            if x == 'drag':
                x1,y1,x2,y2=y.split(',')
                for args in [['xdotool','mousemove','--window',wid,x1,y1],['xdotool','mousedown','1'],['xdotool','mousemove','--window',wid,x2,y2],['xdotool','mouseup','1']]:
                    r['commands'].append(args)
                    subprocess.run(args,env=env,check=True,stdout=log,stderr=log,timeout=10)
                    time.sleep(.12)
                time.sleep(.6)
                if name != '_':capture(name)
                continue
            if x == 'key':
                args=['xdotool','windowfocus','--sync',wid];r['commands'].append(args)
                subprocess.run(args,env=env,check=True,stdout=log,stderr=log,timeout=10)
                args=['xdotool','key',*y.split(',')]
            else:args=['xdotool','mousemove','--window',wid,x,y,'click','1']
            r['commands'].append(args);subprocess.run(args,env=env,check=True,stdout=log,stderr=log,timeout=10);time.sleep(float(delay[0]) if delay else .6)
            if name != "_":capture(name)
        time.sleep(.23)
        capture("spinner-reduced-still")
        region=(248,214,275,241)
        def changed(a,b):
            diff=ImageChops.difference(Image.open(root/f'docs/team/roadmap-{batch}/gallery-{a}.png').crop(region),Image.open(root/f'docs/team/roadmap-{batch}/gallery-{b}.png').crop(region))
            return sum(any(pixel) for pixel in diff.getdata())
        r['spinner_region']=list(region)
        r['normal_changed_pixels']=changed('spinner','spinner-next')
        r['reduced_changed_pixels']=changed('spinner-reduced','spinner-reduced-still')
        assert r['normal_changed_pixels'] > 0 and r['reduced_changed_pixels'] == 0
        r['status']='PASS' 
    finally:
        for p in [app,server]:
            if p is not None:
                p.terminate()
                try:p.wait(timeout=5)
                except subprocess.TimeoutExpired:p.kill();p.wait()
r.update(finished_at=now(),log=str(out/'native-window.txt'),log_sha256=hashlib.sha256((out/'native-window.txt').read_bytes()).hexdigest(),capture='PIL.ImageGrab.grab(xdisplay=display)',cwd=str(cwd))
(out/'native-window.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
