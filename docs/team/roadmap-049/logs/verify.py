import os
import subprocess
import sys
from pathlib import Path

os.environ['PYTHONDONTWRITEBYTECODE'] = '1'
checks = [
    ('final-targeted', ['cargo', 'test', '--locked', '-p', 'acme-ui', '--all-features', '-p', 'acme-ui-gallery']),
    ('final-fmt', ['cargo', 'fmt', '-p', 'acme-ui', '-p', 'acme-ui-gallery', '--', '--check']),
    ('final-clippy', ['cargo', 'clippy', '--locked', '-p', 'acme-ui', '-p', 'acme-ui-gallery', '--all-features', '--all-targets', '--no-deps', '--', '-D', 'warnings']),
    ('workspace-check', ['cargo', 'check', '--locked', '--workspace', '--all-targets']),
    ('workspace-tests', ['cargo', 'test', '--locked', '--workspace']),
    ('gallery-build', ['cargo', 'build', '--locked', '-p', 'acme-ui-gallery']),
    ('gallery-release', ['cargo', 'build', '--locked', '--release', '-p', 'acme-ui-gallery']),
]
for label, command in checks:
    result = subprocess.run([sys.executable, str(Path(__file__).with_name('check.py')), label, *command])
    if result.returncode:
        sys.exit(result.returncode)
sys.exit(subprocess.run([sys.executable, 'scripts/verify_sdk.py', '--profile', 'cpu', '--output', 'docs/team/roadmap-049/logs/cpu']).returncode)
