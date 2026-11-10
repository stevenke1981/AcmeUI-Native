from pathlib import Path
from PIL import Image,ImageChops
root=Path('/workspace/AcmeUI-Native/docs/team/roadmap-027')
base=Image.open(root/'gallery-mobile-list-items.png').convert('RGB').crop((200,0,1100,40))
scrolled=Image.open(root/'gallery-mobile-list-keyboard.png').convert('RGB').crop((200,0,1100,40))
diff=ImageChops.difference(base,scrolled)
raw=diff.tobytes()
changed=sum(any(raw[i:i+3]) for i in range(0,len(raw),3))
print(f'Native CPU toolbar changed pixels after keyboard scrolling: {changed} / {base.width*base.height}')
assert changed == 0, 'Scrolled page text must not paint over the toolbar'
